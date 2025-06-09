// Copyright (c) The Hummanta Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    diagnostics::{Diagnostic, Diagnostics, ErrorType, Level},
    helpers::{CodeLocation, OptionalCodeLocation},
    parser::{ast as pt, visitor::Visitor},
    semantic::{
        ast::{
            ContractDefinition, Expression, Function, Parameter, Statement, Symbol, Type, Variable,
        },
        context::{Context, ResolveTypeContext},
        contract::is_base,
        expression::{resolve_expression::expression, ExprContext, ResolveTo},
        symtable::Symtable,
        tag::resolve_tags,
        visitor::{SemanticVisitable, SemanticVisitor},
    },
};
use thiserror::Error;

#[allow(dead_code)]
pub struct DelayedResolveInitializer {
    var_no: usize,
    contract_no: usize,
    initializer: pt::Expression,
}

#[allow(dead_code)]
pub struct VariableResolver<'a> {
    /// Shared context for diagnostics and state
    ctx: &'a mut Context,
    no: usize,
    contract: Option<ContractDefinition>,
    contract_no: Option<usize>,
    symtable: &'a mut Symtable,
}

impl<'a> VariableResolver<'a> {
    /// Creates a new variable resolver with the given context
    pub fn new(
        ctx: &'a mut Context,
        no: usize,
        contract: Option<ContractDefinition>,
        contract_no: Option<usize>,
        symtable: &'a mut Symtable,
    ) -> Self {
        Self { ctx, no, contract, contract_no, symtable }
    }
}

/// Internal error type for variable resolution logic
#[derive(Debug, Error)]
pub enum VariableResolverError {
    #[error("collect_parameters return none")]
    CollectParameters,
}

impl<'a> SemanticVisitor for VariableResolver<'a> {
    fn visit_sema_source_unit_part(
        &mut self,
        part: &mut super::ast::SourceUnitPart,
    ) -> Result<(), Self::Error> {
        if let pt::SourceUnitPart::VariableDefinition(_) = part.part {
            self.ctx.reject(&part.annotations, "variable");
            part.visit(self)?;
        }

        Ok(())
    }
}

impl<'a> Visitor for VariableResolver<'a> {
    type Error = VariableResolverError;

    fn visit_var_definition(
        &mut self,
        def: &mut pt::VariableDefinition,
    ) -> Result<(), Self::Error> {
        let mut attrs = def.attrs.clone();
        let mut ty = def.ty.clone();

        // For function types, the parser adds the attributes incl visibility to the type,
        // not the pt::VariableDefinition attrs. We need to chomp off the visibility
        // from the attributes before resolving the type
        if let pt::Expression::Type(_, pt::Type::Function { attributes, returns, .. }) = &mut ty {
            let mut filter_var_attrs = |attributes: &mut Vec<pt::FunctionAttribute>| {
                if attributes.is_empty() {
                    return;
                }

                let mut seen_visibility = false;

                // here we must iterate in reverse order: we can only remove the *last* visibility
                // attribute This is due to the insane syntax
                // contract c {
                //    function() external internal x;
                // }
                // The first external means the function type is external, the second internal means
                // the visibility of the x is internal.
                for attr_no in (0..attributes.len()).rev() {
                    if let pt::FunctionAttribute::Immutable(loc) = &attributes[attr_no] {
                        attrs.push(pt::VariableAttribute::Immutable(*loc));
                        attributes.remove(attr_no);
                    } else if !seen_visibility {
                        if let pt::FunctionAttribute::Visibility(v) = &attributes[attr_no] {
                            attrs.push(pt::VariableAttribute::Visibility(v.clone()));
                            attributes.remove(attr_no);
                            seen_visibility = true;
                        }
                    }
                }
            };

            if let Some((_, trailing_attributes)) = returns {
                filter_var_attrs(trailing_attributes);
            } else {
                filter_var_attrs(attributes);
            }
        }

        let mut diagnostics = Diagnostics::default();

        let ty = match self.ctx.resolve_type(
            self.no,
            self.contract_no,
            ResolveTypeContext::None,
            &ty,
            &mut diagnostics,
        ) {
            Ok(s) => s,
            Err(()) => {
                self.ctx.diagnostics.extend(diagnostics);
                return Ok(());
            }
        };

        let mut constant = false;
        let mut visibility: Option<pt::Visibility> = None;
        let mut has_immutable: Option<pt::Loc> = None;
        let mut is_override: Option<(pt::Loc, Vec<usize>)> = None;
        let mut storage_type: Option<pt::StorageType> = None;

        for attr in attrs {
            match &attr {
                pt::VariableAttribute::Constant(loc) => {
                    if constant {
                        self.ctx.diagnostics.push(Diagnostic::error(
                            *loc,
                            "duplicate constant attribute".to_string(),
                        ));
                    }
                    constant = true;
                }
                pt::VariableAttribute::Immutable(loc) => {
                    if let Some(prev) = &has_immutable {
                        self.ctx.diagnostics.push(
                            Diagnostic::builder(*loc, Level::Error)
                                .message("duplicate 'immutable' attribute")
                                .note(*prev, "previous 'immutable' attribute")
                                .build(),
                        );
                    }
                    has_immutable = Some(*loc);
                }
                pt::VariableAttribute::Override(loc, bases) => {
                    if let Some((prev, _)) = &is_override {
                        self.ctx.diagnostics.push(
                            Diagnostic::builder(*loc, Level::Error)
                                .message("duplicate 'override' attribute")
                                .note(*prev, "previous 'override' attribute")
                                .build(),
                        );
                    }

                    let mut list = Vec::new();
                    let mut diagnostics = Diagnostics::default();

                    if let Some(contract_no) = self.contract_no {
                        for name in bases {
                            if let Ok(no) = self.ctx.resolve_contract_with_namespace(
                                self.no,
                                name,
                                &mut diagnostics,
                            ) {
                                if list.contains(&no) {
                                    diagnostics.push(Diagnostic::error(
                                        name.loc,
                                        format!("duplicate override '{name}'"),
                                    ));
                                } else if !is_base(no, contract_no, self.ctx) {
                                    diagnostics.push(Diagnostic::error(
                                        name.loc,
                                        format!(
                                            "override '{}' is not a base contract of '{}'",
                                            name, self.ctx.contracts[contract_no].id
                                        ),
                                    ));
                                } else {
                                    list.push(no);
                                }
                            }
                        }

                        is_override = Some((*loc, list));
                    } else {
                        diagnostics.push(Diagnostic::error(
                            *loc,
                            "global variable has no bases contracts to override".to_string(),
                        ));
                    }

                    self.ctx.diagnostics.extend(diagnostics);
                }
                pt::VariableAttribute::Visibility(v) if self.contract_no.is_none() => {
                    self.ctx.diagnostics.push(Diagnostic::error(
                        v.loc_opt().unwrap(),
                        format!("'{v}': global variable cannot have visibility specifier"),
                    ));
                    return Ok(());
                }
                pt::VariableAttribute::Visibility(pt::Visibility::External(loc)) => {
                    self.ctx.diagnostics.push(Diagnostic::error(
                        loc.unwrap(),
                        "variable cannot be declared external".to_string(),
                    ));
                    return Ok(());
                }
                pt::VariableAttribute::Visibility(v) => {
                    if let Some(e) = &visibility {
                        self.ctx.diagnostics.push(
                            Diagnostic::builder(v.loc_opt().unwrap(), Level::Error)
                                .message(format!("variable visibility redeclared '{v}'"))
                                .note(
                                    e.loc_opt().unwrap(),
                                    format!("location of previous declaration of '{e}'"),
                                )
                                .build(),
                        );
                        return Ok(());
                    }

                    visibility = Some(v.clone());
                }
                pt::VariableAttribute::StorageType(s) => {
                    if storage_type.is_some() {
                        self.ctx.diagnostics.push(Diagnostic::error(
                            attr.loc(),
                            format!(
                                "mutliple storage type specifiers for '{}'",
                                def.name.as_ref().unwrap().name
                            ),
                        ));
                    } else {
                        storage_type = Some(s.clone());
                    }
                }
            }
        }

        if let Some(loc) = &has_immutable {
            if constant {
                self.ctx.diagnostics.push(Diagnostic::error(
                    *loc,
                    "variable cannot be declared both 'immutable' and 'constant'".to_string(),
                ));
                constant = false;
            }
        }

        let visibility = match visibility {
            Some(v) => v,
            None => pt::Visibility::Internal(Some(def.ty.loc())),
        };

        if let pt::Visibility::Public(_) = &visibility {
            // override allowed
        } else if let Some((loc, _)) = &is_override {
            self.ctx.diagnostics.push(Diagnostic::error(
                *loc,
                "only public variable can be declared 'override'".to_string(),
            ));
            is_override = None;
        }

        if let Some(contract) = &self.contract {
            if matches!(contract.ty, pt::ContractTy::Interface(_)) ||
                (matches!(contract.ty, pt::ContractTy::Library(_)) && !constant)
            {
                if contract.name.is_none() || def.name.is_none() {
                    return Ok(());
                }
                self.ctx.diagnostics.push(Diagnostic::error(
                    def.loc,
                    format!(
                        "{} '{}' is not allowed to have contract variable '{}'",
                        contract.ty,
                        contract.name.as_ref().unwrap().name,
                        def.name.as_ref().unwrap().name
                    ),
                ));
                return Ok(());
            }
        } else {
            if !constant {
                self.ctx.diagnostics.push(Diagnostic::error(
                    def.ty.loc(),
                    "global variable must be constant".to_string(),
                ));
                return Ok(());
            }
            if ty.contains_internal_function(self.ctx) {
                self.ctx.diagnostics.push(Diagnostic::error(
                    def.ty.loc(),
                    "global variable cannot be of type internal function".to_string(),
                ));
                return Ok(());
            }
        }

        if ty.contains_internal_function(self.ctx) &&
            matches!(visibility, pt::Visibility::Public(_) | pt::Visibility::External(_))
        {
            self.ctx.diagnostics.push(Diagnostic::error(
                def.ty.loc(),
                format!("variable of type internal function cannot be '{visibility}'"),
            ));
            return Ok(());
        }

        let mut diagnostics = Diagnostics::default();

        let initializer = if constant {
            if let Some(initializer) = &def.initializer {
                let mut context = ExprContext {
                    no: self.no,
                    contract_no: self.contract_no,
                    constant,
                    ..Default::default()
                };
                context.enter_scope();

                match expression(
                    initializer,
                    &mut context,
                    self.ctx,
                    self.symtable,
                    &mut diagnostics,
                    ResolveTo::Type(&ty),
                ) {
                    Ok(res) => {
                        // implicitly conversion to correct ty
                        match res.cast(&def.loc, &ty, true, self.ctx, &mut diagnostics) {
                            Ok(res) => {
                                res.check_constant_overflow(&mut diagnostics);
                                Some(res)
                            }
                            Err(_) => None,
                        }
                    }
                    Err(()) => None,
                }
            } else {
                diagnostics.push(
                    Diagnostic::builder(def.loc, Level::Error)
                        .ty(ErrorType::DeclarationError)
                        .message("missing initializer for constant")
                        .build(),
                );

                None
            }
        } else {
            None
        };

        self.ctx.diagnostics.extend(diagnostics);

        let bases = self.contract_no.map(|contract_no| self.ctx.contract_bases(contract_no));

        let tags = resolve_tags(
            def.name.as_ref().unwrap().loc.no(),
            if self.contract_no.is_none() { "global variable" } else { "state variable" },
            None,
            None,
            bases,
            self.ctx,
        );

        let sdecl = Variable {
            name: def.name.as_ref().unwrap().name.to_string(),
            loc: def.loc,
            tags,
            visibility: visibility.clone(),
            ty: ty.clone(),
            constant,
            immutable: has_immutable.is_some(),
            assigned: def.initializer.is_some(),
            initializer,
            read: matches!(visibility, pt::Visibility::Public(_)),
            storage_type,
        };

        let var_no = if let Some(contract_no) = self.contract_no {
            let var_no = self.ctx.contracts[contract_no].variables.len();
            self.ctx.contracts[contract_no].variables.push(sdecl);
            var_no
        } else {
            let var_no = self.ctx.constants.len();
            self.ctx.constants.push(sdecl);
            var_no
        };

        let success = self.ctx.add_symbol(
            self.no,
            self.contract_no,
            def.name.as_ref().unwrap(),
            Symbol::Variable(def.loc, self.contract_no, var_no),
        );

        // for public variables in contracts, create an accessor function
        if success && matches!(visibility, pt::Visibility::Public(_)) {
            if let Some(contract_no) = self.contract_no {
                // The accessor function returns the value of the storage variable, constant or not.
                let mut expr = if constant {
                    Expression::ConstantVariable {
                        loc: pt::Loc::Implicit,
                        ty: ty.clone(),
                        contract_no: Some(contract_no),
                        var_no,
                    }
                } else {
                    Expression::StorageVariable {
                        loc: pt::Loc::Implicit,
                        ty: Type::StorageRef(false, Box::new(ty.clone())),
                        contract_no,
                        var_no,
                    }
                };

                // If the variable is an array or mapping, the accessor function takes mapping keys
                // or array indices as arguments, and returns the dereferenced value
                let mut symtable = Symtable::default();
                let mut context = ExprContext::default();
                context.enter_scope();
                let mut params = Vec::new();
                let param = collect_parameters(
                    &ty,
                    &def.name,
                    &mut symtable,
                    &mut context,
                    &mut params,
                    &mut expr,
                    self.ctx,
                )
                .ok_or(VariableResolverError::CollectParameters)?;

                if param.ty.contains_mapping(self.ctx) {
                    // we can't return a mapping
                    self.ctx.diagnostics.push(
                        Diagnostic::builder(def.loc, Level::Error)
                            .ty(ErrorType::DeclarationError)
                            .message("mapping in a struct variable cannot be public")
                            .build(),
                    );
                }

                let (body, returns) =
                    accessor_body(expr, param, constant, &mut symtable, &mut context, self.ctx);

                let mut func = Function::new(
                    def.name.as_ref().unwrap().loc,
                    def.name.as_ref().unwrap().loc,
                    def.name.as_ref().unwrap().clone(),
                    Some(contract_no),
                    Vec::new(),
                    pt::FunctionTy::Function,
                    // accessors for constant variables have view mutability
                    Some(pt::Mutability::View(def.name.as_ref().unwrap().loc)),
                    pt::Visibility::External(None),
                    params,
                    returns,
                    self.ctx,
                );

                func.body = body;
                func.is_accessor = true;
                func.has_body = true;
                func.is_override = is_override;
                func.symtable = symtable;

                // add the function to the namespace and then to our contract
                let func_no = self.ctx.functions.len();

                self.ctx.functions.push(func);

                self.ctx.contracts[contract_no].functions.push(func_no);

                // we already have a symbol for
                let symbol = Symbol::Function(vec![(def.loc, func_no)]);

                self.ctx.function_symbols.insert(
                    (def.loc.no(), Some(contract_no), def.name.as_ref().unwrap().name.to_owned()),
                    symbol,
                );
            }
        }

        // ret

        Ok(())
    }
}

pub fn contract_variables(
    _def: &ContractDefinition,
    _no: usize,
    _ctx: &mut Context,
) -> Vec<DelayedResolveInitializer> {
    todo!()
}

pub fn resolve_initializers(
    _initializers: &[DelayedResolveInitializer],
    _no: usize,
    _ctx: &mut Context,
) {
    todo!()
}

#[allow(unused_variables)]
#[allow(clippy::ptr_arg)]
/// For accessor functions, create the parameter list and the return expression
fn collect_parameters(
    ty: &Type,
    name: &Option<pt::Identifier>,
    symtable: &mut Symtable,
    context: &mut ExprContext,
    params: &mut Vec<Parameter<Type>>,
    expr: &mut Expression,
    ctx: &mut Context,
) -> Option<Parameter<Type>> {
    todo!()
}

#[allow(unused_variables)]
/// Build up an ast for the implict accessor function for public state variables.
fn accessor_body(
    expr: Expression,
    param: Parameter<Type>,
    constant: bool,
    symtable: &mut Symtable,
    context: &mut ExprContext,
    ctx: &mut Context,
) -> (Vec<Statement>, Vec<Parameter<Type>>) {
    todo!()
}

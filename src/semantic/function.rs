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
    diagnostics::{Diagnostic, Diagnostics, Level},
    helpers::{CodeLocation, OptionalCodeLocation},
    parser::{
        ast::{self as pt, FunctionDefinition, FunctionTy, Loc},
        visitor::{Visitable, Visitor},
    },
    semantic::{
        ast::{ContractDefinition, Function, Parameter, ParameterAnnotation, Symbol, Type},
        context::{Context, ResolveTypeContext},
        tag::resolve_tags,
        visitor::{SemanticVisitable, SemanticVisitor},
    },
};

use thiserror::Error;

/// Resolve free function
pub struct FunctionResolver<'a> {
    /// Shared context for diagnostics and state
    ctx: &'a mut Context,
    no: usize,
    success: bool,
    mutability: Option<pt::Mutability>,
    params_success: bool,
    params: Vec<Parameter<Type>>,
    func_ty: Option<pt::FunctionTy>,
    is_internal: bool,
    contract_no: Option<usize>,
    returns_success: bool,
    returns: Vec<Parameter<Type>>,
    resolve_bodies: Vec<(usize, Box<FunctionDefinition>)>,
}

impl<'a> FunctionResolver<'a> {
    /// Creates a new function resolver with the given context
    pub fn new(ctx: &'a mut Context, no: usize) -> Self {
        Self {
            ctx,
            no,
            success: true,
            mutability: None,
            params_success: true,
            params: Vec::new(),
            func_ty: None,
            is_internal: false,
            contract_no: None,
            returns_success: true,
            returns: Vec::new(),
            resolve_bodies: Vec::new(),
        }
    }
}

/// Internal error type for function resolution logic
#[derive(Debug, Error)]
pub enum FunctionResolverError {}

impl<'a> SemanticVisitor for FunctionResolver<'a> {
    fn visit_sema_source_unit_part(
        &mut self,
        part: &mut super::ast::SourceUnitPart,
    ) -> Result<(), Self::Error> {
        if let pt::SourceUnitPart::FunctionDefinition(_) = part.part {
            self.ctx.reject(&part.annotations, "function");
            part.visit(self)?;
        }

        Ok(())
    }
}

impl<'a> Visitor for FunctionResolver<'a> {
    type Error = FunctionResolverError;

    fn visit_function(&mut self, func: &mut pt::FunctionDefinition) -> Result<(), Self::Error> {
        self.func_ty.replace(func.ty);

        func.attributes.visit(self)?;

        self.is_internal = true;
        self.contract_no = None;

        func.params.visit(self)?;

        // let (returns, returns_success) =
        //     resolve_returns(&func.returns, true, self.no, None, self.ctx, &mut diagnostics);

        if func.body.is_none() {
            self.ctx
                .diagnostics
                .push(Diagnostic::error(func.loc_prototype, "missing function body"));
            self.success = false;
        }

        if !self.success || !self.returns_success || !self.params_success {
            return Ok(());
        }

        let name = match &func.name {
            Some(s) => s.to_owned(),
            None => {
                self.ctx
                    .diagnostics
                    .push(Diagnostic::error(func.loc_prototype, "missing function name"));
                return Ok(());
            }
        };

        let doc = resolve_tags(
            func.loc_prototype.no(),
            "function",
            Some(&self.params),
            Some(&self.returns),
            None,
            self.ctx,
        );

        let mut fdecl = Function::new(
            func.loc_prototype,
            func.loc,
            name,
            None,
            doc,
            func.ty,
            self.mutability.clone(),
            pt::Visibility::Internal(None),
            self.params.clone(),
            self.returns.clone(),
            self.ctx,
        );

        fdecl.has_body = true;

        let id = func.name.as_ref().unwrap();

        if let Some(prev) = self.ctx.functions.iter().find(|f| fdecl.signature == f.signature) {
            self.ctx.diagnostics.push(
                Diagnostic::builder(func.loc_prototype, Level::Error)
                    .message(format!("overloaded {} with this signature already exist", func.ty))
                    .note(prev.loc_prototype, "location of previous definition")
                    .build(),
            );
            return Ok(());
        }

        let func_no = self.ctx.functions.len();

        self.ctx.functions.push(fdecl);

        if let Some(Symbol::Function(ref mut v)) =
            self.ctx.function_symbols.get_mut(&(self.no, None, id.name.to_owned()))
        {
            v.push((func.loc_prototype, func_no));
        } else {
            self.ctx.add_symbol(self.no, None, id, Symbol::Function(vec![(id.loc, func_no)]));
        }

        self.resolve_bodies.push((func_no, Box::new(func.clone())));

        Ok(())
    }

    fn visit_function_attribute(
        &mut self,
        attribute: &mut pt::FunctionAttribute,
    ) -> Result<(), Self::Error> {
        match attribute {
            pt::FunctionAttribute::Immutable(loc) => {
                self.ctx
                    .diagnostics
                    .push(Diagnostic::error(*loc, "function cannot be declared 'immutable'"));
                self.success = false;
            }
            pt::FunctionAttribute::Mutability(m) => {
                if let Some(e) = &self.mutability {
                    self.ctx.diagnostics.push(
                        Diagnostic::builder(m.loc(), Level::Error)
                            .message(format!("function redeclared '{m}'"))
                            .note(e.loc(), format!("location of previous declaration of '{e}'"))
                            .build(),
                    );
                    self.success = false;
                }

                if let pt::Mutability::Constant(loc) = m {
                    self.ctx.diagnostics.push(Diagnostic::warning(
                        *loc,
                        "'constant' is deprecated. Use 'view' instead",
                    ));

                    self.mutability.replace(pt::Mutability::View(*loc));
                } else {
                    self.mutability.replace(m.clone());
                }
            }
            pt::FunctionAttribute::Visibility(v) => {
                self.ctx.diagnostics.push(Diagnostic::error(
                    v.loc_opt().unwrap(),
                    format!("'{v}': only functions in contracts can have a visibility specifier"),
                ));
                self.success = false;
            }
            pt::FunctionAttribute::Virtual(loc) => {
                self.ctx
                    .diagnostics
                    .push(Diagnostic::error(*loc, "only functions in contracts can be virtual"));
                self.success = false;
            }
            pt::FunctionAttribute::Override(loc, _) => {
                self.ctx
                    .diagnostics
                    .push(Diagnostic::error(*loc, "only functions in contracts can override"));
                self.success = false;
            }
            pt::FunctionAttribute::BaseOrModifier(loc, _) => {
                // We can only fully resolve the base constructors arguments
                // once we have resolved all the constructors, this is not done here yet
                // so we fully resolve these along with the constructor body
                self.ctx.diagnostics.push(Diagnostic::error(
                    *loc,
                    "function modifiers or base contracts are only allowed on functions in contracts",
                ));
                self.success = false;
            }
            pt::FunctionAttribute::Error(_) => {
                self.success = false;
            }
        }

        Ok(())
    }

    /// Resolve the parameter
    fn visit_parameter(
        &mut self,
        loc: &Loc,
        parameter: &Option<pt::Parameter>,
    ) -> Result<(), Self::Error> {
        let parameter = match parameter {
            Some(p @ pt::Parameter { ref annotation, .. }) => {
                if annotation.is_some() && self.func_ty != Some(FunctionTy::Constructor) {
                    self.ctx.diagnostics.push(Diagnostic::error(
                        annotation.as_ref().unwrap().loc,
                        "parameter annotations are only allowed in constructors",
                    ));
                    self.params_success = false;
                    return Ok(());
                } else if annotation.is_some() {
                    self.ctx.diagnostics.push(Diagnostic::error(
                        annotation.as_ref().unwrap().loc,
                        "unexpected parameter annotation",
                    ));
                    self.params_success = false;
                    return Ok(());
                }

                p
            }
            None => {
                self.ctx.diagnostics.push(Diagnostic::error(*loc, "missing parameter type"));
                self.params_success = false;
                return Ok(());
            }
        };

        let mut ty_loc = parameter.ty.loc();

        let mut diagnostics = Diagnostics::default();

        match self.ctx.resolve_type(
            self.no,
            self.contract_no,
            ResolveTypeContext::None,
            &parameter.ty,
            &mut diagnostics,
        ) {
            Ok(ty) => {
                if !self.is_internal && ty.contains_internal_function(self.ctx) {
                    self.ctx.diagnostics.push(Diagnostic::error(
                        parameter.ty.loc(),
                        "parameter of type 'function internal' not allowed public or external functions",
                    ));
                    self.params_success = false;
                }

                let ty = if !ty.can_have_data_location() {
                    if let Some(storage) = &parameter.storage {
                        self.ctx.diagnostics.push(Diagnostic::error(
                            storage.loc(),
                            format!("data location '{storage}' can only be specified for array, struct or mapping")
                        ));
                        self.params_success = false;
                    }

                    ty
                } else if let Some(pt::StorageLocation::Storage(loc)) = parameter.storage {
                    if !self.is_internal {
                        self.ctx.diagnostics.push(Diagnostic::error(
                            loc,
                            "parameter of type 'storage' not allowed public or external functions",
                        ));
                        self.params_success = false;
                    }

                    ty_loc.use_end_from(&loc);

                    Type::StorageRef(false, Box::new(ty))
                } else {
                    if ty.contains_mapping(self.ctx) {
                        self.ctx.diagnostics.push(Diagnostic::error(
                            parameter.ty.loc(),
                            "parameter with mapping type must be of type 'storage'",
                        ));
                        self.params_success = false;
                    }

                    if !ty.fits_in_memory(self.ctx) {
                        self.ctx.diagnostics.push(Diagnostic::error(
                            parameter.ty.loc(),
                            "type is too large to fit into memory",
                        ));
                        self.params_success = false;
                    }

                    ty
                };

                let annotation = parameter
                    .annotation
                    .as_ref()
                    .map(|e| ParameterAnnotation { loc: e.loc, id: e.id.clone() });

                self.params.push(Parameter {
                    loc: *loc,
                    id: parameter.name.clone(),
                    ty,
                    ty_loc: Some(ty_loc),
                    indexed: false,
                    readonly: false,
                    infinite_size: false,
                    recursive: false,
                    annotation,
                });
            }
            Err(()) => self.params_success = false,
        }
        self.ctx.diagnostics.extend(diagnostics);

        Ok(())
    }
}

/// Resolve function declaration in a contract
pub fn contract_function(
    _contract: &ContractDefinition,
    _func: &pt::FunctionDefinition,
    _annotations: &[pt::Annotation],
    _no: usize,
    _ctx: &mut Context,
) -> Option<usize> {
    todo!()
}

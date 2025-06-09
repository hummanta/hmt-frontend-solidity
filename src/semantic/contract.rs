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

use thiserror::Error;

use crate::{
    diagnostics::{Diagnostic, Diagnostics, Level, Note},
    parser::{
        ast as pt,
        visitor::{Visitable, Visitor},
    },
    semantic::{
        ast::{Base, ContractDefinition, ContractPart},
        context::Context,
        expression::{constructor::match_constructor_to_args, ExprContext},
        function,
        symtable::Symtable,
        using::UsingResolver,
        variable,
        visitor::SemanticVisitor,
    },
};

/// Resolve the base contracts list and check for cycles.
pub struct BaseContractResolver<'a> {
    /// Shared context for diagnostics and state
    ctx: &'a mut Context,
    no: usize,
    contract_no: usize,
}

impl<'a> BaseContractResolver<'a> {
    /// Creates a new base contract resolver with the given context
    pub fn new(ctx: &'a mut Context, no: usize) -> Self {
        Self { ctx, no, contract_no: 0 }
    }
}

/// Internal error type for contract resolution logic
#[derive(Debug, Error)]
pub enum ContractResolverError {}

impl<'a> SemanticVisitor for BaseContractResolver<'a> {
    fn visit_sema_contract(
        &mut self,
        contract: &mut super::ast::ContractDefinition,
    ) -> Result<(), Self::Error> {
        self.contract_no = contract.contract_no;
        contract.base.visit(self)?;

        Ok(())
    }
}

impl<'a> Visitor for BaseContractResolver<'a> {
    type Error = ContractResolverError;

    fn visit_base(&mut self, base: &mut pt::Base) -> Result<(), Self::Error> {
        let mut diagnostics = Diagnostics::default();

        let contract_no = self.contract_no;
        let contract_id = self.ctx.contracts[contract_no].id.clone();
        let contract_ty = self.ctx.contracts[contract_no].ty.clone();

        if self.ctx.contracts[contract_no].is_library() {
            self.ctx.diagnostics.push(Diagnostic::error(
                base.loc,
                format!("library '{}' cannot have a base contract", contract_id),
            ));
            return Ok(());
        }

        let name = &base.name;

        let Ok(no) = self.ctx.resolve_contract_with_namespace(self.no, name, &mut diagnostics)
        else {
            return Ok(());
        };

        if no == contract_no {
            self.ctx.diagnostics.push(Diagnostic::error(
                name.loc,
                format!("contract '{name}' cannot have itself as a base contract"),
            ));
        } else if self.ctx.contracts[contract_no].bases.iter().any(|e| e.contract_no == no) {
            self.ctx.diagnostics.push(Diagnostic::error(
                name.loc,
                format!("contract '{}' duplicate base '{}'", contract_id, name),
            ));
        } else if is_base(contract_no, no, self.ctx) {
            self.ctx.diagnostics.push(Diagnostic::error(
                name.loc,
                format!("base '{}' from contract '{}' is cyclic", name, contract_id),
            ));
        } else if self.ctx.contracts[contract_no].is_interface() &&
            !self.ctx.contracts[no].is_interface()
        {
            self.ctx.diagnostics.push(Diagnostic::error(
                name.loc,
                format!(
                    "interface '{}' cannot have {} '{}' as a base",
                    contract_id, self.ctx.contracts[no].ty, name
                ),
            ));
        } else if self.ctx.contracts[no].is_library() {
            self.ctx.diagnostics.push(Diagnostic::error(
                name.loc,
                format!(
                    "library '{}' cannot be used as base contract for {} '{}'",
                    name, contract_ty, contract_id,
                ),
            ));
        } else {
            // We do not resolve the constructor arguments here, since we have not
            // resolved any variables. This means no constants can be used on base
            // constructor args, so we delay this until resolve_base_args()
            self.ctx.contracts[self.contract_no].bases.push(Base {
                loc: base.loc,
                contract_no: no,
                constructor: None,
            });
        }

        self.ctx.diagnostics.extend(diagnostics);

        Ok(())
    }
}

/// Function bodies and state variable initializers can only be resolved once
/// all function prototypes, bases contracts and state variables are resolved.
#[derive(Default)]
struct ResolveLater {
    function_bodies: Vec<DelayedResolveFunction>,
    initializers: Vec<variable::DelayedResolveInitializer>,
}

/// Function body which should be resolved.
/// List of function_no, contract_no, and function parse tree
#[allow(dead_code)]
struct DelayedResolveFunction {
    function_no: usize,
    contract_no: usize,
    function: pt::FunctionDefinition,
    annotations: Vec<pt::Annotation>,
}

/// Resolve the contracts.
pub struct ContractResolver<'a> {
    /// Shared context for diagnostics and state
    ctx: &'a mut Context,
    no: usize,
    delayed: ResolveLater,
    part: Option<ContractPart>,
    contract_no: usize,
}

impl<'a> ContractResolver<'a> {
    /// Creates a new contract resolver with the given context
    pub fn new(ctx: &'a mut Context, no: usize) -> Self {
        Self { ctx, no, delayed: Default::default(), part: None, contract_no: 0 }
    }

    /// Resolve functions declarations, constructor declarations, and contract variables
    /// This returns a list of function bodies to resolve
    fn resolve_declarations(&mut self, def: &ContractDefinition) {
        self.ctx.diagnostics.push(Diagnostic::debug(
            def.loc,
            format!("found {} '{}'", def.ty, def.name.as_ref().unwrap().name),
        ));

        let mut function_no_bodies = Vec::new();

        // resolve state variables. We may need a constant to resolve the array
        // dimension of a function argument.
        self.delayed.initializers.extend(variable::contract_variables(def, self.no, self.ctx));

        // resolve function signatures
        for part in &def.parts {
            if let pt::ContractPart::FunctionDefinition(ref f) = &part.part {
                if let Some(function_no) =
                    function::contract_function(def, f, &part.annotations, self.no, self.ctx)
                {
                    if f.body.is_some() {
                        self.delayed.function_bodies.push(DelayedResolveFunction {
                            contract_no: def.contract_no,
                            function_no,
                            function: f.as_ref().clone(),
                            annotations: part.annotations.clone(),
                        });
                    } else {
                        function_no_bodies.push(function_no);
                    }
                }
            }
        }

        if let pt::ContractTy::Contract(loc) = &def.ty {
            if !function_no_bodies.is_empty() {
                let notes = function_no_bodies
                    .into_iter()
                    .map(|function_no| Note {
                        loc: self.ctx.functions[function_no].loc_prototype,
                        message: format!(
                            "location of function '{}' with no body",
                            self.ctx.functions[function_no].id
                        ),
                    })
                    .collect::<Vec<Note>>();

                self.ctx.diagnostics.push(
                    Diagnostic::builder(*loc, Level::Error).message(format!(
                        "contract should be marked 'abstract contract' since it has {} functions with no body",
                        notes.len()
                    )).notes(notes).build()
                );
            }
        }
    }

    /// Check the inheritance of all functions and other symbols
    fn check_inheritance(&mut self) {
        todo!()
    }

    /// This function checks which function names must be mangled given a
    /// contract. Mangling happens when there is more than one function with the
    /// same name in the given `contract_no`.
    fn mangle_function_names(&mut self) {
        todo!()
    }

    /// This check guarantees that each public Solidity function has a unique selector.
    fn verify_unique_selector(&mut self) {
        todo!()
    }

    /// Constructors and functions are no different pallet contracts.
    /// This function checks that all constructors and function names are unique.
    /// Overloading (mangled function or constructor names) is taken into account.
    fn unique_constructor_names(&mut self) {
        todo!()
    }

    /// Given a contract number, check for function names conflicting with any mangled name.
    /// Only applies to public functions.
    ///
    /// Note: In sema we do not care about the function name too much.
    /// The mangled name is consumed later by the ABI generation.
    fn check_mangled_function_names(&mut self) {
        todo!()
    }

    /// Resolve contract functions bodies
    fn resolve_bodies(&mut self) -> bool {
        todo!()
    }

    /// Check if we have arguments for all the base contracts
    fn check_base_args(&mut self) {
        todo!()
    }
}

impl<'a> SemanticVisitor for ContractResolver<'a> {
    fn visit_sema_contract(
        &mut self,
        contract: &mut ContractDefinition,
    ) -> Result<(), Self::Error> {
        self.contract_no = contract.contract_no;

        self.resolve_declarations(contract);

        // Now we have all the declarations, we can handle base contracts
        self.check_inheritance();
        self.mangle_function_names();
        self.verify_unique_selector();
        self.unique_constructor_names();
        self.check_mangled_function_names();

        // Now we can resolve the initializers
        variable::resolve_initializers(&self.delayed.initializers, self.no, self.ctx);

        // Now we can resolve the bodies
        if !self.resolve_bodies() {
            self.check_base_args();
        }
        Ok(())
    }

    fn visit_sema_contract_part(&mut self, part: &mut ContractPart) -> Result<(), Self::Error> {
        self.part.replace(part.clone());
        Ok(())
    }
}

impl<'a> Visitor for ContractResolver<'a> {
    type Error = ContractResolverError;

    /// Resolve base contract constructor arguments on contract definition
    ///  (not constructor definitions)
    fn visit_base(&mut self, base: &mut pt::Base) -> Result<(), Self::Error> {
        let mut diagnostics = Diagnostics::default();

        let mut context =
            ExprContext { no: self.no, contract_no: Some(self.contract_no), ..Default::default() };
        context.enter_scope();

        let name = &base.name;

        let Ok(base_no) = self.ctx.resolve_contract_with_namespace(self.no, name, &mut diagnostics)
        else {
            self.ctx.diagnostics.extend(diagnostics);
            return Ok(());
        };

        let Some(pos) = self.ctx.contracts[self.contract_no]
            .bases
            .iter()
            .position(|e| e.contract_no == base_no)
        else {
            self.ctx.diagnostics.extend(diagnostics);
            return Ok(());
        };

        if let Some(args) = &base.args {
            let mut symtable = Symtable::default();

            // find constructor which matches this
            if let Ok((Some(constructor_no), args)) = match_constructor_to_args(
                &base.loc,
                args,
                base_no,
                &mut context,
                self.ctx,
                &mut symtable,
                &mut diagnostics,
            ) {
                self.ctx.contracts[self.contract_no].bases[pos].constructor =
                    Some((constructor_no, args));
            }
        }

        self.ctx.diagnostics.extend(diagnostics);
        Ok(())
    }

    /// Resolve the using declarations in a contract
    fn visit_using(&mut self, using: &mut pt::Using) -> Result<(), Self::Error> {
        if let Some(part) = &self.part {
            self.ctx.reject(&part.annotations, "using");
        }

        let mut resolver = UsingResolver::new(self.ctx, self.no, Some(self.contract_no));
        let _ = resolver.visit_using(using);

        if let Some(using) = resolver.finish() {
            self.ctx.contracts[self.contract_no].using.push(using);
        }

        Ok(())
    }
}

// Is a contract a base of another contract
pub fn is_base(base: usize, derived: usize, ctx: &Context) -> bool {
    let bases = &ctx.contracts[derived].bases;

    if base == derived || bases.iter().any(|e| e.contract_no == base) {
        return true;
    }

    bases.iter().any(|parent| is_base(base, parent.contract_no, ctx))
}

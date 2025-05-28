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
    diagnostics::{Diagnostic, Diagnostics},
    parser::{
        ast as pt,
        visitor::{Visitable, Visitor},
    },
};

use super::{ast::Base, context::Context, visitor::SemanticVisitor};

pub struct ContractResolver<'a> {
    /// Shared context for diagnostics and state
    ctx: &'a mut Context,
    no: usize,
    contract_no: usize,
}

impl<'a> ContractResolver<'a> {
    /// Creates a new contract resolver with the given context
    pub fn new(ctx: &'a mut Context, no: usize) -> Self {
        Self { ctx, no, contract_no: 0 }
    }

    // Is a contract a base of another contract
    pub fn is_base(&self, base: usize, derived: usize) -> bool {
        let bases = &self.ctx.contracts[derived].bases;

        if base == derived || bases.iter().any(|e| e.contract_no == base) {
            return true;
        }

        bases.iter().any(|parent| self.is_base(base, parent.contract_no))
    }
}

/// Internal error type for contract resolution logic
#[derive(Debug, Error)]
pub enum ContractResolverError {}

impl<'a> SemanticVisitor for ContractResolver<'a> {
    fn visit_sema_contract(
        &mut self,
        contract: &mut super::ast::ContractDefinition,
    ) -> Result<(), Self::Error> {
        self.contract_no = contract.contract_no;
        contract.base.visit(self)?;

        Ok(())
    }
}

impl<'a> Visitor for ContractResolver<'a> {
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
        } else if self.is_base(contract_no, no) {
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

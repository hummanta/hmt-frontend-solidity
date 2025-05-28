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
    diagnostics::Diagnostic,
    helpers::CodeLocation,
    parser::{
        ast as pt,
        visitor::{Visitable, Visitor},
    },
};

use super::{
    ast::{
        ContractDefinition, ErrorDecl, EventDecl, SourceUnitPart, StructDecl, StructType, Symbol,
    },
    context::Context,
    visitor::SemanticVisitor,
};

/// List the types which should be resolved later
#[derive(Default)]
pub struct ResolveFields {
    structs: Vec<ResolveStructFields>,
    events: Vec<ResolveEventFields>,
    errors: Vec<ResolveErrorFields>,
}

#[allow(dead_code)]
struct ResolveEventFields {
    event_no: usize,
    pt: pt::EventDefinition,
}

#[allow(dead_code)]
struct ResolveErrorFields {
    error_no: usize,
    pt: pt::ErrorDefinition,
}

#[allow(dead_code)]
struct ResolveStructFields {
    struct_no: usize,
    pt: pt::StructDefinition,
    contract: Option<usize>,
}

/// Resolve all the types we can find (enums, structs, contracts).
/// structs can have other structs as fields, include ones that
/// have not been declared yet.
pub struct TypeResolver<'a> {
    /// Shared context for diagnostics and state
    ctx: &'a mut Context,
    no: usize,
    delay: ResolveFields,
    part: Option<SourceUnitPart>,
}

impl<'a> TypeResolver<'a> {
    /// Creates a new type resolver with the given context
    pub fn new(ctx: &'a mut Context, no: usize) -> Self {
        Self { ctx, no, delay: ResolveFields::default(), part: None }
    }
}

/// Internal error type for type resolution logic
#[derive(Debug, Error)]
pub enum TypeResolverError {}

impl<'a> SemanticVisitor for TypeResolver<'a> {
    fn visit_sema_source_unit_part(
        &mut self,
        part: &mut SourceUnitPart,
    ) -> Result<(), Self::Error> {
        self.part = Some(part.clone());
        part.part.visit(self)?;

        Ok(())
    }

    fn visit_sema_contract(
        &mut self,
        _contract: &mut ContractDefinition,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a> Visitor for TypeResolver<'a> {
    type Error = TypeResolverError;

    fn visit_enum(&mut self, _def: &mut pt::EnumDefinition) -> Result<(), Self::Error> {
        self.ctx.reject(&self.part.as_ref().unwrap().annotations, "enum");
        Ok(())
    }

    fn visit_struct(&mut self, def: &mut pt::StructDefinition) -> Result<(), Self::Error> {
        self.ctx.reject(&self.part.as_ref().unwrap().annotations, "struct");

        let struct_no = self.ctx.structs.len();

        if self.ctx.add_symbol(
            self.no,
            None,
            def.name.as_ref().unwrap(),
            Symbol::Struct(def.name.as_ref().unwrap().loc, StructType::UserDefined(struct_no)),
        ) {
            self.ctx.structs.push(StructDecl {
                tags: Vec::new(),
                id: def.name.clone().unwrap(),
                loc: def.name.as_ref().unwrap().loc,
                contract: None,
                fields: Vec::new(),
                offsets: Vec::new(),
                storage_offsets: Vec::new(),
            });

            self.delay.structs.push(ResolveStructFields {
                struct_no,
                pt: def.clone(),
                contract: None,
            });
        }

        Ok(())
    }

    fn visit_event(&mut self, def: &mut pt::EventDefinition) -> Result<(), Self::Error> {
        self.ctx.reject(&self.part.as_ref().unwrap().annotations, "event");

        let event_no = self.ctx.events.len();

        if let Some(Symbol::Event(events)) = self.ctx.variable_symbols.get_mut(&(
            self.no,
            None,
            def.name.as_ref().unwrap().name.to_owned(),
        )) {
            events.push((def.name.as_ref().unwrap().loc, event_no));
        } else if !self.ctx.add_symbol(
            self.no,
            None,
            def.name.as_ref().unwrap(),
            Symbol::Event(vec![(def.name.as_ref().unwrap().loc, event_no)]),
        ) {
            return Ok(());
        }

        self.ctx.events.push(EventDecl {
            tags: Vec::new(),
            id: def.name.as_ref().unwrap().to_owned(),
            loc: def.loc,
            contract: None,
            fields: Vec::new(),
            anonymous: def.anonymous,
            signature: String::new(),
            used: false,
        });

        self.delay.events.push(ResolveEventFields { event_no, pt: def.clone() });

        Ok(())
    }

    fn visit_error(&mut self, def: &mut pt::ErrorDefinition) -> Result<(), Self::Error> {
        match &def.keyword {
            pt::Expression::Variable(id) if id.name == "error" => (),
            _ => {
                // This can be:
                //
                // int[2] var(bool);
                // S var2();
                // function var3(int x);
                // Event var4(bool f1);
                // Error var4(bool f1);
                // Feh.b1 var5();
                self.ctx.diagnostics.push(Diagnostic::error(
                    def.keyword.loc(),
                    "'function', 'error', or 'event' expected",
                ));
                return Ok(());
            }
        }

        self.ctx.reject(&self.part.as_ref().unwrap().annotations, "error");

        let error_no = self.ctx.errors.len();

        if !self.ctx.add_symbol(
            self.no,
            None,
            def.name.as_ref().unwrap(),
            Symbol::Error(def.name.as_ref().unwrap().loc, error_no),
        ) {
            return Ok(());
        }

        self.ctx.errors.push(ErrorDecl {
            tags: Vec::new(),
            name: def.name.as_ref().unwrap().name.to_owned(),
            loc: def.name.as_ref().unwrap().loc,
            contract: None,
            fields: Vec::new(),
            used: false,
        });

        self.delay.errors.push(ResolveErrorFields { error_no, pt: def.clone() });

        Ok(())
    }

    fn visit_type_definition(&mut self, ty: &mut pt::TypeDefinition) -> Result<(), Self::Error> {
        self.ctx.reject(&self.part.as_ref().unwrap().annotations, "type");
        type_decl(ty, self.no, None, self.ctx);

        Ok(())
    }
}

fn type_decl(
    _def: &pt::TypeDefinition,
    _no: usize,
    _contract_no: Option<usize>,
    _ctx: &mut Context,
) {
    todo!()
}

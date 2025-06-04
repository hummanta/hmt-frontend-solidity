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

use indexmap::IndexMap;
use std::fmt::Write;
use thiserror::Error;

use crate::{
    diagnostics::{Diagnostic, Level},
    helpers::CodeLocation,
    parser::{
        ast as pt,
        visitor::{Visitable, Visitor},
    },
    semantic::ast::{ArrayLength, Mapping},
};

use super::{
    ast::{
        ContractDefinition, EnumDecl, ErrorDecl, EventDecl, SourceUnitPart, StructDecl, StructType,
        Symbol, Type,
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

impl Type {
    pub fn to_string(&self, ctx: &Context) -> String {
        match self {
            Type::Bool => "bool".to_string(),
            Type::Address(false) => "address".to_string(),
            Type::Address(true) => "address payable".to_string(),
            Type::Int(n) => format!("int{n}"),
            Type::Uint(n) => format!("uint{n}"),
            Type::Rational => "rational".to_string(),
            Type::Value => format!("uint{}", ctx.value_length * 8),
            Type::Bytes(n) => format!("bytes{n}"),
            Type::String => "string".to_string(),
            Type::DynamicBytes => "bytes".to_string(),
            Type::Enum(n) => format!("enum {}", ctx.enums[*n]),
            Type::Struct(str_ty) => format!("struct {}", str_ty.definition(ctx)),
            Type::Array(ty, len) => format!(
                "{}{}",
                ty.to_string(ctx),
                len.iter()
                    .map(|len| match len {
                        ArrayLength::Fixed(len) => format!("[{len}]"),
                        _ => "[]".to_string(),
                    })
                    .collect::<String>()
            ),
            Type::Mapping(Mapping { key, key_name, value, value_name }) => {
                format!(
                    "mapping({}{}{} => {}{}{})",
                    key.to_string(ctx),
                    if key_name.is_some() { " " } else { "" },
                    key_name.as_ref().map(|id| id.name.as_str()).unwrap_or(""),
                    value.to_string(ctx),
                    if value_name.is_some() { " " } else { "" },
                    value_name.as_ref().map(|id| id.name.as_str()).unwrap_or(""),
                )
            }
            Type::ExternalFunction { params, mutability, returns } |
            Type::InternalFunction { params, mutability, returns } => {
                let mut s = format!(
                    "function({}) {}",
                    params.iter().map(|ty| ty.to_string(ctx)).collect::<Vec<String>>().join(","),
                    if matches!(self, Type::InternalFunction { .. }) {
                        "internal"
                    } else {
                        "external"
                    }
                );

                if !mutability.is_default() {
                    write!(s, " {mutability}").unwrap();
                }

                if !returns.is_empty() {
                    write!(
                        s,
                        " returns ({})",
                        returns
                            .iter()
                            .map(|ty| ty.to_string(ctx))
                            .collect::<Vec<String>>()
                            .join(",")
                    )
                    .unwrap();
                }

                s
            }
            Type::Contract(n) => format!("contract {}", ctx.contracts[*n].id),
            Type::UserType(n) => format!("usertype {}", ctx.user_types[*n]),
            Type::Ref(r) => r.to_string(ctx),
            Type::StorageRef(_, ty) => {
                format!("{} storage", ty.to_string(ctx))
            }
            Type::Void => "void".into(),
            Type::Unreachable => "unreachable".into(),
            // A slice of bytes1 is like bytes
            Type::Slice(ty) if **ty == Type::Bytes(1) => "bytes".into(),
            Type::Slice(ty) => format!("{}[]", ty.to_string(ctx)),
            Type::Unresolved => "unresolved".into(),
            Type::BufferPointer => "buffer_pointer".into(),
            Type::FunctionSelector => "function_selector".into(),
        }
    }
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

    /// Parse enum declaration. If the declaration is invalid, it is still generated
    /// so that we can continue parsing, with errors recorded.
    fn enum_decl(&mut self, def: &pt::EnumDefinition, contract_no: Option<usize>) -> bool {
        let mut valid = true;

        if def.values.is_empty() {
            self.ctx.diagnostics.push(Diagnostic::error(
                def.name.as_ref().unwrap().loc,
                format!("enum '{}' has no fields", def.name.as_ref().unwrap().name),
            ));
            valid = false;
        } else if def.values.len() > 256 {
            self.ctx.diagnostics.push(Diagnostic::error(
                def.name.as_ref().unwrap().loc,
                format!(
                    "enum '{}' has {} fields, which is more than the 256 limit",
                    def.name.as_ref().unwrap().name,
                    def.values.len()
                ),
            ));
            valid = false;
        }

        // check for duplicates
        let mut entries: IndexMap<String, pt::Loc> = IndexMap::new();

        for e in def.values.iter() {
            if let Some(prev) = entries.get(&e.as_ref().unwrap().name.to_string()) {
                self.ctx.diagnostics.push(
                    Diagnostic::builder(e.as_ref().unwrap().loc, Level::Error)
                        .message(format!("duplicate enum value {}", e.as_ref().unwrap().name))
                        .note(*prev, "location of previous definition")
                        .build(),
                );
                valid = false;
                continue;
            }

            entries.insert(e.as_ref().unwrap().name.to_string(), e.as_ref().unwrap().loc);
        }

        let decl = EnumDecl {
            id: def.name.clone().unwrap(),
            loc: def.loc,
            contract: match contract_no {
                Some(c) => Some(self.ctx.contracts[c].id.name.to_owned()),
                None => None,
            },
            ty: Type::Uint(8),
            values: entries,
        };

        let pos = self.ctx.enums.len();

        self.ctx.enums.push(decl);

        if !self.ctx.add_symbol(
            self.no,
            contract_no,
            def.name.as_ref().unwrap(),
            Symbol::Enum(def.name.as_ref().unwrap().loc, pos),
        ) {
            valid = false;
        }

        valid
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

    fn visit_enum(&mut self, def: &mut pt::EnumDefinition) -> Result<(), Self::Error> {
        self.ctx.reject(&self.part.as_ref().unwrap().annotations, "enum");
        let _ = self.enum_decl(def, None);

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

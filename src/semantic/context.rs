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

use std::collections::HashMap;

use anyhow::Result;

use crate::{
    diagnostics::{Diagnostic, Diagnostics, ErrorType, Level},
    parser::ast as pt,
};

use super::{ast::*, file::File};

/// Provides context information for the `resolve_type` function.
#[derive(PartialEq, Eq)]
#[allow(dead_code)]
pub(super) enum ResolveTypeContext {
    None,
    Casting,
    FunctionType,
}

/// Holds all the resolved symbols and types.
#[derive(Debug)]
pub struct Context {
    pub pragmas: Vec<Pragma>,
    pub files: Vec<File>,
    pub enums: Vec<EnumDecl>,
    pub structs: Vec<StructDecl>,
    pub events: Vec<EventDecl>,
    pub errors: Vec<ErrorDecl>,
    pub contracts: Vec<Contract>,
    /// Global using declarations
    pub using: Vec<Using>,
    /// All type declarations
    pub user_types: Vec<UserTypeDecl>,
    /// All functions
    pub functions: Vec<Function>,
    /// Yul functions
    // pub yul_functions: Vec<YulFunction>,
    /// Global constants
    pub constants: Vec<Variable>,
    /// address length in bytes
    pub address_length: usize,
    /// value length in bytes
    pub value_length: usize,
    pub diagnostics: Diagnostics,
    /// There is a separate namespace for functions and non-functions
    pub function_symbols: HashMap<(usize, Option<usize>, String), Symbol>,
    /// Symbol key is file_no, contract, identifier
    pub variable_symbols: HashMap<(usize, Option<usize>, String), Symbol>,
    // each variable in the symbol table should have a unique number
    pub next_id: usize,
    /// For a variable reference at a location, give the constant value
    /// This for use by the language server to show the value of a variable at a location
    // pub var_constants: HashMap<pt::Loc, codegen::Expression>,
    /// Overrides for hover in the language server
    pub hover_overrides: HashMap<pt::Loc, String>,
}

impl Context {
    /// Add symbol to symbol table.
    /// either returns true for success, or adds an appropriate error
    pub fn add_symbol(
        &mut self,
        _no: usize,
        _contract_no: Option<usize>,
        _id: &pt::Identifier,
        _symbol: Symbol,
    ) -> bool {
        todo!()
    }

    pub fn wrong_symbol(symbol: Option<&Symbol>, id: &pt::Identifier) -> Diagnostic {
        match symbol {
            None => Diagnostic::builder(id.loc, Level::Error)
                .ty(ErrorType::DeclarationError)
                .message(format!("'{}' not found", id.name))
                .build(),
            Some(Symbol::Enum(..)) => Diagnostic::builder(id.loc, Level::Error)
                .ty(ErrorType::DeclarationError)
                .message(format!("'{}' is an enum", id.name))
                .build(),
            Some(Symbol::Struct(..)) => Diagnostic::builder(id.loc, Level::Error)
                .ty(ErrorType::DeclarationError)
                .message(format!("'{}' is a struct", id.name))
                .build(),
            Some(Symbol::Event(_)) => Diagnostic::builder(id.loc, Level::Error)
                .ty(ErrorType::DeclarationError)
                .message(format!("'{}' is an event", id.name))
                .build(),
            Some(Symbol::Error(..)) => Diagnostic::builder(id.loc, Level::Error)
                .ty(ErrorType::DeclarationError)
                .message(format!("'{}' is an error", id.name))
                .build(),
            Some(Symbol::Function(_)) => Diagnostic::builder(id.loc, Level::Error)
                .ty(ErrorType::DeclarationError)
                .message(format!("'{}' is a function", id.name))
                .build(),
            Some(Symbol::Contract(..)) => Diagnostic::builder(id.loc, Level::Error)
                .ty(ErrorType::DeclarationError)
                .message(format!("'{}' is a contract", id.name))
                .build(),
            Some(Symbol::Import(..)) => Diagnostic::builder(id.loc, Level::Error)
                .ty(ErrorType::DeclarationError)
                .message(format!("'{}' is an import", id.name))
                .build(),
            Some(Symbol::UserType(..)) => Diagnostic::builder(id.loc, Level::Error)
                .ty(ErrorType::DeclarationError)
                .message(format!("'{}' is an user type", id.name))
                .build(),
            Some(Symbol::Variable(..)) => Diagnostic::builder(id.loc, Level::Error)
                .ty(ErrorType::DeclarationError)
                .message(format!("'{}' is a contract variable", id.name))
                .build(),
        }
    }

    /// If an item does not allow annotations, then generate diagnostic errors.
    pub(crate) fn reject(&mut self, annotations: &[pt::Annotation], item: &str) {
        for note in annotations {
            let msg = format!("annotations not allowed on {item}");
            self.diagnostics.push(Diagnostic::error(note.loc, msg))
        }
    }

    /// Resolve a contract name with namespace
    pub(super) fn resolve_contract_with_namespace(
        &mut self,
        no: usize,
        name: &pt::IdentifierPath,
        diagnostics: &mut Diagnostics,
    ) -> Result<usize, ()> {
        let (id, namespace) = name
            .identifiers
            .split_last()
            .map(|(id, namespace)| (id, namespace.iter().collect()))
            .unwrap();

        let s = self.resolve_namespace(namespace, no, None, id, diagnostics)?;

        if let Some(Symbol::Contract(_, contract_no)) = s {
            Ok(*contract_no)
        } else {
            diagnostics.push(Context::wrong_symbol(s, id));
            Err(())
        }
    }

    /// Resolve a free function name with namespace
    pub(super) fn resolve_function_with_namespace(
        &self,
        file_no: usize,
        contract_no: Option<usize>,
        name: &pt::IdentifierPath,
        diagnostics: &mut Diagnostics,
    ) -> Result<Vec<(pt::Loc, usize)>, ()> {
        let (id, namespace) = name
            .identifiers
            .split_last()
            .map(|(id, namespace)| (id, namespace.iter().collect()))
            .unwrap();

        let symbol = self.resolve_namespace(namespace, file_no, contract_no, id, diagnostics)?;

        if let Some(Symbol::Function(list)) = symbol {
            Ok(list.clone())
        } else {
            diagnostics.push(Context::wrong_symbol(symbol, id));
            Err(())
        }
    }

    /// Resolve the type name with the namespace to a symbol
    fn resolve_namespace(
        &self,
        mut _namespace: Vec<&pt::Identifier>,
        _no: usize,
        mut _contract_no: Option<usize>,
        _id: &pt::Identifier,
        _diagnostics: &mut Diagnostics,
    ) -> Result<Option<&Symbol>, ()> {
        todo!()
    }

    /// Resolve the parsed data type. The type can be a primitive, enum and also an arrays.
    /// The type for address payable is "address payable" used as a type, and "payable" when
    /// casting. So, we need to know what we are resolving for.
    pub(super) fn resolve_type(
        &mut self,
        _file_no: usize,
        _contract_no: Option<usize>,
        _resolve_context: ResolveTypeContext,
        _id: &pt::Expression,
        _diagnostics: &mut Diagnostics,
    ) -> Result<Type, ()> {
        todo!()
    }

    /// base contracts in depth-first post-order
    pub fn contract_bases(&self, contract_no: usize) -> Vec<usize> {
        let mut order = Vec::new();

        fn base(contract_no: usize, order: &mut Vec<usize>, ctx: &Context) {
            for b in ctx.contracts[contract_no].bases.iter().rev() {
                base(b.contract_no, order, ctx);
            }

            if !order.contains(&contract_no) {
                order.push(contract_no);
            }
        }

        base(contract_no, &mut order, self);

        order
    }
}

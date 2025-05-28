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

use super::{
    ast::{Contract, Pragma, Symbol},
    file::File,
};

/// Holds all the resolved symbols and types.
pub struct Context {
    pub pragmas: Vec<Pragma>,
    pub files: Vec<File>,
    pub contracts: Vec<Contract>,
    pub diagnostics: Diagnostics,
    /// There is a separate namespace for functions and non-functions
    pub function_symbols: HashMap<(usize, Option<usize>, String), Symbol>,
    /// Symbol key is file number, contract, identifier
    pub variable_symbols: HashMap<(usize, Option<usize>, String), Symbol>,
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
}

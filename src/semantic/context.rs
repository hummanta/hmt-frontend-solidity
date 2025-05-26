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

use crate::{
    ast as pt,
    diagnostics::{Diagnostic, Diagnostics},
};

use super::{
    ast::{Pragma, Symbol},
    file::File,
};

/// Holds all the resolved symbols and types.
pub struct Context {
    pub pragmas: Vec<Pragma>,
    pub files: Vec<File>,
    pub contracts: Vec<()>,
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
        &self,
        _no: usize,
        _contract_no: Option<usize>,
        _id: &pt::Identifier,
        _symbol: Symbol,
    ) -> bool {
        todo!()
    }

    /// If an item does not allow annotations, then generate diagnostic errors.
    pub(crate) fn reject(&mut self, annotations: &[pt::Annotation], item: &str) {
        for note in annotations {
            let msg = format!("annotations not allowed on {item}");
            self.diagnostics.push(Diagnostic::error(note.loc, msg))
        }
    }
}

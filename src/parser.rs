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

use std::iter::once;

use lalrpop_util::lalrpop_mod;

use crate::{ast::SourceUnit, diagnostics::Diagnostic, lexer::Lexer};

lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::type_complexity)]
    #[allow(clippy::large_enum_variant)]
    grammar
);

pub use grammar::*;

/// Parses source into SourceUnit or returns syntax errors
pub fn parse(source: &str, no: usize) -> Result<SourceUnit, Vec<Diagnostic>> {
    let lexer = Lexer::new(source);
    let parser = grammar::SourceUnitParser::new();
    let mut errors = Vec::new(); // Collected during parse

    parser.parse(source, no, &mut errors, lexer).map_err(|err| {
        errors
            .into_iter()
            .map(|err| Diagnostic::from((&err.error, no)))
            .chain(once(Diagnostic::from((&err, no))))
            .collect()
    })
}

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

pub mod analyzer;
pub mod ast;
pub mod collector;
pub mod context;
pub mod contract;
pub mod expression;
pub mod file;
pub mod function;
pub mod import;
pub mod pragma;
pub mod semicolon;
pub mod symtable;
pub mod tag;
pub mod types;
pub mod using;
pub mod variable;
pub mod visitor;

use self::context::Context;
use crate::resolver::{FileResolver, ResolvedFile};
use anyhow::Result;

/// Analyzes the semantic of the given source code.
pub fn analyze(file: &ResolvedFile, resolver: &mut FileResolver, ctx: &mut Context) -> Result<()> {
    analyzer::analyze(file, resolver, ctx)?;

    if !ctx.diagnostics.any_errors() {
        // Checks for unused variables
        // Checks for unused events
        // Checks for unused errors
    }

    Ok(())
}

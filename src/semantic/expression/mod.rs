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
    diagnostics::Diagnostics,
    parser::ast as pt,
    semantic::{
        ast::{Expression, Type},
        context::Context,
        symtable::{LoopScopes, Symtable, VarScope},
    },
};

pub mod constructor;
pub mod resolve_expression;
pub mod retrieve_type;
pub mod strings;

/// When resolving an expression, what type are we looking for
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ResolveTo<'a> {
    Unknown,        // We don't know what we're looking for, best effort
    Integer,        // Try to resolve to an integer type value (signed or unsigned, any bit width)
    Discard,        // We won't be using the result. For example, an expression as a statement
    Type(&'a Type), // We will be wanting this type please, e.g. `int64 x = 1;`
}

#[derive(Default)]
pub struct ExprContext {
    /// What source file are we in
    pub no: usize,
    // Are we resolving a contract, and if so, which one
    pub contract_no: Option<usize>,
    /// Are resolving the body of a function, and if so, which one
    pub function_no: Option<usize>,
    /// Are we currently in an unchecked block
    pub unchecked: bool,
    /// Are we evaluating a constant expression
    pub constant: bool,
    /// Are we resolving an l-value
    pub lvalue: bool,
    /// Are we resolving a yul function (it cannot have external dependencies)
    pub yul_function: bool,
    /// Loops nesting
    pub loops: LoopScopes,
    /// Stack of currently active variable scopes
    pub active_scopes: Vec<VarScope>,
    /// Solidity v0.5 and earlier don't complain about emit resolving to multiple events
    pub ambiguous_emit: bool,
}

impl ExprContext {
    pub fn enter_scope(&mut self) {
        self.active_scopes.push(VarScope { loc: None, names: HashMap::new() });
    }

    pub fn leave_scope(&mut self, symtable: &mut Symtable, loc: pt::Loc) {
        if let Some(mut curr_scope) = self.active_scopes.pop() {
            curr_scope.loc = Some(loc);
            symtable.scopes.push(curr_scope);
        }
    }
}

impl Expression {
    /// Cast from one type to another, which also automatically derefs any Type::Ref() type.
    /// if the cast is explicit (e.g. bytes32(bar) then implicit should be set to false.
    pub(crate) fn cast(
        &self,
        _loc: &pt::Loc,
        _to: &Type,
        _implicit: bool,
        _ctx: &Context,
        _diagnostics: &mut Diagnostics,
    ) -> Result<Expression, ()> {
        todo!()
    }
}

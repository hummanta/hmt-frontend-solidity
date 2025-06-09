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

use crate::{
    diagnostics::Diagnostics,
    parser::ast as pt,
    semantic::{
        ast::Expression,
        context::Context,
        expression::{ExprContext, ResolveTo},
        symtable::Symtable,
    },
};

#[allow(unused_variables)]
#[allow(clippy::result_unit_err)]
/// Resolve a parsed expression into an AST expression.
/// The resolve_to argument is a hint to what type the result should be.
pub fn expression(
    expr: &pt::Expression,
    context: &mut ExprContext,
    ctx: &mut Context,
    symtable: &mut Symtable,
    diagnostics: &mut Diagnostics,
    resolve_to: ResolveTo,
) -> Result<Expression, ()> {
    todo!()
}

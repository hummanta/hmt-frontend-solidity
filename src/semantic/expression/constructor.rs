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
    semantic::{ast::Expression, context::Context, expression::ExprContext, symtable::Symtable},
};

/// Try and find constructor for arguments
#[allow(clippy::result_unit_err)]
pub fn match_constructor_to_args(
    _loc: &pt::Loc,
    _args: &[pt::Expression],
    _contract_no: usize,
    _context: &mut ExprContext,
    _ctx: &mut Context,
    _symtable: &mut Symtable,
    _diagnostics: &mut Diagnostics,
) -> Result<(Option<usize>, Vec<Expression>), ()> {
    todo!()
}

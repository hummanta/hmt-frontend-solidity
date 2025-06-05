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
    parser::ast as pt,
    semantic::{ast::ContractDefinition, context::Context},
};

#[allow(dead_code)]
pub struct DelayedResolveInitializer {
    var_no: usize,
    contract_no: usize,
    initializer: pt::Expression,
}

pub fn contract_variables(
    _def: &ContractDefinition,
    _no: usize,
    _ctx: &mut Context,
) -> Vec<DelayedResolveInitializer> {
    todo!()
}

pub fn resolve_initializers(
    _initializers: &[DelayedResolveInitializer],
    _no: usize,
    _ctx: &mut Context,
) {
    todo!()
}

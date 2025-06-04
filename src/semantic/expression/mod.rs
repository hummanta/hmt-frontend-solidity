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
        ast::{Expression, Type},
        context::Context,
    },
};

pub(crate) mod strings;

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

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

use crate::semantic::{
    ast::{Parameter, Tag, Type},
    context::Context,
};

/// Resolve the tags for a type from parsed doccomment
#[allow(unused_variables)]
pub fn resolve_tags(
    file_no: usize,
    ty: &str,
    params: Option<&[Parameter<Type>]>,
    returns: Option<&[Parameter<Type>]>,
    bases: Option<Vec<usize>>,
    ctx: &mut Context,
) -> Vec<Tag> {
    todo!()
}

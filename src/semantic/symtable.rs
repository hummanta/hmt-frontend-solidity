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

use indexmap::IndexMap;

use crate::{parser::ast as pt, semantic::ast::Variable};

#[derive(Debug, Clone)]
pub struct VarScope {
    pub loc: Option<pt::Loc>,
    pub names: HashMap<String, usize>,
}

#[derive(Default, Debug, Clone)]
pub struct Symtable {
    pub vars: IndexMap<usize, Variable>,
    pub arguments: Vec<Option<usize>>,
    pub returns: Vec<usize>,
    pub scopes: Vec<VarScope>,
}

pub struct LoopScope {
    pub no_breaks: usize,
    pub no_continues: usize,
}

#[allow(dead_code)]
pub struct LoopScopes(Vec<LoopScope>);

impl Default for LoopScopes {
    fn default() -> Self {
        LoopScopes::new()
    }
}

impl LoopScopes {
    pub fn new() -> Self {
        LoopScopes(Vec::new())
    }
}

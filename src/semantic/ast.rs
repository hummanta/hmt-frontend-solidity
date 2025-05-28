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

use std::{
    cell::OnceCell,
    collections::{BTreeMap, HashMap},
    fmt::{self, Write as _},
};

use crate::parser::ast as pt;

pub struct SourceUnit {
    pub parts: Vec<SourceUnitPart>,
    pub contracts: Vec<ContractDefinition>,
}

pub struct SourceUnitPart {
    pub annotations: Vec<pt::Annotation>,
    pub part: pt::SourceUnitPart,
}

pub struct ContractPart {
    pub annotations: Vec<pt::Annotation>,
    pub part: pt::ContractPart,
}

pub struct ContractDefinition {
    pub contract_no: usize,
    pub loc: pt::Loc,
    pub ty: pt::ContractTy,
    pub annotations: Vec<pt::Annotation>,
    pub name: Option<pt::Identifier>,
    pub base: Vec<pt::Base>,
    pub parts: Vec<ContractPart>,
}

#[derive(Debug)]
pub enum Pragma {
    Identifier { loc: pt::Loc, name: pt::Identifier, value: pt::Identifier },
    StringLiteral { loc: pt::Loc, name: pt::Identifier, value: pt::StringLiteral },
    SolidityVersion { loc: pt::Loc, versions: Vec<VersionReq> },
}

#[derive(Debug)]
pub enum VersionReq {
    Plain { loc: pt::Loc, version: Version },
    Operator { loc: pt::Loc, op: pt::VersionOp, version: Version },
    Range { loc: pt::Loc, from: Version, to: Version },
    Or { loc: pt::Loc, left: Box<VersionReq>, right: Box<VersionReq> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    pub major: u32,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.major.fmt(f)?;
        if let Some(minor) = self.minor {
            f.write_char('.')?;
            minor.fmt(f)?
        }
        if let Some(patch) = self.patch {
            f.write_char('.')?;
            patch.fmt(f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Contract {
    // pub tags: Vec<Tag>,
    pub loc: pt::Loc,
    pub ty: pt::ContractTy,
    pub id: pt::Identifier,
    pub bases: Vec<Base>,
    // pub using: Vec<Using>,
    // pub layout: Vec<Layout>,
    // pub fixed_layout_size: BigInt,
    pub functions: Vec<usize>,
    pub all_functions: BTreeMap<usize, usize>,
    /// maps the name of virtual functions to a vector of overriden functions.
    /// Each time a virtual function is overriden, there will be an entry pushed to the vector. The
    /// last element represents the current overriding function - there will be at least one
    /// entry in this vector.
    pub virtual_functions: HashMap<String, Vec<usize>>,
    pub yul_functions: Vec<usize>,
    // pub variables: Vec<Variable>,
    /// List of contracts this contract instantiates
    pub creates: Vec<usize>,
    /// List of events this contract may emit
    pub emits_events: Vec<usize>,
    pub initializer: Option<usize>,
    // pub default_constructor: Option<(Function, usize)>,
    // pub cfg: Vec<ControlFlowGraph>,
    /// Compiled program. Only available after emit.
    pub code: OnceCell<Vec<u8>>,
    /// Can the contract be instantiated, i.e. not abstract, no errors, etc.
    pub instantiable: bool,
}

impl Contract {
    // Is this a concrete contract, which can be instantiated
    pub fn is_concrete(&self) -> bool {
        matches!(self.ty, pt::ContractTy::Contract(_))
    }

    // Is this an interface
    pub fn is_interface(&self) -> bool {
        matches!(self.ty, pt::ContractTy::Interface(_))
    }

    // Is this an library
    pub fn is_library(&self) -> bool {
        matches!(self.ty, pt::ContractTy::Library(_))
    }
}

#[derive(Debug)]
pub struct Base {
    pub loc: pt::Loc,
    pub contract_no: usize,
    pub constructor: Option<(usize, Vec<Expression>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    Enum(pt::Loc, usize),
    Function(Vec<(pt::Loc, usize)>),
    Variable(pt::Loc, Option<usize>, usize),
    // Struct(pt::Loc, StructType),
    Event(Vec<(pt::Loc, usize)>),
    Error(pt::Loc, usize),
    Contract(pt::Loc, usize),
    Import(pt::Loc, usize),
    UserType(pt::Loc, usize),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Expression {}

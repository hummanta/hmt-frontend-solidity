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

use std::fmt::{self, Write as _};

use crate::ast as pt;

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

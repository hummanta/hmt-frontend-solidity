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

#[derive(Clone, Debug)]
pub struct Program(pub Vec<SourceUnit>);

impl Program {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_program(self)
    }

    pub fn iter(&self) -> impl Iterator<Item = &SourceUnit> {
        self.0.iter()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SourceUnit {
    /// A pragma directive.
    PragmaDirective(Box<PragmaDirective>),

    /// A contract definition.
    ContractDefinition(Box<ContractDefinition>),
}

impl SourceUnit {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_source_unit(self)
    }
}

/// A pragma directive
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PragmaDirective {
    /// pragma version =0.5.16;
    Version(Identifier, Vec<VersionComparator>),
}

impl PragmaDirective {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_pragma(self)
    }
}

/// A `version` list
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VersionComparator {
    /// 0.8.22
    Plain {
        /// List of versions: major, minor, patch. minor and patch are optional
        version: String,
    },
    /// =0.5.16
    Operator {
        /// Semver comparison operator
        op: VersionOp,
        /// version number
        version: String,
    },
    /// foo || bar
    Or {
        /// left part
        left: Box<VersionComparator>,
        /// right part
        right: Box<VersionComparator>,
    },
    /// 0.7.0 - 0.8.22
    Range {
        /// start of range
        from: String,
        /// end of range
        to: String,
    },
}

/// Comparison operator
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum VersionOp {
    /// `=`
    Exact,
    /// `>`
    Greater,
    /// `>=`
    GreaterEq,
    /// `<`
    Less,
    /// `<=`
    LessEq,
    /// `~`
    Tilde,
    /// `^`
    Caret,
    /// `*`
    Wildcard,
}

/// A contract definition.
///
/// `<ty> <name> { <parts>,* }`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ContractDefinition {
    /// The contract type.
    pub ty: ContractTy,
    /// The identifier.
    pub name: Identifier,
    /// The list of contract parts.
    pub parts: Vec<ContractPart>,
}

impl ContractDefinition {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_contract(self)
    }
}

/// The contract type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ContractTy {
    /// `contract`
    Contract,
}

/// A contract part.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ContractPart {}

/// An identifier.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Identifier {
    /// The identifier string.
    pub name: String,
}

pub trait Visitor<T> {
    fn visit_program(&mut self, program: &Program) -> T;
    fn visit_source_unit(&mut self, source_unit: &SourceUnit) -> T;
    fn visit_pragma(&mut self, pragma: &PragmaDirective) -> T;
    fn visit_contract(&mut self, contract: &ContractDefinition) -> T;
}

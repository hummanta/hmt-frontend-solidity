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

/// An expression.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    /// `<1>++`
    PostIncrement(Box<Expression>),
    /// `<1>--`
    PostDecrement(Box<Expression>),
    /// `new <1>`
    New(Box<Expression>),
    /// `<1>\[ [2] \]`
    ArraySubscript(Box<Expression>, Option<Box<Expression>>),
    /// `<1>\[ [2] : [3] \]`
    ArraySlice(Box<Expression>, Option<Box<Expression>>, Option<Box<Expression>>),
    /// `(<1>)`
    Parenthesis(Box<Expression>),
    /// `<1>.<2>`
    MemberAccess(Box<Expression>, Identifier),
    /// `!<1>`
    Not(Box<Expression>),
    /// `~<1>`
    BitwiseNot(Box<Expression>),
    /// `delete <1>`
    Delete(Box<Expression>),
    /// `++<1>`
    PreIncrement(Box<Expression>),
    /// `--<1>`
    PreDecrement(Box<Expression>),
    /// `+<1>`
    ///
    /// Note that this isn't actually supported by Solidity.
    UnaryPlus(Box<Expression>),
    /// `-<1>`
    Negate(Box<Expression>),

    /// `<1> ** <2>`
    Power(Box<Expression>, Box<Expression>),
    /// `<1> * <2>`
    Multiply(Box<Expression>, Box<Expression>),
    /// `<1> / <2>`
    Divide(Box<Expression>, Box<Expression>),
    /// `<1> % <2>`
    Modulo(Box<Expression>, Box<Expression>),
    /// `<1> + <2>`
    Add(Box<Expression>, Box<Expression>),
    /// `<1> - <2>`
    Subtract(Box<Expression>, Box<Expression>),
    /// `<1> << <2>`
    ShiftLeft(Box<Expression>, Box<Expression>),
    /// `<1> >> <2>`
    ShiftRight(Box<Expression>, Box<Expression>),
    /// `<1> & <2>`
    BitwiseAnd(Box<Expression>, Box<Expression>),
    /// `<1> ^ <2>`
    BitwiseXor(Box<Expression>, Box<Expression>),
    /// `<1> | <2>`
    BitwiseOr(Box<Expression>, Box<Expression>),
    /// `<1> < <2>`
    Less(Box<Expression>, Box<Expression>),
    /// `<1> > <2>`
    More(Box<Expression>, Box<Expression>),
    /// `<1> <= <2>`
    LessEqual(Box<Expression>, Box<Expression>),
    /// `<1> >= <2>`
    MoreEqual(Box<Expression>, Box<Expression>),
    /// `<1> == <2>`
    Equal(Box<Expression>, Box<Expression>),
    /// `<1> != <2>`
    NotEqual(Box<Expression>, Box<Expression>),
    /// `<1> && <2>`
    And(Box<Expression>, Box<Expression>),
    /// `<1> || <2>`
    Or(Box<Expression>, Box<Expression>),
    /// `<1> ? <2> : <3>`
    ///
    /// AKA ternary operator.
    ConditionalOperator(Box<Expression>, Box<Expression>, Box<Expression>),
    /// `<1> = <2>`
    Assign(Box<Expression>, Box<Expression>),
    /// `<1> |= <2>`
    AssignOr(Box<Expression>, Box<Expression>),
    /// `<1> &= <2>`
    AssignAnd(Box<Expression>, Box<Expression>),
    /// `<1> ^= <2>`
    AssignXor(Box<Expression>, Box<Expression>),
    /// `<1> <<= <2>`
    AssignShiftLeft(Box<Expression>, Box<Expression>),
    /// `<1> >>= <2>`
    AssignShiftRight(Box<Expression>, Box<Expression>),
    /// `<1> += <2>`
    AssignAdd(Box<Expression>, Box<Expression>),
    /// `<1> -= <2>`
    AssignSubtract(Box<Expression>, Box<Expression>),
    /// `<1> *= <2>`
    AssignMultiply(Box<Expression>, Box<Expression>),
    /// `<1> /= <2>`
    AssignDivide(Box<Expression>, Box<Expression>),
    /// `<1> %= <2>`
    AssignModulo(Box<Expression>, Box<Expression>),

    /// `true` or `false`
    BoolLiteral(bool),
    /// ``
    NumberLiteral(String, Option<Identifier>),
    /// ``
    RationalNumberLiteral(String, Option<Identifier>),
    /// ``
    HexNumberLiteral(String, Option<Identifier>),
    /// `<1>+`. See [StringLiteral].
    StringLiteral(Vec<StringLiteral>),
    /// `<1>+`. See [HexLiteral].
    HexLiteral(Vec<HexLiteral>),
    /// `0x[a-fA-F0-9]{40}`
    ///
    /// This [should be correctly checksummed][ref],
    /// but it currently isn't being enforced in the parser.
    ///
    /// [ref]: https://docs.soliditylang.org/en/latest/types.html#address-literals
    AddressLiteral(String),
    /// Any valid [Identifier].
    Variable(Identifier),
    /// `\[ <1>.* \]`
    ArrayLiteral(Vec<Expression>),
}

impl Expression {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_expression(self)
    }
}

/// A string literal.
///
/// `[unicode]"<string>"`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StringLiteral {
    /// Whether this is a unicode string.
    pub unicode: bool,
    /// The string literal.
    ///
    /// Does not contain the quotes or the `unicode` prefix.
    pub string: String,
}

/// A hex literal.
///
/// `hex"<literal>"`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HexLiteral {
    /// The hex literal.
    ///
    /// Contains the `hex` prefix.
    pub hex: String,
}

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
    fn visit_expression(&mut self, exp: &Expression) -> T;
}

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

    /// A variable definition.
    VariableDefinition(Box<VariableDefinition>),

    /// A function definition.
    FunctionDefinition(Box<FunctionDefinition>),

    /// A stray semicolon.
    StraySemicolon,
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
pub enum ContractPart {
    /// A variable definition.
    VariableDefinition(Box<VariableDefinition>),

    /// A function definition.
    FunctionDefinition(Box<FunctionDefinition>),

    /// A stray semicolon.
    StraySemicolon,
}

impl ContractPart {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_contract_part(self)
    }
}

/// A variable definition.
///
/// `<ty> <attrs>* <name> [= <initializer>]`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VariableDefinition {
    /// The type.
    pub ty: Expression,
    /// The list of variable attributes.
    pub attrs: Vec<VariableAttribute>,
    /// The identifier.
    ///
    /// This field is `None` only if an error occurred during parsing.
    pub name: Option<Identifier>,
    /// The optional initializer.
    pub initializer: Option<Expression>,
}

impl VariableDefinition {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_variable(self)
    }
}

/// A variable attribute.
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u8)] // for cmp; order of variants is important
pub enum VariableAttribute {
    /// The visibility.
    ///
    /// Only used for storage variables.
    Visibility(Visibility),

    /// `constant`
    Constant,

    /// `immutable`
    Immutable,

    /// `ovveride(<1>,*)`
    Override(Vec<IdentifierPath>),

    /// Storage type.
    StorageType(StorageType),
}

/// Function visibility.
///
/// Deprecated for [FunctionTy] other than `Function`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u8)] // for cmp; order of variants is important
pub enum Visibility {
    /// `external`
    External,

    /// `public`
    Public,

    /// `internal`
    Internal,

    /// `private`
    Private,
}

/// Soroban storage types.
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u8)] // for cmp; order of variants is important
pub enum StorageType {
    /// `Temporary`
    Temporary,

    /// `persistent`
    Persistent,

    /// `Instance`
    Instance,
}

/// A function definition.
///
/// `<ty> [name](<params>,*) [attributes] [returns] [body]`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionDefinition {
    /// The function type.
    pub ty: FunctionTy,
    /// The optional identifier.
    ///
    /// This can be `None` for old style fallback functions.
    pub name: Option<Identifier>,
    /// The parameter list.
    pub params: ParameterList,
    /// The function attributes.
    pub attributes: Vec<FunctionAttribute>,
    /// The return parameter list.
    pub returns: ParameterList,
    /// The function body.
    ///
    /// If `None`, the declaration ended with a semicolon.
    pub body: Option<Statement>,
}

impl FunctionDefinition {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_function(self)
    }
}

/// A function's type.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FunctionTy {
    /// `constructor`
    Constructor,

    /// `function`
    Function,

    /// `fallback`
    Fallback,

    /// `receive`
    Receive,

    /// `modifier`
    Modifier,
}

/// Type alias for a list of function parameters.
pub type ParameterList = Vec<Option<Parameter>>;

/// A parameter.
///
/// `<ty> [storage] <name>`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parameter {
    /// An optional annotation '@annotation'.
    pub annotation: Option<Annotation>,
    /// The type.
    pub ty: Expression,
    /// The optional memory location.
    pub storage: Option<StorageLocation>,
    /// The optional identifier.
    pub name: Option<Identifier>,
}

impl Parameter {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_parameter(self)
    }
}

/// An annotation.
///
/// `@<id>(<value>)`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Annotation {
    /// The identifier.
    pub id: Identifier,
    /// The value.
    pub value: Option<Expression>,
}

impl Annotation {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_annotation(self)
    }
}

/// Dynamic type location.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StorageLocation {
    /// `memory`
    Memory,

    /// `storage`
    Storage,

    /// `calldata`
    Calldata,
}

/// A function attribute.
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u8)] // for cmp; order of variants is important
pub enum FunctionAttribute {
    /// Visibility attribute.
    Visibility(Visibility),

    /// Mutability attribute.
    Mutability(Mutability),

    /// `virtual`
    Virtual,

    /// `immutable`
    Immutable,

    /// `override[(<identifier path>,*)]`
    Override(Vec<IdentifierPath>),

    /// A modifier or constructor invocation.
    BaseOrModifier(Base),

    /// An error occurred during parsing.
    Error,
}

/// Function mutability.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Mutability {
    /// `pure`
    Pure,

    /// `view`
    View,

    /// `constant`
    Constant,

    /// `payable`
    Payable,
}

/// A function modifier invocation (see [FunctionAttribute])
/// or a contract inheritance specifier (see [ContractDefinition]).
///
/// Both have the same semantics:
///
/// `<name>[(<args>,*)]`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Base {
    /// The identifier path.
    pub name: IdentifierPath,
    /// The optional arguments.
    pub args: Option<Vec<Expression>>,
}

impl Base {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_base(self)
    }
}

/// A named argument.
///
/// `<name>: <expr>`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NamedArgument {
    /// The identifier.
    pub name: Identifier,
    /// The value.
    pub expr: Expression,
}

impl NamedArgument {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_named_argument(self)
    }
}

/// A statement.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement {}

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

/// A qualified identifier.
///
/// `<identifiers>.*`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IdentifierPath {
    /// The list of identifiers.
    pub identifiers: Vec<Identifier>,
}

pub trait Visitor<T> {
    fn visit_program(&mut self, program: &Program) -> T;
    fn visit_source_unit(&mut self, source_unit: &SourceUnit) -> T;
    fn visit_pragma(&mut self, pragma: &PragmaDirective) -> T;
    fn visit_contract(&mut self, contract: &ContractDefinition) -> T;
    fn visit_contract_part(&mut self, part: &ContractPart) -> T;
    fn visit_variable(&mut self, var: &VariableDefinition) -> T;
    fn visit_function(&mut self, func: &FunctionDefinition) -> T;
    fn visit_parameter(&mut self, param: &Parameter) -> T;
    fn visit_annotation(&mut self, ano: &Annotation) -> T;
    fn visit_base(&mut self, base: &Base) -> T;
    fn visit_named_argument(&mut self, arg: &NamedArgument) -> T;
    fn visit_expression(&mut self, exp: &Expression) -> T;
}

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

use std::fmt;

use logos::Logos;

use crate::error::LexicalError;

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+", skip r"//.*\n?", error = LexicalError)]
pub enum Token {
    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[regex("@[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice().to_string())]
    Annotation(String),

    #[regex(r#"(unicode)?"[_a-zA-Z][_0-9a-zA-Z]*""#, |lex| lex.slice().to_string())]
    StringLiteral(String),

    #[regex(r#"hex["']([0-9a-fA-F]{2}(_?[0-9a-fA-F]{2})*)*["']"#, |lex| lex.slice().to_string())]
    HexLiteral(String),

    #[regex("0x[0-9a-fA-F]{40}", |lex| lex.slice().to_string())]
    AddressLiteral(String),

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
    Number(f64),

    RationalNumber(String),

    #[regex(r"0x([0-9a-fA-F]{2}(_?[0-9a-fA-F]{2})*)*", |lex| lex.slice().to_string())]
    HexNumber(String),

    #[token(";")]
    Semicolon,

    #[token("{")]
    OpenCurlyBrace,

    #[token("}")]
    CloseCurlyBrace,

    #[token("(")]
    OpenParenthesis,

    #[token(")")]
    CloseParenthesis,

    #[token("=")]
    Assign,

    #[token("==")]
    Equal,

    #[token("=>")]
    Arrow,

    #[token("->")]
    YulArrow,

    #[token("|=")]
    BitwiseOrAssign,

    #[token("^=")]
    BitwiseXorAssign,

    #[token("&=")]
    BitwiseAndAssign,

    #[token("<<=")]
    ShiftLeftAssign,

    #[token(">>=")]
    ShiftRightAssign,

    #[token("+=")]
    AddAssign,

    #[token("-=")]
    SubtractAssign,

    #[token("*=")]
    MulAssign,

    #[token("/=")]
    DivideAssign,

    #[token("%=")]
    ModuloAssign,

    #[token("?")]
    Question,

    #[token(":")]
    Colon,

    #[token(":=")]
    ColonAssign,

    #[token("||")]
    Or,

    #[token("&&")]
    And,

    #[token("!=")]
    NotEqual,

    #[token("<")]
    Less,

    #[token("<=")]
    LessEqual,

    #[token(">")]
    More,

    #[token(">=")]
    MoreEqual,

    #[token("|")]
    BitwiseOr,

    #[token("&")]
    BitwiseAnd,

    #[token("^")]
    BitwiseXor,

    #[token("<<")]
    ShiftLeft,

    #[token(">>")]
    ShiftRight,

    #[token("+")]
    Add,

    #[token("-")]
    Subtract,

    #[token("*")]
    Mul,

    #[token("/")]
    Divide,

    #[token("%")]
    Modulo,

    #[token("**")]
    Power,

    #[token("!")]
    Not,

    #[token("~")]
    BitwiseNot,

    #[token("++")]
    Increment,

    #[token("--")]
    Decrement,

    #[token("[")]
    OpenBracket,

    #[token("]")]
    CloseBracket,

    #[token(".")]
    Member,

    #[token(",")]
    Comma,

    Uint(u16),
    Int(u16),
    Bytes(u8),

    #[token("byte")]
    Byte,

    #[token("struct")]
    Struct,

    #[token("memory")]
    Memory,

    #[token("calldata")]
    Calldata,

    #[token("storage")]
    Storage,

    #[token("import")]
    Import,

    #[token("contract")]
    Contract,

    #[token("pragma")]
    Pragma,

    #[token("bool")]
    Bool,

    #[token("address")]
    Address,

    #[token("string")]
    String,

    #[token("bytes")]
    DynamicBytes,

    #[token("delete")]
    Delete,

    #[token("new")]
    New,

    #[token("interface")]
    Interface,

    #[token("library")]
    Library,

    #[token("event")]
    Event,

    #[token("enum")]
    Enum,

    #[token("type")]
    Type,

    #[token("public")]
    Public,

    #[token("private")]
    Private,

    #[token("external")]
    External,

    #[token("internal")]
    Internal,

    #[token("constant")]
    Constant,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("pure")]
    Pure,

    #[token("view")]
    View,

    #[token("payable")]
    Payable,

    #[token("constructor")]
    Constructor,

    #[token("function")]
    Function,

    #[token("returns")]
    Returns,

    #[token("return")]
    Return,

    #[token("revert")]
    Revert,

    #[token("if")]
    If,

    #[token("for")]
    For,

    #[token("while")]
    While,

    #[token("else")]
    Else,

    #[token("do")]
    Do,

    #[token("continue")]
    Continue,

    #[token("break")]
    Break,

    #[token("throw")]
    Throw,

    #[token("emit")]
    Emit,

    #[token("anonymous")]
    Anonymous,

    #[token("indexed")]
    Indexed,

    #[token("mapping")]
    Mapping,

    #[token("try")]
    Try,

    #[token("catch")]
    Catch,

    #[token("receive")]
    Receive,

    #[token("fallback")]
    Fallback,

    #[token("as")]
    As,

    #[token("is")]
    Is,

    #[token("abstract")]
    Abstract,

    #[token("virtual")]
    Virtual,

    #[token("override")]
    Override,

    #[token("using")]
    Using,

    #[token("modifier")]
    Modifier,

    #[token("immutable")]
    Immutable,

    #[token("unchecked")]
    Unchecked,

    #[token("assembly")]
    Assembly,

    #[token("let")]
    Let,

    #[token("leave")]
    Leave,

    #[token("switch")]
    Switch,

    #[token("case")]
    Case,

    #[token("default")]
    Default,

    #[token("persistent")]
    Persistent,

    #[token("temporary")]
    Temporary,

    #[token("instance")]
    Instance,

    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

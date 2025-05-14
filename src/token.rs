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

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
    Number(f64),

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

    #[token("||")]
    Or,

    #[token("<")]
    Less,

    #[token("<=")]
    LessEqual,

    #[token(">")]
    More,

    #[token(">=")]
    MoreEqual,

    #[token("^")]
    BitwiseXor,

    #[token("-")]
    Subtract,

    #[token("*")]
    Mul,

    #[token("~")]
    BitwiseNot,

    #[token("pragma")]
    Pragma,

    #[token("contract")]
    Contract,

    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

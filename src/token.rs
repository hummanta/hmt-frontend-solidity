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
#[logos(skip r"[ \t\n\f]+", skip r"#.*\n?", error = LexicalError)]
pub enum Token {
    // Keywords
    #[token("var")]
    Var,

    #[token("print")]
    Print,

    // Identifier
    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Literals
    #[regex("[1-9][0-9]*", |lex| lex.slice().parse())]
    Integer(i64),

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("=")]
    Assign,

    #[token(";")]
    Semicolon,

    // Operators
    #[token("+")]
    Add,

    #[token("-")]
    Sub,

    #[token("*")]
    Mul,

    #[token("/")]
    Div,

    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod test {
    use logos::Logos;

    use super::Token;

    #[test]
    fn test_var() {
        let mut lexer = Token::lexer("var x = 42;");

        assert_eq!(lexer.next(), Some(Ok(Token::Var)));

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier(String::from("x")))));
        assert_eq!(lexer.next(), Some(Ok(Token::Assign)));
        assert_eq!(lexer.next(), Some(Ok(Token::Integer(42))));
        assert_eq!(lexer.next(), Some(Ok(Token::Semicolon)));
    }

    #[test]
    fn test_print() {
        let mut lexer = Token::lexer("print(1 + 2);");

        assert_eq!(lexer.next(), Some(Ok(Token::Print)));
        assert_eq!(lexer.next(), Some(Ok(Token::LParen)));
        assert_eq!(lexer.next(), Some(Ok(Token::Integer(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::Add)));
        assert_eq!(lexer.next(), Some(Ok(Token::Integer(2))));
        assert_eq!(lexer.next(), Some(Ok(Token::RParen)));
        assert_eq!(lexer.next(), Some(Ok(Token::Semicolon)));
    }
}

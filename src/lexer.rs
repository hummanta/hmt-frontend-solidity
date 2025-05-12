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

use logos::{Logos, SpannedIter};

use crate::{error::LexicalError, token::Token};

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'source> {
    tokens: SpannedIter<'source, Token>,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self { tokens: Token::lexer(source).spanned() }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.next().map(|(token, span)| match token {
            Ok(token) => Ok((span.start, token, span.end)),
            Err(_) => Ok((span.start, Token::Error, span.end)),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::Lexer, token::Token};

    #[test]
    fn test_lex_var() {
        let mut lexer = Lexer::new("var x = 42;");

        assert_eq!(lexer.next(), Some(Ok((0, Token::Var, 3))));
        assert_eq!(lexer.next(), Some(Ok((4, Token::Identifier(String::from("x")), 5))));
        assert_eq!(lexer.next(), Some(Ok((6, Token::Assign, 7))));
        assert_eq!(lexer.next(), Some(Ok((8, Token::Integer(42), 10))));
        assert_eq!(lexer.next(), Some(Ok((10, Token::Semicolon, 11))));
    }
}

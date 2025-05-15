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

pub struct Lexer<'input> {
    tokens: SpannedIter<'input, Token<'input>>,
}

impl<'input> Lexer<'input> {
    pub fn new(source: &'input str) -> Self {
        Self { tokens: Token::lexer(source).spanned() }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token<'input>, usize, LexicalError>;

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
    fn test_lex_pragma() {
        let mut lexer = Lexer::new("pragma solidity ^0.8;");

        assert_eq!(lexer.next(), Some(Ok((0, Token::Pragma, 6))));
        assert_eq!(lexer.next(), Some(Ok((7, Token::Identifier("solidity"), 15))));
        assert_eq!(lexer.next(), Some(Ok((16, Token::BitwiseXor, 17))));
        assert_eq!(lexer.next(), Some(Ok((17, Token::Number("0.8"), 20))));
        assert_eq!(lexer.next(), Some(Ok((20, Token::Semicolon, 21))));
    }
}

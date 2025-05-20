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

use thiserror::Error;

use crate::ast::Loc;

/// An error thrown by [Lexer].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LexicalError {
    #[error("end of file found in comment")]
    EndOfFileInComment(Loc),

    #[error("end of file found in string literal")]
    EndOfFileInString(Loc),

    #[error("end of file found in hex literal string")]
    EndofFileInHex(Loc),

    #[error("missing number")]
    MissingNumber(Loc),

    #[error("invalid character '{1}' in hex literal string")]
    InvalidCharacterInHexLiteral(Loc, char),

    #[error("unrecognised token '{1}'")]
    UnrecognisedToken(Loc, String),

    #[error("missing exponent")]
    MissingExponent(Loc),

    #[error("'{1}' found where 'from' expected")]
    ExpectedFrom(Loc, String),

    #[error("invalid token")]
    InvalidToken,
}

impl Default for LexicalError {
    fn default() -> Self {
        Self::InvalidToken
    }
}

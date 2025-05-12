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

use std::{
    fmt::{self, Display},
    num::ParseIntError,
};

use ariadne::{Report, ReportKind, Source};
use lalrpop_util::{ErrorRecovery, ParseError as LalrpopParseError};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    #[default]
    InvalidToken,
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidInteger(err) => write!(f, "Invalid integer: {}", err),
            Self::InvalidToken => write!(f, "Invalid token"),
        }
    }
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub note: Option<String>,
}

impl ParseError {
    pub fn new<T: ToString>(message: T) -> Self {
        Self { message: message.to_string(), note: None }
    }

    pub fn note<T: ToString>(mut self, note: T) -> Self {
        self.note.replace(note.to_string());
        self
    }

    pub fn report(&self, source: &str) -> std::io::Result<String> {
        let mut report = Report::build(ReportKind::Error, 0..0).with_message(&self.message);

        if let Some(note) = &self.note {
            report = report.with_note(note);
        }

        let mut bytes = Vec::new();
        report.finish().write(Source::from(source), &mut bytes)?;

        let string = unsafe {
            // SAFETY: We known that the bytes are valid UTF-8
            String::from_utf8_unchecked(bytes)
        };

        Ok(string)
    }
}

impl<L: Display, T: Display, E: ToString> From<ErrorRecovery<L, T, E>> for ParseError {
    #[inline]
    fn from(value: ErrorRecovery<L, T, E>) -> Self {
        value.error.into()
    }
}

impl<L: Display, T: Display, E: ToString> From<LalrpopParseError<L, T, E>> for ParseError {
    fn from(error: LalrpopParseError<L, T, E>) -> Self {
        match error {
            LalrpopParseError::InvalidToken { location: _ } => Self::new("invalid token"),
            LalrpopParseError::UnrecognizedToken { token: (_l, token, _r), expected } => Self::new(
                format!("unrecognised token '{}', expected {}", token, expected.join(", ")),
            ),
            LalrpopParseError::User { error } => Self::new(error.to_string()),
            LalrpopParseError::ExtraToken { token } => {
                Self::new(format!("extra token '{}' encountered", token.0))
            }
            LalrpopParseError::UnrecognizedEof { expected, location: _ } => {
                Self::new(format!("unexpected end of file, expecting {}", expected.join(", ")))
            }
        }
    }
}

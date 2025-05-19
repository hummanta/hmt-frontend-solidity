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

//! Solidity parser diagnostics.

use strum::{AsRefStr, Display, EnumString};

use crate::ast::Loc;

/// The level of a diagnostic.
#[derive(Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq, EnumString, AsRefStr, Display)]
pub enum Level {
    /// Debug diagnostic level.
    #[strum(serialize = "debug")]
    Debug,
    /// Info diagnostic level.
    #[strum(serialize = "info")]
    Info,
    /// Warning diagnostic level.
    #[strum(serialize = "warning")]
    Warning,
    /// Error diagnostic level.
    #[strum(serialize = "error")]
    Error,
}

/// The type of a diagnostic.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorType {
    /// No specific error type.
    None,
    /// Parser error.
    ParserError,
    /// Syntax error.
    SyntaxError,
    /// Declaration error.
    DeclarationError,
    /// Cast error.
    CastError,
    /// Type error.
    TypeError,
    /// Warning.
    Warning,
}

/// A diagnostic note.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Note {
    /// The code location of the note.
    pub loc: Loc,
    /// The message of the note.
    pub message: String,
}

/// A Solidity diagnostic.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Diagnostic {
    /// The code location of the diagnostic.
    pub loc: Loc,
    /// The level of the diagnostic.
    pub level: Level,
    /// The type of diagnostic.
    pub ty: ErrorType,
    /// The message of the diagnostic.
    pub message: String,
    /// Extra notes about the diagnostic.
    pub notes: Vec<Note>,
}

impl Diagnostic {
    /// Create a new builder for Diagnostic.
    pub fn builder(loc: Loc, level: Level) -> DiagnosticBuilder {
        DiagnosticBuilder::new(loc, level)
    }

    #[inline]
    /// Instantiate a new Diagnostic with the given location and message at the debug level.
    pub fn debug(loc: Loc, msg: impl Into<String>) -> Self {
        DiagnosticBuilder::new(loc, Level::Debug).message(msg).build()
    }

    #[inline]
    /// Instantiate a new Diagnostic with the given location and message at the info level.
    pub fn info(loc: Loc, msg: impl Into<String>) -> Self {
        DiagnosticBuilder::new(loc, Level::Info).message(msg).build()
    }

    #[inline]
    /// Instantiate a new warning Diagnostic.
    pub fn warning(loc: Loc, msg: impl Into<String>) -> Self {
        DiagnosticBuilder::new(loc, Level::Warning).ty(ErrorType::Warning).message(msg).build()
    }

    #[inline]
    /// Instantiate a new syntax error Diagnostic.
    pub fn error(loc: Loc, msg: impl Into<String>) -> Self {
        DiagnosticBuilder::new(loc, Level::Error).ty(ErrorType::SyntaxError).message(msg).build()
    }
}

/// A builder for `Diagnostic`.
pub struct DiagnosticBuilder {
    loc: Loc,
    level: Level,
    ty: ErrorType,
    message: String,
    notes: Vec<Note>,
}

impl DiagnosticBuilder {
    /// Create a new DiagnosticBuilder.
    pub fn new(loc: Loc, level: Level) -> Self {
        Self { loc, level, ty: ErrorType::None, message: String::new(), notes: Vec::new() }
    }

    /// Set the error type
    pub fn ty(mut self, ty: ErrorType) -> Self {
        self.ty = ty;
        self
    }

    /// Set the message
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }

    /// Add a single note.
    pub fn note(mut self, loc: Loc, msg: impl Into<String>) -> Self {
        self.notes.push(Note { loc, message: msg.into() });
        self
    }

    /// Add multiple notes.
    pub fn notes(mut self, notes: Vec<Note>) -> Self {
        self.notes = notes;
        self
    }

    /// Finalize and create the `Diagnostic`.
    pub fn build(self) -> Diagnostic {
        Diagnostic {
            loc: self.loc,
            level: self.level,
            ty: self.ty,
            message: self.message,
            notes: self.notes,
        }
    }
}

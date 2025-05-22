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

use std::{
    ops::Range,
    slice::{Iter, IterMut},
};

use ariadne::{Cache, Label, Report, ReportKind, Span};
use itertools::Itertools;
use lalrpop_util::ParseError;
use strum::{AsRefStr, Display, EnumString};

use crate::{ast::Loc, error::LexicalError, helpers::CodeLocation, token::Token};

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
    #[inline]
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

/// Convert lalrop parser error to a Diagnostic
impl<'input> From<(&ParseError<usize, Token<'input>, LexicalError>, usize)> for Diagnostic {
    fn from((error, no): (&ParseError<usize, Token<'input>, LexicalError>, usize)) -> Self {
        match error {
            ParseError::InvalidToken { location } => {
                Diagnostic::builder(Loc::File(no, *location, *location), Level::Error)
                    .ty(ErrorType::ParserError)
                    .message("invalid token")
                    .build()
            }
            ParseError::UnrecognizedToken { token: (l, token, r), expected } => {
                Diagnostic::builder(Loc::File(no, *l, *r), Level::Error)
                    .ty(ErrorType::ParserError)
                    .message(format!(
                        "unrecognised token '{}', expected {}",
                        token,
                        expected.join(", ")
                    ))
                    .build()
            }
            ParseError::User { error } => Diagnostic::builder(error.loc(), Level::Error)
                .ty(ErrorType::ParserError)
                .message(error.to_string())
                .build(),
            ParseError::ExtraToken { token } => {
                Diagnostic::builder(Loc::File(no, token.0, token.2), Level::Error)
                    .ty(ErrorType::ParserError)
                    .message(format!("extra token '{}' encountered", token.0))
                    .build()
            }
            ParseError::UnrecognizedEof { expected, location } => {
                Diagnostic::builder(Loc::File(no, *location, *location), Level::Error)
                    .ty(ErrorType::ParserError)
                    .message(format!("unexpected end of file, expecting {}", expected.join(", ")))
                    .build()
            }
        }
    }
}

/// Convert Diagnostic to ariadne::Report
impl<'a> From<&Diagnostic> for Report<'a, Range<usize>> {
    fn from(val: &Diagnostic) -> Self {
        // Initialize report builder with level and location
        let mut report = Report::build(
            match val.level {
                Level::Debug => ReportKind::Advice,
                Level::Info => ReportKind::Advice,
                Level::Warning => ReportKind::Warning,
                Level::Error => ReportKind::Error,
            },
            val.loc.range(),
        )
        .with_message(&val.message);

        // Initialize labels vector
        let mut labels = Vec::new();
        for note in &val.notes {
            labels.push(Label::new(note.loc.range()).with_message(&note.message));
        }
        report = report.with_labels(labels);

        // Finish building report
        report.finish()
    }
}

/// Extension trait for writing ariadne reports to strings.
pub trait ReportToStringExt<'a, S: Span> {
    /// Write the report to a string.
    fn write_to_string<C: Cache<S::SourceId>>(
        &self,
        cache: C,
    ) -> Result<String, Box<dyn std::error::Error>>;
}

impl<'a, S: Span> ReportToStringExt<'a, S> for Report<'a, S> {
    fn write_to_string<C: Cache<S::SourceId>>(
        &self,
        cache: C,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut vec = Vec::new();
        self.write(cache, &mut vec)?;
        Ok(String::from_utf8(vec)?)
    }
}

/// A collection of diagnostics with error tracking.
///
/// Maintains a list of diagnostics and tracks whether any errors are present.
/// Provides methods for adding diagnostics and checking error status.
#[derive(Default, Debug)]
pub struct Diagnostics {
    contents: Vec<Diagnostic>,
    has_error: bool,
}

impl Diagnostics {
    /// Creates a new empty Diagnostics collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if there are any diagnostics in the collection.
    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    /// Returns the number of diagnostics in the collection.
    pub fn len(&self) -> usize {
        self.contents.len()
    }

    /// Returns an iterator over the diagnostics.
    pub fn iter(&self) -> Iter<Diagnostic> {
        self.contents.iter()
    }

    /// Returns a mutable iterator over the diagnostics.
    pub fn iter_mut(&mut self) -> IterMut<Diagnostic> {
        self.contents.iter_mut()
    }

    /// Adds a diagnostic to the collection.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        if matches!(diagnostic.level, Level::Error) {
            self.has_error = true;
        }
        self.contents.push(diagnostic)
    }

    /// Extends the collection with another Diagnostics.
    pub fn extend(&mut self, diagnostics: Diagnostics) {
        self.has_error |= diagnostics.has_error;
        self.contents.extend(diagnostics.contents);
    }

    /// Appends diagnostics from a vector into this collection.
    pub fn append(&mut self, diagnostics: &mut Vec<Diagnostic>) {
        if !self.has_error {
            self.has_error = diagnostics.iter().any(|m| m.level == Level::Error);
        }
        self.contents.append(diagnostics);
    }

    /// Checks if there are any error-level diagnostics in the collection.
    pub fn any_errors(&self) -> bool {
        self.has_error
    }

    /// Returns all error-level diagnostics in the collection.
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.contents.iter().filter(|x| x.level == Level::Error).collect()
    }

    /// Returns the message of the first error-level diagnostic.
    pub fn first_error(&self) -> String {
        self.contents.iter().find_or_first(|&x| x.level == Level::Error).unwrap().message.to_owned()
    }

    /// Returns all warning-level diagnostics in the collection.
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.contents.iter().filter(|x| x.level == Level::Warning).collect()
    }

    /// Returns the first warning-level diagnostic.
    pub fn frist_warning(&self) -> &Diagnostic {
        self.contents.iter().find_or_first(|&x| x.level == Level::Warning).unwrap()
    }

    /// Returns the count of warning-level diagnostics.
    pub fn count_warnings(&self) -> usize {
        self.contents.iter().filter(|&x| x.level == Level::Warning).count()
    }

    /// Checks if any warning-level diagnostic contains the given message.
    pub fn warning_contains(&self, message: &str) -> bool {
        self.warnings().iter().any(|x| x.message == message)
    }

    /// Checks if any diagnostic contains the given message.
    pub fn contains_message(&self, message: &str) -> bool {
        self.contents.iter().any(|x| x.message == message)
    }

    /// Sorts and deduplicates diagnostics, ensuring they're in order by location.
    pub fn normalize(&mut self) {
        self.contents.sort();
        self.contents.dedup();
    }
}

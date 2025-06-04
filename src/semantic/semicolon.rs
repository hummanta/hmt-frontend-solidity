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

use super::context::Context;

use crate::{
    diagnostics::Diagnostic,
    parser::{ast as pt, visitor::Visitor},
};

/// Check for stray semicolons
pub struct StraySemicolonChecker<'a> {
    /// Shared context for diagnostics and state
    ctx: &'a mut Context,
}

impl<'a> StraySemicolonChecker<'a> {
    /// Creates a new stray semicolons checker with the given context
    pub fn new(ctx: &'a mut Context) -> Self {
        Self { ctx }
    }
}

/// Internal error type for stray semicolons check logic
#[derive(Debug, Error)]
pub enum StraySemicolonCheckerError {}

impl<'a> Visitor for StraySemicolonChecker<'a> {
    type Error = StraySemicolonCheckerError;

    fn visit_source_unit(&mut self, source_unit: &mut pt::SourceUnit) -> Result<(), Self::Error> {
        for part in &source_unit.0 {
            match part {
                pt::SourceUnitPart::StraySemicolon(loc) => {
                    self.ctx
                        .diagnostics
                        .push(Diagnostic::error(*loc, "stray semicolon".to_string()));
                }
                pt::SourceUnitPart::ContractDefinition(contract) => {
                    for part in &contract.parts {
                        if let pt::ContractPart::StraySemicolon(loc) = part {
                            self.ctx
                                .diagnostics
                                .push(Diagnostic::error(*loc, "stray semicolon".to_string()));
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }
}

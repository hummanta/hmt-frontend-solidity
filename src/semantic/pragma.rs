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

use super::{
    context::Context,
    visitor::{SemanticVisitable, SemanticVisitor},
};

use super::ast;
use crate::{ast as pt, diagnostics::Diagnostic};

/// Resolve pragma from the parse tree
pub struct PragmaResolver<'a> {
    /// Shared compiler context for diagnostics and state
    ctx: &'a mut Context,
}

impl<'a> PragmaResolver<'a> {
    /// Creates a new pragma resolver with the given compiler context
    pub fn new(ctx: &'a mut Context) -> Self {
        Self { ctx }
    }

    /// Processes a plain pragma directive (identifier with value)
    fn plain(&mut self, loc: &pt::Loc, name: &str, value: &str) {
        if name == "experimental" && value == "ABIEncoderV2" {
            self.ctx.diagnostics.push(Diagnostic::debug(
                *loc,
                "pragma 'experimental' with value 'ABIEncoderV2' is ignored",
            ));
        } else if name == "experimental" && value == "solidity" {
            self.ctx
                .diagnostics
                .push(Diagnostic::error(*loc, "experimental solidity features are not supported"));
        } else if name == "abicoder" && (value == "v1" || value == "v2") {
            self.ctx.diagnostics.push(Diagnostic::debug(*loc, "pragma 'abicoder' ignored"));
        } else {
            self.ctx.diagnostics.push(Diagnostic::error(
                *loc,
                format!("unknown pragma '{}' with value '{}'", name, value),
            ));
        }
    }

    /// Parses a version comparator from the parse tree into an AST version requirement
    fn parse_version_comparator(
        &mut self,
        version: &pt::VersionComparator,
    ) -> Result<ast::VersionReq, PragmaResolverError> {
        match version {
            pt::VersionComparator::Plain { loc, version } => {
                Ok(ast::VersionReq::Plain { loc: *loc, version: self.parse_version(loc, version)? })
            }
            pt::VersionComparator::Operator { loc, op, version } => Ok(ast::VersionReq::Operator {
                loc: *loc,
                op: *op,
                version: self.parse_version(loc, version)?,
            }),
            pt::VersionComparator::Range { loc, from, to } => Ok(ast::VersionReq::Range {
                loc: *loc,
                from: self.parse_version(loc, from)?,
                to: self.parse_version(loc, to)?,
            }),
            pt::VersionComparator::Or { loc, left, right } => Ok(ast::VersionReq::Or {
                loc: *loc,
                left: self.parse_version_comparator(left)?.into(),
                right: self.parse_version_comparator(right)?.into(),
            }),
        }
    }

    /// Parses a version string into an `ast::Version`
    fn parse_version(
        &mut self,
        loc: &pt::Loc,
        version: &[String],
    ) -> Result<ast::Version, PragmaResolverError> {
        let mut res = Vec::with_capacity(3);

        for v in version {
            if let Ok(v) = v.parse() {
                res.push(v);
            } else {
                self.ctx
                    .diagnostics
                    .push(Diagnostic::error(*loc, format!("'{v}' is not a valid number")));
                return Err(PragmaResolverError::InvalidVersionComponent);
            }
        }

        if version.len() > 3 {
            self.ctx.diagnostics.push(Diagnostic::error(
                *loc,
                "no more than three numbers allowed - major.minor.patch",
            ));
            return Err(PragmaResolverError::TooManyVersionComponents);
        }

        Ok(ast::Version { major: res[0], minor: res.get(1).cloned(), patch: res.get(2).cloned() })
    }
}

/// Internal error type for pragma resolution logic
#[derive(Debug, Error)]
pub enum PragmaResolverError {
    #[error("Invalid version component")]
    InvalidVersionComponent,
    #[error("Too many version components")]
    TooManyVersionComponents,
}

impl<'a> SemanticVisitor for PragmaResolver<'a> {
    type Error = PragmaResolverError;

    /// Visits a source unit and processes any pragma directives found,
    /// and rejects any annotations on pragma directives.
    fn visit_source_unit(&mut self, source_unit: &mut ast::SourceUnit) -> Result<(), Self::Error> {
        for part in source_unit.parts.iter_mut() {
            if matches!(part.part, pt::SourceUnitPart::PragmaDirective(_)) {
                self.ctx.reject(&part.annotations, "pragma");
                part.visit(self)?;
            }
        }

        Ok(())
    }

    /// Visits a pragma directive and processes it according to its type:
    /// - For identifier pragmas: Validates known pragma names/values
    /// - For string literal pragmas: Processes as plain pragmas
    /// - For version pragmas: Parses and validates Solidity version requirements
    fn visit_pragma(&mut self, pragma: &pt::PragmaDirective) -> Result<(), Self::Error> {
        match pragma {
            pt::PragmaDirective::Identifier(loc, Some(ident), Some(value)) => {
                self.plain(loc, &ident.name, &value.name);
                self.ctx.pragmas.push(ast::Pragma::Identifier {
                    loc: *loc,
                    name: ident.clone(),
                    value: value.clone(),
                });
            }
            pt::PragmaDirective::StringLiteral(loc, ident, value) => {
                self.plain(loc, &ident.name, &value.string);
                self.ctx.pragmas.push(ast::Pragma::StringLiteral {
                    loc: *loc,
                    name: ident.clone(),
                    value: value.clone(),
                });
            }
            pt::PragmaDirective::Version(loc, ident, versions) => {
                if ident.name != "solidity" {
                    self.ctx.diagnostics.push(Diagnostic::error(
                        ident.loc,
                        format!("unknown pragma '{}'", ident.name),
                    ));
                    return Ok(());
                }

                // parser versions
                let mut res = Vec::new();

                for version in versions {
                    let Ok(v) = self.parse_version_comparator(version) else {
                        return Ok(());
                    };
                    res.push(v);
                }

                if res.len() > 1 && res.iter().any(|v| matches!(v, ast::VersionReq::Range { .. })) {
                    self.ctx.diagnostics.push(Diagnostic::error(
                        *loc,
                        "version ranges can only be combined with the || operator",
                    ));
                }

                self.ctx.pragmas.push(ast::Pragma::SolidityVersion { loc: *loc, versions: res });
            }

            // only occurs when there is a parse error, name or value is None
            pt::PragmaDirective::Identifier { .. } => (),
        }

        Ok(())
    }
}

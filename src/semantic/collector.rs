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

use std::mem;

use thiserror::Error;

use crate::{
    diagnostics::Diagnostic,
    parser::{
        ast as pt,
        visitor::{Visitable, Visitor},
    },
};

use super::{
    ast::{ContractDefinition, ContractPart, SourceUnit, SourceUnitPart},
    context::Context,
};

/// Collects annotations in Solidity source code during semantic analysis.
pub struct AnnotationCollector<'a> {
    /// Shared compiler context for diagnostics and state
    ctx: &'a mut Context,
    /// Temporary buffer for collected annotation nodes
    annotations: Vec<pt::Annotation>,
    /// Processed parts of the source unit (non-contract elements)
    parts: Vec<SourceUnitPart>,
    /// Processed contract definitions from the source
    contracts: Vec<ContractDefinition>,
    /// Sequential numbering for processed contracts
    no: usize,
}

impl<'a> AnnotationCollector<'a> {
    /// Creates a new collector with the given compiler context
    pub fn new(ctx: &'a mut Context) -> Self {
        let contract_no = ctx.contracts.len();

        Self {
            ctx,
            annotations: Vec::new(),
            parts: Vec::new(),
            contracts: Vec::new(),
            no: contract_no,
        }
    }

    /// Finalizes the collection process and returns the analyzed source unit
    pub fn collect(&mut self) -> SourceUnit {
        if !self.annotations.is_empty() {
            for note in &self.annotations {
                self.ctx.diagnostics.push(Diagnostic::error(
                    note.loc,
                    "annotations should precede 'contract' or other item",
                ));
            }
        }

        let parts = mem::take(&mut self.parts);
        let contracts = mem::take(&mut self.contracts);

        SourceUnit { parts, contracts }
    }
}

/// Placeholder error type for annotation collection (currently unused)
#[derive(Debug, Error)]
pub enum CollectorError {}

impl<'a> Visitor for AnnotationCollector<'a> {
    type Error = CollectorError;

    /// Visits and processes all parts of a source unit, handling annotations
    fn visit_source_unit(&mut self, source_unit: &mut pt::SourceUnit) -> Result<(), Self::Error> {
        for part in source_unit.0.iter_mut() {
            if let pt::SourceUnitPart::Annotation(note) = part {
                self.annotations.push(note.as_ref().clone());
                continue;
            }

            if let pt::SourceUnitPart::ContractDefinition(_) = part {
                part.visit(self)?;
                continue;
            }

            self.parts.push(SourceUnitPart {
                annotations: mem::take(&mut self.annotations),
                part: part.clone(),
            });
        }

        Ok(())
    }

    /// Visits and processes a contract definition, handling its annotations and parts
    fn visit_contract(&mut self, contract: &mut pt::ContractDefinition) -> Result<(), Self::Error> {
        let mut parts = Vec::new();
        let mut annotations = Vec::new();

        for part in contract.parts.iter_mut() {
            if let pt::ContractPart::Annotation(note) = part {
                annotations.push(note.as_ref().clone());
                continue;
            }

            parts.push(ContractPart {
                annotations: mem::take(&mut annotations),
                part: part.clone(),
            });
        }

        if !annotations.is_empty() {
            for note in &annotations {
                self.ctx.diagnostics.push(Diagnostic::error(
                    note.loc,
                    "annotations should precede 'constructor' or other item",
                ));
            }
        }

        self.contracts.push(ContractDefinition {
            contract_no: self.no,
            loc: contract.loc,
            ty: contract.ty.clone(),
            annotations: mem::take(&mut self.annotations),
            name: contract.name.clone(),
            base: contract.base.clone(),
            parts: mem::take(&mut parts),
        });

        self.no += 1;

        Ok(())
    }
}

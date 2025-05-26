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
    analyzer,
    ast::{self, Symbol},
    context::Context,
    expression::strings::unescape,
    visitor::{SemanticVisitable, SemanticVisitor},
};

use std::ffi::OsString;

use crate::{
    ast as pt,
    diagnostics::Diagnostic,
    resolver::{FileResolver, ResolvedFile},
};

/// Find import file, resolve it by calling analyze and add it to the context
pub struct ImportResolver<'a> {
    /// Shared context for diagnostics and state
    ctx: &'a mut Context,
    resolver: &'a mut FileResolver,
    parent: Option<&'a ResolvedFile>,
    filename: Option<pt::StringLiteral>,
    import_file_no: usize,
    no: usize,
}

impl<'a> ImportResolver<'a> {
    /// Creates a new import resolver with the given context
    pub fn new(
        ctx: &'a mut Context,
        resolver: &'a mut FileResolver,
        parent: Option<&'a ResolvedFile>,
        no: usize,
    ) -> Self {
        Self { ctx, resolver, parent, filename: None, import_file_no: 0, no }
    }

    /// Adds symbol to context if it doesn't already exist with the same definition
    fn add_symbol(
        &mut self,
        function: bool,
        contract_no: Option<usize>,
        name: String,
        symbol: Symbol,
    ) {
        let symbols = match function {
            true => &self.ctx.function_symbols,
            false => &self.ctx.variable_symbols,
        };

        if symbols.get(&(self.no, contract_no, name.to_owned())) != Some(&symbol) {
            let filename = self.filename.as_ref().unwrap();
            let new_symbol = pt::Identifier { name, loc: filename.loc };

            self.ctx.add_symbol(self.no, contract_no, &new_symbol, symbol);
        }
    }
}

/// Placeholder error type for import resolver (currently unused)
#[derive(Debug, Error)]
pub enum ImportResolverError {}

impl<'a> SemanticVisitor for ImportResolver<'a> {
    type Error = ImportResolverError;

    /// Visits a source unit and processes any import directives found,
    /// and rejects any annotations on import directives.
    fn visit_source_unit(&mut self, source_unit: &mut ast::SourceUnit) -> Result<(), Self::Error> {
        for part in source_unit.parts.iter_mut() {
            if matches!(part.part, pt::SourceUnitPart::ImportDirective(_)) {
                self.ctx.reject(&part.annotations, "import");
                part.visit(self)?;
            }
        }

        Ok(())
    }

    fn visit_import(&mut self, import: &mut pt::Import) -> Result<(), Self::Error> {
        let path = match import {
            pt::Import::Plain(f, _) |
            pt::Import::GlobalSymbol(f, _, _) |
            pt::Import::Rename(f, _, _) => f,
        };

        let filename = match path {
            pt::ImportPath::Filename(f) => f,
            pt::ImportPath::Path(path) => {
                self.ctx
                    .diagnostics
                    .push(Diagnostic::error(path.loc, "experimental import paths not supported"));
                return Ok(());
            }
        };

        if filename.string.is_empty() {
            self.ctx.diagnostics.push(Diagnostic::error(filename.loc, "import path empty"));
            return Ok(());
        }

        let (valid, bs) = unescape(
            &filename.string,
            filename.loc.start(),
            filename.loc.no(),
            &mut self.ctx.diagnostics,
        );

        if !valid {
            return Ok(());
        }

        let os_filename = if let Some(res) = osstring_from_vec(&filename.loc, bs, self.ctx) {
            res
        } else {
            return Ok(());
        };

        self.filename.replace(filename.clone());

        self.import_file_no = if let Some(builtin_file_no) = self
            .ctx
            .files
            .iter()
            .position(|file| file.cache_no.is_none() && file.path == os_filename)
        {
            // import "solana"
            builtin_file_no
        } else {
            match self.resolver.resolve(self.parent, &os_filename) {
                Err(message) => {
                    self.ctx.diagnostics.push(Diagnostic::error(filename.loc, message));
                    return Ok(());
                }
                Ok(file) => {
                    if !self.ctx.files.iter().any(|f| f.path == file.full_path) {
                        let _ = analyzer::analyze(&file, self.resolver, self.ctx);

                        // give up if we failed
                        if self.ctx.diagnostics.any_errors() {
                            return Ok(());
                        }
                    }

                    self.ctx
                        .files
                        .iter()
                        .position(|f| f.path == file.full_path)
                        .expect("import should be loaded by now")
                }
            }
        };

        import.visit(self)?;

        Ok(())
    }

    fn visit_import_plain(
        &mut self,
        _: pt::Loc,
        _: &mut pt::ImportPath,
    ) -> Result<(), Self::Error> {
        // Process variable symbols
        let exports: Vec<_> = self
            .ctx
            .variable_symbols
            .iter()
            .filter(|((no, _, _), _)| *no == self.import_file_no)
            .map(|((_, contract_no, name), symbol)| (*contract_no, name.clone(), symbol.clone()))
            .collect();

        for (contract_no, name, symbol) in exports {
            self.add_symbol(false, contract_no, name, symbol);
        }

        // Process function symbols
        let exports: Vec<_> = self
            .ctx
            .function_symbols
            .iter()
            .filter(|((no, contract_no, _), _)| *no == self.import_file_no && contract_no.is_none())
            .map(|((_, _, name), symbol)| (name.clone(), symbol.clone()))
            .collect();

        for (name, symbol) in exports {
            self.add_symbol(true, None, name, symbol);
        }

        Ok(())
    }

    fn visit_import_global(
        &mut self,
        _: pt::Loc,
        _: &mut pt::ImportPath,
        alias: &mut pt::Identifier,
    ) -> Result<(), Self::Error> {
        self.ctx.add_symbol(self.no, None, alias, Symbol::Import(alias.loc, self.import_file_no));
        Ok(())
    }

    fn visit_import_renames(
        &mut self,
        _: pt::Loc,
        imports: &mut [(pt::Identifier, Option<pt::Identifier>)],
        _: &mut pt::ImportPath,
    ) -> Result<(), Self::Error> {
        let mut symbols = Vec::new();

        for (from, rename_to) in imports {
            let id = rename_to.as_ref().unwrap_or(from);

            // Try variable symbols first
            if let Some(symbol) =
                self.ctx.variable_symbols.get(&(self.import_file_no, None, from.name.to_owned()))
            {
                symbols.push((false, id.clone(), symbol.clone()));
            }
            // Then try function symbols
            else if let Some(symbol) =
                self.ctx.function_symbols.get(&(self.import_file_no, None, from.name.to_owned()))
            {
                symbols.push((true, id.clone(), symbol.clone()));
            } else {
                let filename = self.filename.as_ref().unwrap();
                self.ctx.diagnostics.push(Diagnostic::error(
                    from.loc,
                    format!("import '{}' does not export '{}'", filename.string, from.name),
                ));
            }
        }

        for (function, id, symbol) in symbols {
            self.add_symbol(function, None, id.name, symbol);
        }

        Ok(())
    }
}

#[cfg(unix)]
fn osstring_from_vec(_: &pt::Loc, bs: Vec<u8>, _: &mut Context) -> Option<OsString> {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    Some(OsString::from_vec(bs))
}

#[cfg(not(unix))]
fn osstring_from_vec(loc: &pt::Loc, bs: Vec<u8>, ctx: &mut Context) -> Option<OsString> {
    match str::from_utf8(&bs) {
        Ok(s) => Some(OsString::from(s)),
        Err(_) => {
            ctx.diagnostics.push(Diagnostic::error(*loc, "string is not a valid filename"));
            None
        }
    }
}

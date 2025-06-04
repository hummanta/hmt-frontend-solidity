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

use super::{
    collector::AnnotationCollector, context::Context, contract::ContractResolver, file::File,
    import::ImportResolver, pragma::PragmaResolver, visitor::SemanticVisitable,
};

use crate::{
    parser::{parse, visitor::Visitable},
    resolver::{FileResolver, ResolvedFile},
    semantic::{types::TypeResolver, using::UsingResolver},
};

use anyhow::{bail, Result};

/// Parse and resolve a file and its imports in a recursive manner.
pub(crate) fn analyze(
    file: &ResolvedFile,
    resolver: &mut FileResolver,
    ctx: &mut Context,
) -> Result<()> {
    let no = ctx.files.len();

    let (source, cache_no) = resolver.get_file_contents_and_no(&file.full_path);
    ctx.files.push(File::new(file.full_path.clone(), &source, cache_no, file.import_no));

    let mut ast = match parse(&source, no) {
        Ok(ast) => ast,
        Err(mut errors) => {
            ctx.diagnostics.append(&mut errors);
            bail!("Parsing failed");
        }
    };

    // Walk through the parse tree and collect all the
    // anonotations for each items, also inside contracts.
    let mut collector = AnnotationCollector::new(ctx);
    ast.visit(&mut collector)?;
    let mut tree = collector.collect();

    // First resolve all the types we can find
    tree.visit(&mut TypeResolver::new(ctx, no))?;

    // Resolve pragmas and imports
    tree.visit(&mut PragmaResolver::new(ctx))?;
    tree.visit(&mut ImportResolver::new(ctx, resolver, Some(file), no))?;

    // Resolve the base contracts list and check for cycles.
    tree.visit(&mut ContractResolver::new(ctx, no))?;

    // Now we can resolve the global using directives
    tree.visit(&mut UsingResolver::new(ctx, no, None))?;

    Ok(())
}

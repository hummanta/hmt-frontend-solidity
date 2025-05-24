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

use super::{collector::AnnotationCollector, context::Context, file::File};

use crate::{
    parser::parse,
    resolver::{FileResolver, ResolvedFile},
    visitor::Visitable,
};

/// Parse and resolve a file and its imports in a recursive manner.
pub(crate) fn analyze(file: &ResolvedFile, resolver: &mut FileResolver, ctx: &mut Context) {
    let no = ctx.files.len();

    let (source, cache_no) = resolver.get_file_contents_and_no(&file.full_path);
    ctx.files.push(File::new(file.full_path.clone(), &source, cache_no, file.import_no));

    let mut ast = match parse(&source, no) {
        Ok(ast) => ast,
        Err(mut errors) => {
            ctx.diagnostics.append(&mut errors);
            return;
        }
    };

    // Walk through the parse tree and collect all the
    // anonotations for each items, also inside contracts.
    let mut collector = AnnotationCollector::new(ctx);
    if ast.visit(&mut collector).is_err() {
        return;
    }
    let _tree = collector.collect();
}

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

use std::path::PathBuf;

/// Any Solidity file, either the main file or anything that was imported
#[derive(Clone, Debug)]
pub struct File {
    /// The on-disk filename
    pub path: PathBuf,
    /// Used for offset to line-column conversions
    pub line_starts: Vec<usize>,
    /// Indicates the file number in FileResolver.files
    pub cache_no: Option<usize>,
    /// Index into FileResolver.import_paths. This is `None` when this File was
    /// created not during `parse_and_resolve` (e.g., builtins)
    pub import_no: Option<usize>,
}

impl File {
    pub fn new(path: PathBuf, contents: &str, cache_no: usize, import_no: Option<usize>) -> Self {
        let mut line_starts = Vec::new();

        for (indice, c) in contents.char_indices() {
            if c == '\n' {
                line_starts.push(indice + 1);
            }
        }

        Self { path, line_starts, cache_no: Some(cache_no), import_no }
    }
}

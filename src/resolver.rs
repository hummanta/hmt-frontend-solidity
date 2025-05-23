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
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Default)]
pub struct FileResolver {
    /// Set of import paths search for imports
    import_paths: Vec<(Option<OsString>, PathBuf)>,
    /// List file by path
    cached_paths: HashMap<PathBuf, usize>,
    /// The actual file contents
    files: Vec<ResolvedFile>,
}

/// When we resolve a file, we need to know its base compared to the import so
/// we can resolve the next import, and the full path on the filesystem.
/// Since the same filename can exists in multiple imports, we need to tell the
/// user exactly which file has errors / warnings.
#[derive(Clone, Debug)]
pub struct ResolvedFile {
    /// Original name used on cli or import statement
    pub path: OsString,
    /// Full path on the filesystem
    pub full_path: PathBuf,
    /// Which import path was used, if any
    pub import_no: Option<usize>,
    /// The actual file contents
    pub contents: Arc<str>,
}

impl FileResolver {
    /// Add import path
    pub fn add_import_path(&mut self, path: &Path) {
        assert!(!self.import_paths.contains(&(None, path.to_path_buf())));
        self.import_paths.push((None, path.to_path_buf()));
    }

    /// Add import map
    pub fn add_import_map(&mut self, map: OsString, path: PathBuf) {
        let map = Some(map);

        if let Some((_, e)) = self.import_paths.iter_mut().find(|(k, _)| *k == map) {
            *e = path;
        } else {
            self.import_paths.push((map, path));
        }
    }

    /// Get the import path and the optional mapping corresponding to `import_no`.
    pub fn get_import_path(&self, import_no: usize) -> Option<&(Option<OsString>, PathBuf)> {
        self.import_paths.get(import_no)
    }

    /// Get the import paths
    pub fn get_import_paths(&self) -> &[(Option<OsString>, PathBuf)] {
        self.import_paths.as_slice()
    }

    /// Get the import path corresponding to a map
    pub fn get_import_map(&self, map: &OsString) -> Option<&PathBuf> {
        self.import_paths.iter().find(|(m, _)| m.as_ref() == Some(map)).map(|(_, pb)| pb)
    }

    /// Update the cache for the filename with the given contents
    pub fn set_file_contents(&mut self, path: &str, contents: String) {
        let pos = self.files.len();
        let pathbuf = PathBuf::from(path);

        self.files.push(ResolvedFile {
            path: path.into(),
            full_path: pathbuf.clone(),
            import_no: None,
            contents: Arc::from(contents),
        });
        self.cached_paths.insert(pathbuf, pos);
    }

    /// Get the file contents of `no`th file if it exists
    pub fn get_contents_of_no(&self, no: usize) -> Option<Arc<str>> {
        self.files.get(no).map(|f| f.contents.clone())
    }

    /// Get file with contents. This must be a file which was previously added to the cache.
    pub fn get_file_contents_and_number(&self, file: &Path) -> (Arc<str>, usize) {
        let no = self.cached_paths[file];
        (self.files[no].contents.clone(), no)
    }
}

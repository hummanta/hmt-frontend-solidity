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

use itertools::Itertools;
use normalize_path::NormalizePath;
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs::File,
    io::Read,
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

    /// Get file with contents and number.
    /// This must be a file which was previously added to the cache.
    pub fn get_file_contents_and_no(&self, file: &Path) -> (Arc<str>, usize) {
        let no = self.cached_paths[file];
        (self.files[no].contents.clone(), no)
    }

    /// Atempt to resolve a file, either from the cache or from the filesystem.
    /// Returns Ok(Some(..)) if the file is found and loaded.
    /// Returns Ok(None) if no file by this path can be found.
    /// Returns Err(..) if a file was found but could not be read.
    fn try_file(
        &mut self,
        filename: &OsStr,
        path: &Path,
        import_no: Option<usize>,
    ) -> Result<Option<ResolvedFile>, String> {
        // For accessing the cache, remove "." and ".." path components
        let cache_path = path.normalize();

        if let Some(cache) = self.cached_paths.get(&cache_path) {
            let mut file = self.files[*cache].clone();
            file.import_no = import_no;
            return Ok(Some(file));
        }

        if let Ok(full_path) = path.canonicalize() {
            let file = self.load_file(filename, &full_path, import_no)?;
            return Ok(Some(file.clone()));
        }

        Ok(None)
    }

    /// Populate the cache with absolute file path
    fn load_file(
        &mut self,
        filename: &OsStr,
        path: &Path,
        import_no: Option<usize>,
    ) -> Result<&ResolvedFile, String> {
        let path_filename = PathBuf::from(filename);
        if let Some(cache) = self.cached_paths.get(&path_filename) {
            if self.files[*cache].import_no == import_no {
                return Ok(&self.files[*cache]);
            }
        }

        // read the file
        let mut f = match File::open(path) {
            Err(err_info) => {
                return Err(format!("cannot open file '{}': {}", path.display(), err_info));
            }
            Ok(file) => file,
        };

        let mut contents = String::new();
        if let Err(e) = f.read_to_string(&mut contents) {
            return Err(format!("failed to read file '{}': {}", path.display(), e));
        }

        let pos = self.files.len();
        self.files.push(ResolvedFile {
            path: filename.into(),
            full_path: path.to_path_buf(),
            import_no,
            contents: Arc::from(contents),
        });
        self.cached_paths.insert(path.to_path_buf(), pos);

        Ok(&self.files[pos])
    }

    /// Walk the import path to search for a file. If no import path is set up,
    /// return, Check each import path if the file can be found in a subdirectory
    /// of that path, and return the canonicalized path.
    pub fn resolve(
        &mut self,
        parent: Option<&ResolvedFile>,
        filename: &OsStr,
    ) -> Result<ResolvedFile, String> {
        let path_filename = PathBuf::from(filename);

        // See https://docs.soliditylang.org/en/v0.8.17/path-resolution.html
        let mut result: Vec<ResolvedFile> = vec![];

        // Only when the path starts with ./ or ../ are relative paths considered; this means
        // that `import "b.sol";` will check the import paths for b.sol, while `import "./b.sol";`
        // will only the path relative to the current file.
        if path_filename.starts_with("./") || path_filename.starts_with("../") {
            if let Some(ResolvedFile { import_no, full_path, .. }) = parent {
                let curdir = PathBuf::from(".");
                let base = full_path.parent().unwrap_or(&curdir);
                let path = base.join(&path_filename);

                if let Some(file) = self.try_file(filename, &path, *import_no)? {
                    // No ambiguity possible, so just return
                    return Ok(file);
                }
            }

            return Err(format!("file not found '{}'", path_filename.display()));
        }

        if parent.is_none() {
            if let Some(file) = self.try_file(filename, &path_filename, None)? {
                return Ok(file);
            } else if path_filename.is_absolute() {
                return Err(format!("file not found '{}'", path_filename.display()));
            }
        }

        // first check maps
        let mut remapped = path_filename.clone();

        for import_map_no in 0..self.import_paths.len() {
            if let (Some(mapping), target) = &self.import_paths[import_map_no].clone() {
                if let Ok(relpath) = path_filename.strip_prefix(mapping) {
                    remapped = target.join(relpath);
                }
            }
        }

        let path = remapped;

        // walk over the import paths until we find one that resolves
        for import_no in 0..self.import_paths.len() {
            if let (None, import_path) = &self.import_paths[import_no] {
                let path = import_path.join(&path);

                if let Some(file) = self.try_file(filename, &path, Some(import_no))? {
                    result.push(file);
                }
            }
        }

        // If there was no defined import path, then try the file directly. See
        // https://docs.soliditylang.org/en/v0.8.17/path-resolution.html#base-path-and-include-paths
        // "By default the base path is empty, which leaves the source unit name unchanged."
        if !self.import_paths.iter().any(|(m, _)| m.is_none()) {
            if let Some(file) = self.try_file(filename, &path, None)? {
                result.push(file);
            }
        }

        match result.len() {
            0 => Err(format!("file not found '{}'", path_filename.display())),
            1 => Ok(result.pop().unwrap()),
            _ => Err(format!(
                "found multiple files matching: '{}': {}",
                path_filename.display(),
                result.iter().map(|f| format!("'{}'", f.full_path.display())).join(", ")
            )),
        }
    }
}

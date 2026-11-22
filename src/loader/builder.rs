use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use crate::Result;
use crate::error::GnaError;

use super::GnaLibrary;

/// A builder to configure and execute searching and loading of the GNA library.
#[derive(Debug, Default, Clone)]
pub struct GnaLibraryBuilder {
    candidates: Vec<CandidateSource>,
    system_fallback: bool,
}

#[derive(Debug, Clone)]
enum CandidateSource {
    DirectPath(PathBuf),
    EnvVar(OsString),
}

impl GnaLibraryBuilder {
    /// Create a new empty builder.
    pub fn new() -> Self {
        Self {
            candidates: Vec::new(),
            system_fallback: false,
        }
    }

    /// Add an explicit file or directory path candidate to search.
    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.candidates
            .push(CandidateSource::DirectPath(path.as_ref().to_path_buf()));
        self
    }

    /// Add an environment variable candidate to search.
    /// If the environment variable value is a directory, the default DLL name is appended.
    pub fn env<K: AsRef<OsStr>>(mut self, var_name: K) -> Self {
        self.candidates
            .push(CandidateSource::EnvVar(var_name.as_ref().to_os_string()));
        self
    }

    /// Add standard environment variables (`GNA_LIB_PATH` and `GNA_LIB_DIR`) as search candidates.
    pub fn default_envs(self) -> Self {
        self.env("GNA_LIB_PATH").env("GNA_LIB_DIR")
    }

    /// Add the current working directory (`./gna.dll`) as a search candidate.
    pub fn current_dir(self) -> Self {
        self.path(PathBuf::from(GnaLibrary::default_dll_name()))
    }

    /// Set whether to fall back to the system DLL search path (e.g. system `PATH` or standard OS loader search)
    /// if none of the candidate files were found or loaded.
    pub fn system_fallback(mut self, enabled: bool) -> Self {
        self.system_fallback = enabled;
        self
    }

    /// Attempt to load the GNA library by trying each configured candidate in order.
    ///
    /// Returns `Ok(GnaLibrary)` on the first candidate that exists and loads successfully.
    /// If all candidates fail, returns `GnaError::LibrarySearchFailed` containing all tried paths and the last error encountered.
    pub fn load(&self) -> Result<GnaLibrary> {
        let default_name = GnaLibrary::default_dll_name();
        let mut tried_paths = Vec::new();
        let mut last_error: Option<String> = None;

        for candidate in &self.candidates {
            match candidate {
                CandidateSource::DirectPath(path) => {
                    let resolved = if path.is_dir() {
                        path.join(default_name)
                    } else {
                        path.clone()
                    };

                    if resolved.exists() {
                        match GnaLibrary::load_from_path(&resolved) {
                            Ok(lib) => return Ok(lib),
                            Err(err) => {
                                last_error = Some(err.to_string());
                            }
                        }
                    }
                    tried_paths.push(resolved);
                }
                CandidateSource::EnvVar(var_name) => {
                    if let Some(val) = env::var_os(var_name) {
                        let path = PathBuf::from(val);
                        let resolved = if path.is_dir() {
                            path.join(default_name)
                        } else {
                            path
                        };

                        if resolved.exists() {
                            match GnaLibrary::load_from_path(&resolved) {
                                Ok(lib) => return Ok(lib),
                                Err(err) => {
                                    last_error = Some(err.to_string());
                                }
                            }
                        }
                        tried_paths.push(resolved);
                    }
                }
            }
        }

        // If system fallback is enabled, try loading using OS standard search path
        if self.system_fallback {
            let sys_name = PathBuf::from(default_name);
            match GnaLibrary::load_from_path(&sys_name) {
                Ok(lib) => return Ok(lib),
                Err(err) => {
                    last_error = Some(err.to_string());
                    tried_paths.push(sys_name);
                }
            }
        }

        Err(GnaError::LibrarySearchFailed {
            tried: tried_paths,
            last_error,
        })
    }
}

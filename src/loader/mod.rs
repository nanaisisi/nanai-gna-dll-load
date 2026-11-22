mod builder;

pub use builder::GnaLibraryBuilder;

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::{GnaError, Result};
use crate::symbol_table::GnaSymbolTable;

/// Represents a dynamically loaded GNA runtime library.
///
/// Wrapping `libloading::Library` in an `Arc` ensures that loaded symbols
/// remain valid as long as any device or buffer referencing this library is alive.
#[derive(Clone)]
pub struct GnaLibrary {
    _lib: Option<Arc<libloading::Library>>,
    symbols: Arc<GnaSymbolTable>,
    path: PathBuf,
}

impl std::fmt::Debug for GnaLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GnaLibrary")
            .field("path", &self.path)
            .finish()
    }
}

impl GnaLibrary {
    /// Create a GnaLibrary instance from a symbol table without loading a dynamic library.
    /// Useful for testing, mocking, or static symbol tables.
    pub fn from_symbol_table(symbols: GnaSymbolTable) -> Self {
        Self {
            _lib: None,
            symbols: Arc::new(symbols),
            path: PathBuf::from("mock_gna.dll"),
        }
    }

    /// Return the standard platform-specific DLL/shared library file name.
    /// - Windows: `"gna.dll"`
    /// - macOS: `"libgna.dylib"`
    /// - Linux / other Unix: `"libgna.so"`
    pub fn default_dll_name() -> &'static str {
        #[cfg(target_os = "windows")]
        const DLL_NAME: &str = "gna.dll";
        #[cfg(target_os = "macos")]
        const DLL_NAME: &str = "libgna.dylib";
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        const DLL_NAME: &str = "libgna.so";

        DLL_NAME
    }

    /// Create a new builder to configure search paths, environment variables, and fallback rules.
    pub fn builder() -> GnaLibraryBuilder {
        GnaLibraryBuilder::new()
    }

    /// Attempt to load the GNA dynamic library from an explicit file path.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let lib_res = unsafe { libloading::Library::new(path_ref) };
        let lib = lib_res.map_err(GnaError::LibraryLoadError)?;

        let symbols = GnaSymbolTable::load(&lib)?;

        Ok(Self {
            _lib: Some(Arc::new(lib)),
            symbols: Arc::new(symbols),
            path: path_ref.to_path_buf(),
        })
    }

    /// Attempt to load the GNA dynamic library from a path or directory specified by an environment variable.
    ///
    /// If the environment variable points to a file, that file is loaded directly.
    /// If it points to a directory, the platform default library name (e.g. `gna.dll`) is appended.
    ///
    /// # Errors
    /// Returns `GnaError::EnvVarNotSet` if the environment variable is not defined or invalid unicode.
    /// Returns `GnaError::LibraryPathNotFound` if the resolved path does not exist on disk.
    /// Returns `GnaError::LibraryLoadError` or `GnaError::SymbolNotFound` if loading fails.
    pub fn load_from_env<K: AsRef<OsStr>>(var_name: K) -> Result<Self> {
        let var_os = var_name.as_ref();
        let var_str = var_os.to_string_lossy().into_owned();

        let val = env::var_os(var_os).ok_or(GnaError::EnvVarNotSet(var_str))?;
        let path = PathBuf::from(val);

        let target_path = if path.is_dir() {
            path.join(Self::default_dll_name())
        } else {
            path
        };

        if !target_path.exists() {
            return Err(GnaError::LibraryPathNotFound(target_path));
        }

        Self::load_from_path(target_path)
    }

    /// Attempt to load from the specified environment variable; if not set or not found, fall back to `fallback_path`.
    pub fn load_from_env_or_path<K: AsRef<OsStr>, P: AsRef<Path>>(
        var_name: K,
        fallback_path: P,
    ) -> Result<Self> {
        match Self::load_from_env(var_name) {
            Ok(lib) => Ok(lib),
            Err(_) => Self::load_from_path(fallback_path),
        }
    }

    /// Load the GNA library by searching standard and configured locations:
    /// 1. Environment variable `GNA_LIB_PATH` (direct file path or directory)
    /// 2. Directory specified by `GNA_LIB_DIR` / `gna.dll` (or `libgna.so`)
    /// 3. Current working directory
    /// 4. System DLL search path
    pub fn load_default() -> Result<Self> {
        Self::builder()
            .default_envs()
            .current_dir()
            .system_fallback(true)
            .load()
    }

    /// Get the path of the loaded library.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Access the resolved symbol table.
    pub fn symbols(&self) -> &GnaSymbolTable {
        &self.symbols
    }

    /// Query the library version string from GNA DLL if supported.
    pub fn get_library_version(&self) -> Result<String> {
        let version_fn = self
            .symbols()
            .get_library_version
            .ok_or_else(|| GnaError::Other("Gna2GetLibraryVersion is not supported".into()))?;

        let mut buf = vec![0u8; 128];
        let status =
            unsafe { version_fn(buf.as_mut_ptr() as *mut std::ffi::c_char, buf.len() as u32) };
        if status != crate::types::GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }

        let cstr = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as *const std::ffi::c_char) };
        Ok(cstr.to_string_lossy().into_owned())
    }

    /// Override model memory alignment constraints if supported.
    pub fn override_alignment(&self, alignment: u32) -> Result<()> {
        let align_fn = self
            .symbols()
            .model_override_alignment
            .ok_or_else(|| GnaError::Other("Gna2ModelOverrideAlignment is not supported".into()))?;

        let status = unsafe { align_fn(alignment) };
        if status != crate::types::GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }
}

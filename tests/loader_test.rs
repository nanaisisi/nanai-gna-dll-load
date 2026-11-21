use std::env;
use std::ffi::{OsStr, OsString, c_char, c_void};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use tempfile::tempdir;

use nanai_gna_dll_load::{
    error::GnaError,
    loader::{GnaLibrary, GnaLibraryBuilder},
    symbol_table::GnaSymbolTable,
    types::{GNA2_STATUS_SUCCESS, Gna2DeviceVersion, Gna2Status},
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    saved: Vec<(OsString, Option<OsString>)>,
}

impl EnvGuard {
    fn new() -> Self {
        Self { saved: Vec::new() }
    }

    fn save_key(&mut self, key: &OsStr) {
        if !self.saved.iter().any(|(k, _)| k.as_os_str() == key) {
            self.saved.push((key.to_os_string(), env::var_os(key)));
        }
    }

    fn set(&mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) {
        let key_ref = key.as_ref();
        self.save_key(key_ref);
        unsafe {
            env::set_var(key_ref, val);
        }
    }

    fn remove(&mut self, key: impl AsRef<OsStr>) {
        let key_ref = key.as_ref();
        self.save_key(key_ref);
        unsafe {
            env::remove_var(key_ref);
        }
    }

    fn strip_dll_from_path(&mut self, dll_name: &str) {
        self.save_key(OsStr::new("PATH"));
        if let Some(current_path) = env::var_os("PATH") {
            let filtered: Vec<PathBuf> = env::split_paths(&current_path)
                .filter(|dir| !dir.join(dll_name).exists())
                .collect();
            let joined = env::join_paths(filtered).expect("failed to join filtered PATH");
            unsafe {
                env::set_var("PATH", joined);
            }
        }
    }

    fn prepend_to_path(&mut self, dir: &Path) {
        self.save_key(OsStr::new("PATH"));
        let mut dirs = vec![dir.to_path_buf()];
        if let Some(current_path) = env::var_os("PATH") {
            dirs.extend(env::split_paths(&current_path));
        }
        let joined = env::join_paths(dirs).expect("failed to join prepended PATH");
        unsafe {
            env::set_var("PATH", joined);
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, orig) in self.saved.iter().rev() {
            unsafe {
                match orig {
                    Some(v) => env::set_var(key, v),
                    None => env::remove_var(key),
                }
            }
        }
    }
}

struct CwdGuard {
    original: PathBuf,
}

impl CwdGuard {
    fn enter(dir: &Path) -> Self {
        let original = env::current_dir().expect("failed to capture current_dir");
        env::set_current_dir(dir).expect("failed to switch current_dir");
        Self { original }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

const MINIMAL_MOCK_DLL_SRC: &str = r#"
use std::ffi::{c_char, c_void};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceGetCount(device_count: *mut u32) -> i32 {
    if !device_count.is_null() {
        unsafe { *device_count = 1 };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceGetVersion(_device_index: u32, device_version: *mut u32) -> i32 {
    if !device_version.is_null() {
        unsafe { *device_version = 0x30 };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceOpen(_device_index: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceClose(_device_index: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2MemoryAlloc(
    size_requested: u32,
    size_granted: *mut u32,
    memory_address: *mut *mut c_void,
) -> i32 {
    if !size_granted.is_null() {
        unsafe { *size_granted = size_requested };
    }
    if !memory_address.is_null() {
        unsafe { *memory_address = std::ptr::null_mut() };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2MemoryFree(_memory: *mut c_void) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2GetLibraryVersion(
    version_buffer: *mut c_char,
    version_buffer_size: u32,
) -> i32 {
    let ver = b"1.0.0-test\0";
    if version_buffer.is_null() || (version_buffer_size as usize) < ver.len() {
        return -6;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(ver.as_ptr(), version_buffer as *mut u8, ver.len());
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelOverrideAlignment(_new_alignment: u32) -> i32 {
    0
}
"#;

const MISSING_SYMBOL_DLL_SRC: &str = r#"
use std::ffi::c_void;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceGetCount(_device_count: *mut u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceGetVersion(_device_index: u32, _device_version: *mut u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceOpen(_device_index: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceClose(_device_index: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2MemoryAlloc(
    _size_requested: u32,
    _size_granted: *mut u32,
    _memory_address: *mut *mut c_void,
) -> i32 {
    0
}
// Intentionally omit Gna2MemoryFree
"#;

fn compile_cdylib(dir: &Path, stem: &str, filename: &str, source: &str) -> PathBuf {
    let src_path = dir.join(format!("{stem}.rs"));
    let dll_path = dir.join(filename);
    fs::write(&src_path, source).expect("failed to write mock DLL source");
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let output = Command::new(&rustc)
        .arg("--edition")
        .arg("2024")
        .arg("--crate-type")
        .arg("cdylib")
        .arg(&src_path)
        .arg("-o")
        .arg(&dll_path)
        .output()
        .expect("failed to execute rustc");
    assert!(
        output.status.success(),
        "rustc failed for {}: {}",
        dll_path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    dll_path
}

unsafe extern "C" fn stub_device_get_count(_count: *mut u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}
unsafe extern "C" fn stub_device_get_version(
    _idx: u32,
    _ver: *mut Gna2DeviceVersion,
) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}
unsafe extern "C" fn stub_device_open(_idx: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}
unsafe extern "C" fn stub_device_close(_idx: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}
unsafe extern "C" fn stub_memory_alloc(
    _size: u32,
    _granted: *mut u32,
    _addr: *mut *mut c_void,
) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}
unsafe extern "C" fn stub_memory_free(_mem: *mut c_void) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

fn base_mock_symbol_table() -> GnaSymbolTable {
    GnaSymbolTable {
        device_get_count: stub_device_get_count,
        device_get_version: stub_device_get_version,
        device_open: stub_device_open,
        device_create_for_export: None,
        device_close: stub_device_close,
        device_set_number_of_threads: None,
        get_library_version: None,
        memory_alloc: stub_memory_alloc,
        memory_alloc_for_device: None,
        memory_free: stub_memory_free,
        memory_set_tag: None,
        model_create: None,
        model_release: None,
        model_get_last_error: None,
        model_error_get_message: None,
        model_error_get_max_message_length: None,
        operation_init_fully_connected_affine: None,
        request_config_create: None,
        request_config_set_operand_buffer: None,
        request_config_enable_active_list: None,
        request_config_enable_hardware_consistency: None,
        request_config_set_acceleration_mode: None,
        request_config_release: None,
        request_enqueue: None,
        request_wait: None,
        instrumentation_config_create: None,
        instrumentation_config_assign_to_request_config: None,
        instrumentation_config_set_unit: None,
        instrumentation_config_set_mode: None,
        instrumentation_config_release: None,
        model_export_config_create: None,
        model_export_config_release: None,
        model_export_config_set_source: None,
        model_export_config_set_target: None,
        model_export: None,
        model_override_alignment: None,
        status_get_message: None,
        status_get_max_message_length: None,
    }
}

#[test]
fn test_default_dll_name() {
    let name = GnaLibrary::default_dll_name();
    #[cfg(target_os = "windows")]
    assert_eq!(name, "gna.dll");
    #[cfg(target_os = "macos")]
    assert_eq!(name, "libgna.dylib");
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    assert_eq!(name, "libgna.so");
}

#[test]
fn test_load_from_nonexistent_path() {
    let bad_path = PathBuf::from("nonexistent_path_to_gna.dll");
    let res = GnaLibrary::load_from_path(&bad_path);
    assert!(res.is_err());
    match res.unwrap_err() {
        GnaError::LibraryLoadError(_) => {}
        other => panic!("Expected LibraryLoadError, got: {:?}", other),
    }
}

#[test]
fn test_load_from_env_var_not_set() {
    let res = GnaLibrary::load_from_env("TOTALLY_NONEXISTENT_GNA_ENV_12345");
    assert!(res.is_err());
    match res.unwrap_err() {
        GnaError::EnvVarNotSet(var) => {
            assert_eq!(var, "TOTALLY_NONEXISTENT_GNA_ENV_12345");
        }
        other => panic!("Expected EnvVarNotSet, got: {:?}", other),
    }
}

#[test]
fn test_load_from_env_file_not_found() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut guard = EnvGuard::new();
    guard.set("TEST_GNA_NOT_FOUND_ENV", "C:\\some_invalid_path\\gna.dll");

    let res = GnaLibrary::load_from_env("TEST_GNA_NOT_FOUND_ENV");
    assert!(res.is_err());
    match res.unwrap_err() {
        GnaError::LibraryPathNotFound(p) => {
            assert!(p.to_string_lossy().contains("some_invalid_path"));
        }
        other => panic!("Expected LibraryPathNotFound, got: {:?}", other),
    }
}

#[test]
fn test_load_from_env_dir_appends_default_dll() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();

    let mut guard = EnvGuard::new();
    guard.set("TEST_GNA_DIR_ENV", &dir_path);

    let res = GnaLibrary::load_from_env("TEST_GNA_DIR_ENV");
    assert!(res.is_err());
    match res.unwrap_err() {
        GnaError::LibraryPathNotFound(target) => {
            assert_eq!(target, dir_path.join(GnaLibrary::default_dll_name()));
        }
        other => panic!("Expected LibraryPathNotFound, got: {:?}", other),
    }
}

#[test]
fn test_load_from_env_or_path_fallback() {
    let bad_fallback = PathBuf::from("fallback_invalid.dll");
    let res = GnaLibrary::load_from_env_or_path("TOTALLY_NONEXISTENT_GNA_ENV_9999", &bad_fallback);
    assert!(res.is_err());
    match res.unwrap_err() {
        GnaError::LibraryLoadError(_) => {}
        other => panic!("Expected LibraryLoadError, got: {:?}", other),
    }
}

#[test]
fn test_builder_search_failed_reporting() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut guard = EnvGuard::new();
    guard.remove("CUSTOM_ENV_FOR_TEST");

    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();

    // Create a dummy file that is not a valid DLL/PE image
    let dummy_file = dir_path.join("dummy_gna.dll");
    fs::write(&dummy_file, b"not a dll").unwrap();

    let res = GnaLibraryBuilder::new()
        .path(&dummy_file)
        .env("CUSTOM_ENV_FOR_TEST")
        .load();

    assert!(res.is_err());
    match res.unwrap_err() {
        GnaError::LibrarySearchFailed { tried, last_error } => {
            assert!(tried.contains(&dummy_file));
            assert!(last_error.is_some());
        }
        other => panic!("Expected LibrarySearchFailed, got: {:?}", other),
    }
}

#[test]
fn test_device_version_and_generations() {
    use nanai_gna_dll_load::types::{
        Gna2DeviceGeneration, Gna2DeviceVersion, Gna2InstrumentationMode, Gna2InstrumentationUnit,
        Gna2ModelExportComponent,
    };

    assert_eq!(Gna2DeviceVersion::SOFTWARE_EMULATION.0, 0);
    assert_eq!(Gna2DeviceVersion::EMBEDDED_1_0.0, 0x10E);
    assert_eq!(Gna2DeviceVersion::EMBEDDED_3_1.0, 0x310E);
    assert_eq!(Gna2DeviceVersion::EMBEDDED_3_1.as_str(), "GNA Embedded 3.1");

    assert_eq!(Gna2DeviceGeneration::Gen3_1 as u32, 0x310);
    assert_eq!(Gna2InstrumentationUnit::Cycles as u32, 2);
    assert_eq!(Gna2InstrumentationMode::Disabled as i32, -1);
    assert_eq!(
        Gna2ModelExportComponent::ExternalBufferOutputDump as u32,
        22
    );
}

#[test]
fn test_debug_clone_and_path_from_symbol_table() {
    let lib = GnaLibrary::from_symbol_table(base_mock_symbol_table());
    assert_eq!(lib.path(), Path::new("mock_gna.dll"));

    let dbg = format!("{:?}", lib);
    assert!(dbg.contains("GnaLibrary"));
    assert!(dbg.contains("mock_gna.dll"));

    let cloned = lib.clone();
    assert_eq!(cloned.path(), lib.path());
}

#[test]
fn test_get_library_version_none_err_and_ok() {
    // None branch
    let lib_none = GnaLibrary::from_symbol_table(base_mock_symbol_table());
    match lib_none.get_library_version().unwrap_err() {
        GnaError::Other(msg) => assert!(msg.contains("Gna2GetLibraryVersion")),
        other => panic!("Expected Other error, got: {:?}", other),
    }

    // Error status branch
    unsafe extern "C" fn version_err(_buf: *mut c_char, _len: u32) -> Gna2Status {
        -1
    }
    let mut table_err = base_mock_symbol_table();
    table_err.get_library_version = Some(version_err);
    let lib_err = GnaLibrary::from_symbol_table(table_err);
    match lib_err.get_library_version().unwrap_err() {
        GnaError::StatusError { status, .. } => assert_eq!(status, -1),
        other => panic!("Expected StatusError, got: {:?}", other),
    }

    // Success branch
    unsafe extern "C" fn version_ok(buf: *mut c_char, len: u32) -> Gna2Status {
        let ver = b"2.1.3\0";
        if buf.is_null() || (len as usize) < ver.len() {
            return -6;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(ver.as_ptr(), buf as *mut u8, ver.len());
        }
        GNA2_STATUS_SUCCESS
    }
    let mut table_ok = base_mock_symbol_table();
    table_ok.get_library_version = Some(version_ok);
    let lib_ok = GnaLibrary::from_symbol_table(table_ok);
    assert_eq!(lib_ok.get_library_version().unwrap(), "2.1.3");
}

#[test]
fn test_override_alignment_none_err_and_ok() {
    // None branch
    let lib_none = GnaLibrary::from_symbol_table(base_mock_symbol_table());
    match lib_none.override_alignment(64).unwrap_err() {
        GnaError::Other(msg) => assert!(msg.contains("Gna2ModelOverrideAlignment")),
        other => panic!("Expected Other error, got: {:?}", other),
    }

    // Error status branch
    unsafe extern "C" fn align_err(_alignment: u32) -> Gna2Status {
        -5
    }
    let mut table_err = base_mock_symbol_table();
    table_err.model_override_alignment = Some(align_err);
    let lib_err = GnaLibrary::from_symbol_table(table_err);
    match lib_err.override_alignment(64).unwrap_err() {
        GnaError::StatusError { status, .. } => assert_eq!(status, -5),
        other => panic!("Expected StatusError, got: {:?}", other),
    }

    // Success branch
    static LAST_ALIGNMENT: AtomicU32 = AtomicU32::new(0);
    unsafe extern "C" fn align_ok(alignment: u32) -> Gna2Status {
        LAST_ALIGNMENT.store(alignment, Ordering::SeqCst);
        GNA2_STATUS_SUCCESS
    }
    let mut table_ok = base_mock_symbol_table();
    table_ok.model_override_alignment = Some(align_ok);
    let lib_ok = GnaLibrary::from_symbol_table(table_ok);
    lib_ok.override_alignment(128).unwrap();
    assert_eq!(LAST_ALIGNMENT.load(Ordering::SeqCst), 128);
}

#[test]
fn test_load_from_path_success_and_missing_symbol() {
    let dir = tempdir().unwrap();
    let incomplete_dll = compile_cdylib(
        dir.path(),
        "incomplete",
        "incomplete.dll",
        MISSING_SYMBOL_DLL_SRC,
    );
    let valid_dll = compile_cdylib(dir.path(), "valid", "valid_gna.dll", MINIMAL_MOCK_DLL_SRC);

    match GnaLibrary::load_from_path(&incomplete_dll).unwrap_err() {
        GnaError::SymbolNotFound { symbol, .. } => assert_eq!(symbol, "Gna2MemoryFree"),
        other => panic!("Expected SymbolNotFound, got: {:?}", other),
    }

    let lib = GnaLibrary::load_from_path(&valid_dll).unwrap();
    assert_eq!(lib.path(), valid_dll.as_path());
    assert_eq!(lib.get_library_version().unwrap(), "1.0.0-test");
    lib.override_alignment(64).unwrap();
    let dbg = format!("{:?}", lib);
    assert!(dbg.contains("valid_gna.dll"));

    // Drop loaded library before TempDir drops so Windows unlocks the DLL file.
    drop(lib);
}

#[test]
fn test_load_from_env_and_env_or_path_success() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let compiled_dll = compile_cdylib(dir.path(), "mock", "direct_gna.dll", MINIMAL_MOCK_DLL_SRC);
    let incomplete_dll = compile_cdylib(
        dir.path(),
        "env_incomplete",
        "env_incomplete.dll",
        MISSING_SYMBOL_DLL_SRC,
    );

    let sub_dir = dir.path().join("sub_dir");
    fs::create_dir(&sub_dir).unwrap();
    let dir_dll = sub_dir.join(GnaLibrary::default_dll_name());
    fs::copy(&compiled_dll, &dir_dll).unwrap();

    let bad_dir = dir.path().join("bad_dir");
    fs::create_dir(&bad_dir).unwrap();
    let bad_dll = bad_dir.join(GnaLibrary::default_dll_name());
    fs::write(&bad_dll, b"not a dll").unwrap();

    let mut guard = EnvGuard::new();

    // Direct DLL path in env var
    guard.set("TEST_LOAD_ENV_FILE", &compiled_dll);
    let lib = GnaLibrary::load_from_env("TEST_LOAD_ENV_FILE").unwrap();
    assert_eq!(lib.path(), compiled_dll.as_path());
    drop(lib);

    // Directory path in env var (appends gna.dll)
    guard.set("TEST_LOAD_ENV_DIR", &sub_dir);
    let lib = GnaLibrary::load_from_env("TEST_LOAD_ENV_DIR").unwrap();
    assert_eq!(lib.path(), dir_dll.as_path());
    drop(lib);

    // Existing invalid DLL in env var hits load_from_path error
    guard.set("TEST_LOAD_ENV_BAD", &bad_dll);
    match GnaLibrary::load_from_env("TEST_LOAD_ENV_BAD").unwrap_err() {
        GnaError::LibraryLoadError(_) => {}
        other => panic!("Expected LibraryLoadError, got: {:?}", other),
    }

    // Existing DLL missing required symbol in env var hits symbol table error via PathBuf instantiation
    guard.set("TEST_LOAD_ENV_INCOMPLETE", &incomplete_dll);
    match GnaLibrary::load_from_env("TEST_LOAD_ENV_INCOMPLETE").unwrap_err() {
        GnaError::SymbolNotFound { symbol, .. } => assert_eq!(symbol, "Gna2MemoryFree"),
        other => panic!("Expected SymbolNotFound, got: {:?}", other),
    }

    // load_from_env_or_path succeeds via env var for both (&str, &str) and (&str, &PathBuf)
    let lib = GnaLibrary::load_from_env_or_path("TEST_LOAD_ENV_FILE", "nonexistent_fallback.dll")
        .unwrap();
    assert_eq!(lib.path(), compiled_dll.as_path());
    drop(lib);

    let lib = GnaLibrary::load_from_env_or_path("TEST_LOAD_ENV_FILE", &compiled_dll).unwrap();
    assert_eq!(lib.path(), compiled_dll.as_path());
    drop(lib);

    // load_from_env_or_path falls back to valid fallback path when env is unset (&PathBuf and &str)
    let lib =
        GnaLibrary::load_from_env_or_path("TOTALLY_UNSET_ENV_FOR_OR_PATH", &compiled_dll).unwrap();
    assert_eq!(lib.path(), compiled_dll.as_path());
    drop(lib);

    let compiled_str = compiled_dll.to_str().unwrap();
    let lib =
        GnaLibrary::load_from_env_or_path("TOTALLY_UNSET_ENV_FOR_OR_PATH", compiled_str).unwrap();
    assert_eq!(lib.path(), compiled_dll.as_path());
    drop(lib);

    // load_from_env_or_path (&str, &str) exercises load_from_path::<&str> error branches
    let incomplete_str = incomplete_dll.to_str().unwrap();
    match GnaLibrary::load_from_env_or_path("TOTALLY_UNSET_ENV_FOR_OR_PATH", incomplete_str)
        .unwrap_err()
    {
        GnaError::SymbolNotFound { symbol, .. } => assert_eq!(symbol, "Gna2MemoryFree"),
        other => panic!("Expected SymbolNotFound, got: {:?}", other),
    }

    match GnaLibrary::load_from_env_or_path(
        "TOTALLY_UNSET_ENV_FOR_OR_PATH",
        "nonexistent_fallback.dll",
    )
    .unwrap_err()
    {
        GnaError::LibraryLoadError(_) => {}
        other => panic!("Expected LibraryLoadError, got: {:?}", other),
    }
}

#[test]
fn test_builder_directory_and_env_candidates() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let compiled_dll = compile_cdylib(dir.path(), "mock", "mock.dll", MINIMAL_MOCK_DLL_SRC);

    let dir_empty = dir.path().join("dir_empty");
    let dir_bad = dir.path().join("dir_bad");
    let dir_ok = dir.path().join("dir_ok");
    let env_dir_empty = dir.path().join("env_dir_empty");
    let env_dir_bad = dir.path().join("env_dir_bad");
    let env_dir_ok = dir.path().join("env_dir_ok");
    for d in [
        &dir_empty,
        &dir_bad,
        &dir_ok,
        &env_dir_empty,
        &env_dir_bad,
        &env_dir_ok,
    ] {
        fs::create_dir(d).unwrap();
    }

    let default_name = GnaLibrary::default_dll_name();
    fs::write(dir_bad.join(default_name), b"corrupt dll").unwrap();
    fs::copy(&compiled_dll, dir_ok.join(default_name)).unwrap();
    fs::write(env_dir_bad.join(default_name), b"corrupt env dll").unwrap();
    fs::copy(&compiled_dll, env_dir_ok.join(default_name)).unwrap();

    let env_bad_file = dir.path().join("env_bad_file.dll");
    fs::write(&env_bad_file, b"corrupt file").unwrap();
    let env_missing_file = dir.path().join("does_not_exist.dll");

    let mut guard = EnvGuard::new();
    guard.remove("TEST_BUILDER_UNSET");
    guard.set("TEST_BUILDER_MISSING_FILE", &env_missing_file);
    guard.set("TEST_BUILDER_DIR_EMPTY", &env_dir_empty);
    guard.set("TEST_BUILDER_DIR_BAD", &env_dir_bad);
    guard.set("TEST_BUILDER_BAD_FILE", &env_bad_file);
    guard.set("TEST_BUILDER_OK_FILE", &compiled_dll);
    guard.set("TEST_BUILDER_OK_DIR", &env_dir_ok);

    // Builder defaults/debug/clone smoke check (empty and populated with DirectPath + EnvVar)
    let default_builder = GnaLibraryBuilder::default();
    let _cloned_builder = default_builder.clone();
    assert!(format!("{:?}", default_builder).contains("GnaLibraryBuilder"));

    let populated_builder = GnaLibraryBuilder::default()
        .path(&dir_empty)
        .env("TEST_BUILDER_OK_FILE")
        .system_fallback(true);
    let cloned_populated = populated_builder.clone();
    let dbg_populated = format!("{:?}", cloned_populated);
    assert!(dbg_populated.contains("DirectPath"));
    assert!(dbg_populated.contains("EnvVar"));
    assert!(dbg_populated.contains("system_fallback: true"));

    // All failing path & env candidates -> LibrarySearchFailed with tried paths and last_error
    let err = GnaLibrary::builder()
        .path(&dir_empty)
        .path(&dir_bad)
        .env("TEST_BUILDER_UNSET")
        .env("TEST_BUILDER_MISSING_FILE")
        .env("TEST_BUILDER_DIR_EMPTY")
        .env("TEST_BUILDER_DIR_BAD")
        .env("TEST_BUILDER_BAD_FILE")
        .load()
        .unwrap_err();
    match err {
        GnaError::LibrarySearchFailed { tried, last_error } => {
            assert!(tried.contains(&dir_empty.join(default_name)));
            assert!(tried.contains(&dir_bad.join(default_name)));
            assert!(tried.contains(&env_missing_file));
            assert!(tried.contains(&env_dir_empty.join(default_name)));
            assert!(tried.contains(&env_dir_bad.join(default_name)));
            assert!(tried.contains(&env_bad_file));
            assert!(last_error.is_some());
        }
        other => panic!("Expected LibrarySearchFailed, got: {:?}", other),
    }

    // Directory DirectPath candidate succeeds
    let lib = GnaLibrary::builder()
        .path(&dir_empty)
        .path(&dir_bad)
        .path(&dir_ok)
        .load()
        .unwrap();
    assert_eq!(lib.path(), dir_ok.join(default_name).as_path());
    drop(lib);

    // File EnvVar candidate succeeds
    let lib = GnaLibrary::builder()
        .env("TEST_BUILDER_UNSET")
        .env("TEST_BUILDER_BAD_FILE")
        .env("TEST_BUILDER_OK_FILE")
        .load()
        .unwrap();
    assert_eq!(lib.path(), compiled_dll.as_path());
    drop(lib);

    // Directory EnvVar candidate succeeds
    let lib = GnaLibrary::builder()
        .env("TEST_BUILDER_DIR_EMPTY")
        .env("TEST_BUILDER_OK_DIR")
        .load()
        .unwrap();
    assert_eq!(lib.path(), env_dir_ok.join(default_name).as_path());
    drop(lib);
}

#[test]
fn test_builder_current_dir_system_fallback_and_load_default() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let default_name = GnaLibrary::default_dll_name();

    // Compile mock DLL BEFORE altering PATH so rustc/linker can find toolchain tools.
    let compiled_dll = compile_cdylib(
        dir.path(),
        "mock",
        "compiled_mock.dll",
        MINIMAL_MOCK_DLL_SRC,
    );

    let empty_cwd = dir.path().join("empty_cwd");
    let cwd_with_dll = dir.path().join("cwd_with_dll");
    let path_with_dll = dir.path().join("path_with_dll");
    let env_with_dll = dir.path().join("env_with_dll");
    for d in [&empty_cwd, &cwd_with_dll, &path_with_dll, &env_with_dll] {
        fs::create_dir(d).unwrap();
    }
    fs::copy(&compiled_dll, cwd_with_dll.join(default_name)).unwrap();
    fs::copy(&compiled_dll, path_with_dll.join(default_name)).unwrap();
    fs::copy(&compiled_dll, env_with_dll.join(default_name)).unwrap();

    let mut guard = EnvGuard::new();
    guard.remove("GNA_LIB_PATH");
    guard.remove("GNA_LIB_DIR");
    guard.strip_dll_from_path(default_name);

    // 1. Clean environment + empty cwd -> default_envs + current_dir + system_fallback all fail
    {
        let _cwd = CwdGuard::enter(&empty_cwd);

        let err = GnaLibrary::builder()
            .default_envs()
            .current_dir()
            .system_fallback(true)
            .load()
            .unwrap_err();
        match err {
            GnaError::LibrarySearchFailed { tried, last_error } => {
                let expected = PathBuf::from(default_name);
                assert_eq!(tried, vec![expected.clone(), expected]);
                assert!(last_error.is_some());
            }
            other => panic!("Expected LibrarySearchFailed, got: {:?}", other),
        }

        let err_default = GnaLibrary::load_default().unwrap_err();
        assert!(matches!(err_default, GnaError::LibrarySearchFailed { .. }));
    }

    // 2. current_dir contains gna.dll -> builder().current_dir() and load_default() both succeed
    {
        let _cwd = CwdGuard::enter(&cwd_with_dll);

        let lib = GnaLibrary::builder().current_dir().load().unwrap();
        assert_eq!(lib.path(), Path::new(default_name));
        drop(lib);

        let lib = GnaLibrary::load_default().unwrap();
        assert_eq!(lib.path(), Path::new(default_name));
        drop(lib);
    }

    // 3. system_fallback(true) succeeds when gna.dll is discoverable via OS PATH
    {
        let _cwd = CwdGuard::enter(&empty_cwd);
        let mut path_guard = EnvGuard::new();
        path_guard.prepend_to_path(&path_with_dll);

        let lib = GnaLibrary::builder().system_fallback(true).load().unwrap();
        assert_eq!(lib.path(), Path::new(default_name));
        drop(lib);
    }

    // 4. load_default() succeeds via GNA_LIB_PATH and GNA_LIB_DIR
    {
        let _cwd = CwdGuard::enter(&empty_cwd);

        guard.set("GNA_LIB_PATH", &compiled_dll);
        let lib = GnaLibrary::load_default().unwrap();
        assert_eq!(lib.path(), compiled_dll.as_path());
        drop(lib);

        guard.remove("GNA_LIB_PATH");
        guard.set("GNA_LIB_DIR", &env_with_dll);
        let lib = GnaLibrary::load_default().unwrap();
        assert_eq!(lib.path(), env_with_dll.join(default_name).as_path());
        drop(lib);
    }
}

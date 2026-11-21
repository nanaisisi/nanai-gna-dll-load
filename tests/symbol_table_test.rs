use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use tempfile::tempdir;

use nanai_gna_dll_load::{
    error::GnaError,
    loader::{GnaLibrary, GnaLibraryBuilder},
    symbol_table::GnaSymbolTable,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    saved: Vec<(OsString, Option<OsString>)>,
}

impl EnvGuard {
    fn new() -> Self {
        Self { saved: Vec::new() }
    }

    fn set(&mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) {
        let key_ref = key.as_ref();
        if !self.saved.iter().any(|(k, _)| k.as_os_str() == key_ref) {
            self.saved
                .push((key_ref.to_os_string(), env::var_os(key_ref)));
        }
        unsafe {
            env::set_var(key_ref, val);
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

const ALL_SYMBOLS_DLL_SRC: &str = r#"
use std::ffi::{c_char, c_void};

// Required symbols (6)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceGetCount(device_count: *mut u32) -> i32 {
    if !device_count.is_null() {
        unsafe { *device_count = 2 };
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

// Optional symbols (32)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceCreateForExport(
    _target_device_version: u32,
    device_index: *mut u32,
) -> i32 {
    if !device_index.is_null() {
        unsafe { *device_index = 0 };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceSetNumberOfThreads(
    _device_index: u32,
    _number_of_threads: u32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2GetLibraryVersion(
    version_buffer: *mut c_char,
    version_buffer_size: u32,
) -> i32 {
    let ver = b"3.5.0-mock\0";
    if version_buffer.is_null() || (version_buffer_size as usize) < ver.len() {
        return -6;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(ver.as_ptr(), version_buffer as *mut u8, ver.len());
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2MemoryAllocForDevice(
    _device_index: u32,
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
pub unsafe extern "C" fn Gna2MemorySetTag(_memory: *mut c_void, _tag: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelCreate(
    _device_index: u32,
    _model: *const c_void,
    model_id: *mut u32,
) -> i32 {
    if !model_id.is_null() {
        unsafe { *model_id = 1 };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelRelease(_model_id: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelGetLastError(_error: *mut c_void) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelErrorGetMessage(
    _error: *const c_void,
    _message_buffer: *mut c_char,
    _message_buffer_size: u32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelErrorGetMaxMessageLength() -> u32 {
    512
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2OperationInitFullyConnectedAffine(
    _operation: *mut c_void,
    _user_allocator: *const c_void,
    _inputs: *mut c_void,
    _outputs: *mut c_void,
    _weights: *mut c_void,
    _biases: *mut c_void,
    _activation: *mut c_void,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2RequestConfigCreate(
    _model_id: u32,
    request_config_id: *mut u32,
) -> i32 {
    if !request_config_id.is_null() {
        unsafe { *request_config_id = 1 };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2RequestConfigSetOperandBuffer(
    _request_config_id: u32,
    _operation_index: u32,
    _operand_index: u32,
    _address: *mut c_void,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2RequestConfigEnableActiveList(
    _request_config_id: u32,
    _operation_index: u32,
    _number_of_indices: u32,
    _indices: *const u32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2RequestConfigEnableHardwareConsistency(
    _request_config_id: u32,
    _device_version: u32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2RequestConfigSetAccelerationMode(
    _request_config_id: u32,
    _acceleration_mode: i32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2RequestConfigRelease(_request_config_id: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2RequestEnqueue(
    _request_config_id: u32,
    request_id: *mut u32,
) -> i32 {
    if !request_id.is_null() {
        unsafe { *request_id = 1 };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2RequestWait(_request_id: u32, _timeout_milliseconds: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2InstrumentationConfigCreate(
    _number_of_points: u32,
    _selected_points: *const i32,
    _results: *mut u64,
    instrumentation_config_id: *mut u32,
) -> i32 {
    if !instrumentation_config_id.is_null() {
        unsafe { *instrumentation_config_id = 1 };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2InstrumentationConfigAssignToRequestConfig(
    _instrumentation_config_id: u32,
    _request_config_id: u32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2InstrumentationConfigSetUnit(
    _instrumentation_config_id: u32,
    _instrumentation_unit: i32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2InstrumentationConfigSetMode(
    _instrumentation_config_id: u32,
    _instrumentation_mode: i32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2InstrumentationConfigRelease(
    _instrumentation_config_id: u32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelExportConfigCreate(
    _user_allocator: *const c_void,
    export_config_id: *mut u32,
) -> i32 {
    if !export_config_id.is_null() {
        unsafe { *export_config_id = 1 };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelExportConfigRelease(_export_config_id: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelExportConfigSetSource(
    _export_config_id: u32,
    _source_device_index: u32,
    _source_model_id: u32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelExportConfigSetTarget(
    _export_config_id: u32,
    _target_device_version: u32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelExport(
    _export_config_id: u32,
    _component_type: i32,
    _export_buffer: *mut *mut c_void,
    _export_buffer_size: *mut u32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelOverrideAlignment(_new_alignment: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2StatusGetMessage(
    _status: i32,
    _buffer: *mut c_char,
    _buffer_size: u32,
) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2StatusGetMaxMessageLength() -> u32 {
    256
}
"#;

const REQUIRED_SYMBOLS: [(&str, &str); 6] = [
    (
        "Gna2DeviceGetCount",
        r#"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceGetCount(device_count: *mut u32) -> i32 {
    if !device_count.is_null() {
        unsafe { *device_count = 1 };
    }
    0
}
"#,
    ),
    (
        "Gna2DeviceGetVersion",
        r#"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceGetVersion(_device_index: u32, device_version: *mut u32) -> i32 {
    if !device_version.is_null() {
        unsafe { *device_version = 0x20 };
    }
    0
}
"#,
    ),
    (
        "Gna2DeviceOpen",
        r#"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceOpen(_device_index: u32) -> i32 {
    0
}
"#,
    ),
    (
        "Gna2DeviceClose",
        r#"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceClose(_device_index: u32) -> i32 {
    0
}
"#,
    ),
    (
        "Gna2MemoryAlloc",
        r#"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2MemoryAlloc(
    size_requested: u32,
    size_granted: *mut u32,
    memory_address: *mut *mut std::ffi::c_void,
) -> i32 {
    if !size_granted.is_null() {
        unsafe { *size_granted = size_requested };
    }
    if !memory_address.is_null() {
        unsafe { *memory_address = std::ptr::null_mut() };
    }
    0
}
"#,
    ),
    (
        "Gna2MemoryFree",
        r#"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2MemoryFree(_memory: *mut std::ffi::c_void) -> i32 {
    0
}
"#,
    ),
];

fn compile_mock_dll(dir: &Path, stem: &str, dll_name: &str, source: &str) -> PathBuf {
    let src_path = dir.join(format!("{stem}.rs"));
    let dll_path = dir.join(dll_name);
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
        .expect("failed to invoke rustc");
    assert!(
        output.status.success(),
        "rustc failed to compile {}: {}",
        dll_path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    dll_path
}

fn build_required_source(omit: Option<&str>) -> String {
    let mut src = String::new();
    for (name, body) in REQUIRED_SYMBOLS {
        if omit != Some(name) {
            src.push_str(body);
        }
    }
    src
}

fn assert_missing_required_symbol(omitted: &'static str) {
    let dir = tempdir().unwrap();
    let source = build_required_source(Some(omitted));
    let dll_path = compile_mock_dll(dir.path(), "partial", "partial_gna.dll", &source);

    // Verify GnaSymbolTable::load directly
    {
        let raw_lib = unsafe { libloading::Library::new(&dll_path) }.unwrap();
        match GnaSymbolTable::load(&raw_lib) {
            Err(GnaError::SymbolNotFound { symbol, .. }) => assert_eq!(symbol, omitted),
            Ok(_) => panic!("Expected SymbolNotFound for {}, got Ok", omitted),
            Err(other) => panic!("Expected SymbolNotFound for {}, got: {:?}", omitted, other),
        }
        drop(raw_lib);
    }

    // Verify GnaLibrary::load_from_path propagates SymbolNotFound
    match GnaLibrary::load_from_path(&dll_path) {
        Err(GnaError::SymbolNotFound { symbol, .. }) => assert_eq!(symbol, omitted),
        Ok(_) => panic!("Expected SymbolNotFound for {}, got Ok", omitted),
        Err(other) => panic!("Expected SymbolNotFound for {}, got: {:?}", omitted, other),
    }
}

#[test]
fn test_symbol_table_load_all_symbols_present_and_loader_success() {
    let dir = tempdir().unwrap();
    let default_name = GnaLibrary::default_dll_name();
    let dll_path = compile_mock_dll(dir.path(), "all_syms", default_name, ALL_SYMBOLS_DLL_SRC);

    // 1. Direct GnaSymbolTable::load verification
    {
        let raw_lib = unsafe { libloading::Library::new(&dll_path) }.unwrap();
        let table = GnaSymbolTable::load(&raw_lib).unwrap();

        assert!(table.device_create_for_export.is_some());
        assert!(table.device_set_number_of_threads.is_some());
        assert!(table.get_library_version.is_some());
        assert!(table.memory_alloc_for_device.is_some());
        assert!(table.memory_set_tag.is_some());
        assert!(table.model_create.is_some());
        assert!(table.model_release.is_some());
        assert!(table.model_get_last_error.is_some());
        assert!(table.model_error_get_message.is_some());
        assert!(table.model_error_get_max_message_length.is_some());
        assert!(table.operation_init_fully_connected_affine.is_some());
        assert!(table.request_config_create.is_some());
        assert!(table.request_config_set_operand_buffer.is_some());
        assert!(table.request_config_enable_active_list.is_some());
        assert!(table.request_config_enable_hardware_consistency.is_some());
        assert!(table.request_config_set_acceleration_mode.is_some());
        assert!(table.request_config_release.is_some());
        assert!(table.request_enqueue.is_some());
        assert!(table.request_wait.is_some());
        assert!(table.instrumentation_config_create.is_some());
        assert!(
            table
                .instrumentation_config_assign_to_request_config
                .is_some()
        );
        assert!(table.instrumentation_config_set_unit.is_some());
        assert!(table.instrumentation_config_set_mode.is_some());
        assert!(table.instrumentation_config_release.is_some());
        assert!(table.model_export_config_create.is_some());
        assert!(table.model_export_config_release.is_some());
        assert!(table.model_export_config_set_source.is_some());
        assert!(table.model_export_config_set_target.is_some());
        assert!(table.model_export.is_some());
        assert!(table.model_override_alignment.is_some());
        assert!(table.status_get_message.is_some());
        assert!(table.status_get_max_message_length.is_some());

        let mut count = 0u32;
        let status = unsafe { (table.device_get_count)(&mut count) };
        assert_eq!(status, 0);
        assert_eq!(count, 2);
        assert_eq!(
            unsafe { (table.model_error_get_max_message_length.unwrap())() },
            512
        );
        assert_eq!(
            unsafe { (table.status_get_max_message_length.unwrap())() },
            256
        );

        drop(raw_lib);
    }

    // 2. GnaLibrary::load_from_path
    {
        let lib = GnaLibrary::load_from_path(&dll_path).unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        assert_eq!(lib.get_library_version().unwrap(), "3.5.0-mock");
        lib.override_alignment(64).unwrap();
        assert!(lib.symbols().model_create.is_some());
        drop(lib);
    }

    // 3. load_from_env, load_from_env_or_path, and GnaLibraryBuilder::load
    {
        let _lock = ENV_LOCK.lock().unwrap();
        let mut guard = EnvGuard::new();
        guard.set("TEST_SYM_TABLE_FILE", &dll_path);
        guard.set("TEST_SYM_TABLE_DIR", dir.path());

        let lib = GnaLibrary::load_from_env("TEST_SYM_TABLE_FILE").unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        drop(lib);

        let lib = GnaLibrary::load_from_env("TEST_SYM_TABLE_DIR").unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        drop(lib);

        let lib = GnaLibrary::load_from_env_or_path("TEST_SYM_TABLE_FILE", "missing_fallback.dll")
            .unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        drop(lib);

        let lib =
            GnaLibrary::load_from_env_or_path("TEST_SYM_TABLE_MISSING_VAR", &dll_path).unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        drop(lib);

        let lib = GnaLibraryBuilder::new().path(&dll_path).load().unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        drop(lib);

        let lib = GnaLibraryBuilder::new()
            .env("TEST_SYM_TABLE_DIR")
            .load()
            .unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        drop(lib);
    }
}

#[test]
fn test_symbol_table_load_required_only_symbols() {
    let dir = tempdir().unwrap();
    let source = build_required_source(None);
    let dll_path = compile_mock_dll(dir.path(), "req_only", "req_only_gna.dll", &source);

    {
        let raw_lib = unsafe { libloading::Library::new(&dll_path) }.unwrap();
        let table = GnaSymbolTable::load(&raw_lib).unwrap();

        assert!(table.device_create_for_export.is_none());
        assert!(table.device_set_number_of_threads.is_none());
        assert!(table.get_library_version.is_none());
        assert!(table.memory_alloc_for_device.is_none());
        assert!(table.memory_set_tag.is_none());
        assert!(table.model_create.is_none());
        assert!(table.model_release.is_none());
        assert!(table.model_get_last_error.is_none());
        assert!(table.model_error_get_message.is_none());
        assert!(table.model_error_get_max_message_length.is_none());
        assert!(table.operation_init_fully_connected_affine.is_none());
        assert!(table.request_config_create.is_none());
        assert!(table.request_config_set_operand_buffer.is_none());
        assert!(table.request_config_enable_active_list.is_none());
        assert!(table.request_config_enable_hardware_consistency.is_none());
        assert!(table.request_config_set_acceleration_mode.is_none());
        assert!(table.request_config_release.is_none());
        assert!(table.request_enqueue.is_none());
        assert!(table.request_wait.is_none());
        assert!(table.instrumentation_config_create.is_none());
        assert!(
            table
                .instrumentation_config_assign_to_request_config
                .is_none()
        );
        assert!(table.instrumentation_config_set_unit.is_none());
        assert!(table.instrumentation_config_set_mode.is_none());
        assert!(table.instrumentation_config_release.is_none());
        assert!(table.model_export_config_create.is_none());
        assert!(table.model_export_config_release.is_none());
        assert!(table.model_export_config_set_source.is_none());
        assert!(table.model_export_config_set_target.is_none());
        assert!(table.model_export.is_none());
        assert!(table.model_override_alignment.is_none());
        assert!(table.status_get_message.is_none());
        assert!(table.status_get_max_message_length.is_none());

        let mut count = 0u32;
        assert_eq!(unsafe { (table.device_get_count)(&mut count) }, 0);
        assert_eq!(count, 1);

        drop(raw_lib);
    }

    let lib = GnaLibrary::load_from_path(&dll_path).unwrap();
    assert!(matches!(lib.get_library_version(), Err(GnaError::Other(_))));
    assert!(matches!(
        lib.override_alignment(64),
        Err(GnaError::Other(_))
    ));
    drop(lib);
}

#[test]
fn test_symbol_table_missing_device_get_count() {
    assert_missing_required_symbol("Gna2DeviceGetCount");
}

#[test]
fn test_symbol_table_missing_device_get_version() {
    assert_missing_required_symbol("Gna2DeviceGetVersion");
}

#[test]
fn test_symbol_table_missing_device_open() {
    assert_missing_required_symbol("Gna2DeviceOpen");
}

#[test]
fn test_symbol_table_missing_device_close() {
    assert_missing_required_symbol("Gna2DeviceClose");
}

#[test]
fn test_symbol_table_missing_memory_alloc() {
    assert_missing_required_symbol("Gna2MemoryAlloc");
}

#[test]
fn test_symbol_table_missing_memory_free() {
    assert_missing_required_symbol("Gna2MemoryFree");
}

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

use nanai_gna_dll_load::{error::GnaError, loader::GnaLibrary, symbol_table::GnaSymbolTable};

pub(super) const REQUIRED_SYMBOLS: [(&str, &str); 6] = [
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

pub(super) fn compile_mock_dll(dir: &Path, stem: &str, dll_name: &str, source: &str) -> PathBuf {
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

pub(super) fn build_required_source(omit: Option<&str>) -> String {
    let mut src = String::new();
    for (name, body) in REQUIRED_SYMBOLS {
        if omit != Some(name) {
            src.push_str(body);
        }
    }
    src
}

pub(super) fn assert_missing_required_symbol(omitted: &'static str) {
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

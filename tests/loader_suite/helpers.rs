use std::env;
use std::ffi::c_void;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use nanai_gna_dll_load::{
    symbol_table::GnaSymbolTable,
    types::{GNA2_STATUS_SUCCESS, Gna2DeviceVersion, Gna2Status},
};

pub(super) fn compile_cdylib(dir: &Path, stem: &str, filename: &str, source: &str) -> PathBuf {
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

pub(super) fn base_mock_symbol_table() -> GnaSymbolTable {
    GnaSymbolTable::from_required(
        stub_device_get_count,
        stub_device_get_version,
        stub_device_open,
        stub_device_close,
        stub_memory_alloc,
        stub_memory_free,
    )
}

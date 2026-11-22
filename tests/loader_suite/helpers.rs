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

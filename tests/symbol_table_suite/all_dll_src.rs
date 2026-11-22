pub(super) const ALL_SYMBOLS_DLL_SRC: &str = r#"
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

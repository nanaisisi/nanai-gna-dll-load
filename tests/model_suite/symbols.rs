use std::ffi::{c_char, c_void};
use std::ptr;

use nanai_gna_dll_load::{
    symbol_table::GnaSymbolTable,
    types::{
        GNA2_STATUS_SUCCESS, Gna2DataType, Gna2DeviceVersion, Gna2ModelError, Gna2Status,
        Gna2Tensor,
    },
};

use super::mock::{
    MOCK_STATE, mock_model_create, mock_model_release, mock_operation_init_fully_connected_affine,
};

unsafe extern "C" fn mock_device_get_count(device_count: *mut u32) -> Gna2Status {
    if !device_count.is_null() {
        unsafe { *device_count = 1 };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_get_version(
    device_index: u32,
    device_version: *mut Gna2DeviceVersion,
) -> Gna2Status {
    if device_index != 0 {
        return -5;
    }
    if !device_version.is_null() {
        unsafe { *device_version = Gna2DeviceVersion::SOFTWARE_EMULATION };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_open(_device_index: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_close(_device_index: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_memory_alloc(
    _size_requested: u32,
    _size_granted: *mut u32,
    _memory_address: *mut *mut c_void,
) -> Gna2Status {
    -4
}

unsafe extern "C" fn mock_memory_free(_memory: *mut c_void) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_model_get_last_error(error: *mut Gna2ModelError) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let state = s.borrow();
        if state.get_last_error_status == GNA2_STATUS_SUCCESS && !error.is_null() {
            unsafe { *error = state.last_error_desc };
        }
        state.get_last_error_status
    })
}

unsafe extern "C" fn mock_model_error_get_message(
    _error: *const Gna2ModelError,
    message_buffer: *mut c_char,
    message_buffer_size: u32,
) -> Gna2Status {
    let (status, msg) = MOCK_STATE.with(|s| {
        let state = s.borrow();
        (state.error_get_message_status, state.error_message.clone())
    });
    if status != GNA2_STATUS_SUCCESS {
        return status;
    }
    if !message_buffer.is_null() && message_buffer_size > 0 {
        let bytes = msg.as_bytes();
        let copy_len = bytes.len().min((message_buffer_size - 1) as usize);
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), message_buffer as *mut u8, copy_len);
            *message_buffer.add(copy_len) = 0;
        }
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_model_error_get_max_message_length() -> u32 {
    MOCK_STATE.with(|s| s.borrow().max_error_message_len)
}

unsafe extern "C" fn mock_status_get_message(
    _status: Gna2Status,
    buffer: *mut c_char,
    buffer_size: u32,
) -> Gna2Status {
    let (status, msg) = MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.status_get_message_calls += 1;
        (
            state.status_get_message_status,
            state.status_message.clone(),
        )
    });
    if status != GNA2_STATUS_SUCCESS {
        return status;
    }
    if !buffer.is_null() && buffer_size > 0 {
        let bytes = msg.as_bytes();
        let copy_len = bytes.len().min((buffer_size - 1) as usize);
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, copy_len);
            *buffer.add(copy_len) = 0;
        }
    }
    GNA2_STATUS_SUCCESS
}

pub(super) fn full_mock_symbol_table() -> GnaSymbolTable {
    GnaSymbolTable {
        device_get_count: mock_device_get_count,
        device_get_version: mock_device_get_version,
        device_open: mock_device_open,
        device_create_for_export: None,
        device_close: mock_device_close,
        device_set_number_of_threads: None,
        get_library_version: None,
        memory_alloc: mock_memory_alloc,
        memory_alloc_for_device: None,
        memory_free: mock_memory_free,
        memory_set_tag: None,
        model_create: Some(mock_model_create),
        model_release: Some(mock_model_release),
        model_get_last_error: Some(mock_model_get_last_error),
        model_error_get_message: Some(mock_model_error_get_message),
        model_error_get_max_message_length: Some(mock_model_error_get_max_message_length),
        operation_init_fully_connected_affine: Some(mock_operation_init_fully_connected_affine),
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
        status_get_message: Some(mock_status_get_message),
        status_get_max_message_length: None,
    }
}

pub(super) fn sample_tensors(
    in_buf: &mut [i16],
    out_buf: &mut [i32],
    w_buf: &mut [i16],
    b_buf: &mut [i32],
) -> (Gna2Tensor, Gna2Tensor, Gna2Tensor, Gna2Tensor) {
    (
        Gna2Tensor::d2(
            16,
            4,
            Gna2DataType::Int16,
            in_buf.as_mut_ptr() as *mut c_void,
        ),
        Gna2Tensor::d2(
            8,
            4,
            Gna2DataType::Int32,
            out_buf.as_mut_ptr() as *mut c_void,
        ),
        Gna2Tensor::d2(
            8,
            16,
            Gna2DataType::Int16,
            w_buf.as_mut_ptr() as *mut c_void,
        ),
        Gna2Tensor::d1(8, Gna2DataType::Int32, b_buf.as_mut_ptr() as *mut c_void),
    )
}

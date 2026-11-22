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
    GnaSymbolTable::from_required(
        mock_device_get_count,
        mock_device_get_version,
        mock_device_open,
        mock_device_close,
        mock_memory_alloc,
        mock_memory_free,
    )
    .with_model_create(mock_model_create)
    .with_model_release(mock_model_release)
    .with_model_get_last_error(mock_model_get_last_error)
    .with_model_error_get_message(mock_model_error_get_message)
    .with_model_error_get_max_message_length(mock_model_error_get_max_message_length)
    .with_operation_init_fully_connected_affine(mock_operation_init_fully_connected_affine)
    .with_status_get_message(mock_status_get_message)
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

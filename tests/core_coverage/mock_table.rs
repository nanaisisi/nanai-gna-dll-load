use std::alloc::dealloc;
use std::ffi::c_void;

use nanai_gna_dll_load::loader::GnaLibrary;
use nanai_gna_dll_load::symbol_table::GnaSymbolTable;
use nanai_gna_dll_load::types::{GNA2_STATUS_SUCCESS, Gna2DeviceVersion, Gna2Status};

use super::mock_state::*;

unsafe extern "C" fn mock_device_create_for_export(
    _target_device_version: Gna2DeviceVersion,
    device_index: *mut u32,
) -> Gna2Status {
    let (status, idx) = read_mock_state(|s| {
        (
            s.device_create_for_export_status,
            s.device_create_for_export_index,
        )
    });
    if status == GNA2_STATUS_SUCCESS && !device_index.is_null() {
        unsafe { *device_index = idx };
    }
    status
}

unsafe extern "C" fn mock_device_set_number_of_threads(
    device_index: u32,
    number_of_threads: u32,
) -> Gna2Status {
    with_mock_state(|s| {
        s.last_threads_arg = Some((device_index, number_of_threads));
        s.device_set_threads_status
    })
}

unsafe extern "C" fn mock_memory_alloc(
    size_requested: u32,
    size_granted: *mut u32,
    memory_address: *mut *mut c_void,
) -> Gna2Status {
    let (status, return_null, granted_override) = with_mock_state(|s| {
        s.memory_alloc_calls.push(size_requested);
        (
            s.memory_alloc_status,
            s.memory_alloc_return_null,
            s.memory_alloc_granted_override,
        )
    });

    if status != GNA2_STATUS_SUCCESS {
        return status;
    }

    let granted = granted_override.unwrap_or(size_requested);
    if !size_granted.is_null() {
        unsafe { *size_granted = granted };
    }

    if return_null {
        if !memory_address.is_null() {
            unsafe { *memory_address = std::ptr::null_mut() };
        }
        return GNA2_STATUS_SUCCESS;
    }

    let Some(ptr) = alloc_tracked(size_requested) else {
        return -8;
    };
    if !memory_address.is_null() {
        unsafe { *memory_address = ptr };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_memory_alloc_for_device(
    device_index: u32,
    size_requested: u32,
    size_granted: *mut u32,
    memory_address: *mut *mut c_void,
) -> Gna2Status {
    let (status, return_null) = with_mock_state(|s| {
        s.memory_alloc_for_device_calls
            .push((device_index, size_requested));
        (
            s.memory_alloc_for_device_status,
            s.memory_alloc_for_device_return_null,
        )
    });

    if status != GNA2_STATUS_SUCCESS {
        return status;
    }

    if !size_granted.is_null() {
        unsafe { *size_granted = size_requested };
    }

    if return_null {
        if !memory_address.is_null() {
            unsafe { *memory_address = std::ptr::null_mut() };
        }
        return GNA2_STATUS_SUCCESS;
    }

    let Some(ptr) = alloc_tracked(size_requested) else {
        return -8;
    };
    if !memory_address.is_null() {
        unsafe { *memory_address = ptr };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_memory_free(memory: *mut c_void) -> Gna2Status {
    if !memory.is_null()
        && let Some(layout) = allocations().lock().unwrap().remove(&(memory as usize))
    {
        unsafe { dealloc(memory as *mut u8, layout) };
    }
    with_mock_state(|s| {
        s.memory_free_calls.push(memory as usize);
        s.memory_free_status
    })
}

unsafe extern "C" fn mock_memory_set_tag(memory: *mut c_void, tag: u32) -> Gna2Status {
    with_mock_state(|s| {
        s.memory_set_tag_calls.push((memory as usize, tag));
        s.memory_set_tag_status
    })
}

pub(super) fn base_mock_symbol_table() -> GnaSymbolTable {
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

pub(super) fn full_mock_library() -> GnaLibrary {
    let mut symbols = base_mock_symbol_table();
    symbols.device_create_for_export = Some(mock_device_create_for_export);
    symbols.device_set_number_of_threads = Some(mock_device_set_number_of_threads);
    symbols.memory_alloc_for_device = Some(mock_memory_alloc_for_device);
    symbols.memory_set_tag = Some(mock_memory_set_tag);
    GnaLibrary::from_symbol_table(symbols)
}

// ---------------------------------------------------------------------------
// src/error.rs tests
// ---------------------------------------------------------------------------

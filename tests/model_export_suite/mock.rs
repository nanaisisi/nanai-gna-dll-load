use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr;

use nanai_gna_dll_load::{
    symbol_table::GnaSymbolTable,
    types::{
        GNA2_STATUS_SUCCESS, Gna2DeviceVersion, Gna2ModelExportComponent, Gna2Status,
        Gna2UserAllocator,
    },
};

#[derive(Default)]
pub(super) struct MockExportState {
    pub create_status: Gna2Status,
    pub set_source_status: Gna2Status,
    pub set_target_status: Gna2Status,
    pub export_status: Gna2Status,

    pub next_config_id: u32,
    pub saved_allocator: Option<Gna2UserAllocator>,
    pub allocator_zero_was_null: bool,
    pub allocator_scratch_succeeded: bool,

    pub recorded_source: Option<(u32, u32, u32)>,
    pub recorded_target: Option<(u32, Gna2DeviceVersion)>,
    pub recorded_export_call: Option<(u32, Gna2ModelExportComponent)>,
    pub export_payload: Vec<u8>,

    pub release_calls: u32,
    pub released_ids: Vec<u32>,
}

thread_local! {
    pub(super) static MOCK_STATE: RefCell<MockExportState> = RefCell::new(MockExportState::default());
    static USER_ALLOCATIONS: RefCell<HashMap<usize, Layout>> = RefCell::new(HashMap::new());
}

pub(super) unsafe extern "C" fn test_user_allocator(size: u32) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }
    let layout = Layout::from_size_align(size as usize, 16).expect("valid allocation layout");
    let ptr = unsafe { alloc_zeroed(layout) };
    if !ptr.is_null() {
        USER_ALLOCATIONS.with(|map| {
            map.borrow_mut().insert(ptr as usize, layout);
        });
    }
    ptr as *mut c_void
}

pub(super) fn free_user_allocation(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    USER_ALLOCATIONS.with(|map| {
        if let Some(layout) = map.borrow_mut().remove(&(ptr as usize)) {
            unsafe {
                dealloc(ptr as *mut u8, layout);
            }
        }
    });
}

pub(super) fn remaining_user_allocations() -> usize {
    USER_ALLOCATIONS.with(|map| map.borrow().len())
}

pub(super) fn reset_mock_state() {
    let leftover: Vec<(usize, Layout)> =
        USER_ALLOCATIONS.with(|map| map.borrow_mut().drain().collect());
    for (addr, layout) in leftover {
        unsafe {
            dealloc(addr as *mut u8, layout);
        }
    }

    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        *state = MockExportState {
            next_config_id: 42,
            export_payload: b"GNA_EXPORT_BINARY_V3".to_vec(),
            ..MockExportState::default()
        };
    });
}

unsafe extern "C" fn mock_device_get_count(device_count: *mut u32) -> Gna2Status {
    if !device_count.is_null() {
        unsafe { *device_count = 1 };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_get_version(
    _device_index: u32,
    device_version: *mut Gna2DeviceVersion,
) -> Gna2Status {
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

unsafe extern "C" fn mock_model_export_config_create(
    user_allocator: Gna2UserAllocator,
    export_config_id: *mut u32,
) -> Gna2Status {
    let status = MOCK_STATE.with(|s| s.borrow().create_status);
    if status != GNA2_STATUS_SUCCESS {
        return status;
    }

    let zero_ptr = unsafe { user_allocator(0) };
    let scratch_ptr = unsafe { user_allocator(16) };
    let scratch_ok = !scratch_ptr.is_null();
    free_user_allocation(scratch_ptr);

    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.saved_allocator = Some(user_allocator);
        state.allocator_zero_was_null = zero_ptr.is_null();
        state.allocator_scratch_succeeded = scratch_ok;
        if !export_config_id.is_null() {
            unsafe { *export_config_id = state.next_config_id };
            state.next_config_id += 1;
        }
    });

    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_model_export_config_release(export_config_id: u32) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.release_calls += 1;
        state.released_ids.push(export_config_id);
    });
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_model_export_config_set_source(
    export_config_id: u32,
    source_device_index: u32,
    source_model_id: u32,
) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.recorded_source = Some((export_config_id, source_device_index, source_model_id));
        state.set_source_status
    })
}

unsafe extern "C" fn mock_model_export_config_set_target(
    export_config_id: u32,
    target_device_version: Gna2DeviceVersion,
) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.recorded_target = Some((export_config_id, target_device_version));
        state.set_target_status
    })
}

unsafe extern "C" fn mock_model_export(
    export_config_id: u32,
    component_type: Gna2ModelExportComponent,
    export_buffer: *mut *mut c_void,
    export_buffer_size: *mut u32,
) -> Gna2Status {
    let (status, allocator_opt, payload) = MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.recorded_export_call = Some((export_config_id, component_type));
        (
            state.export_status,
            state.saved_allocator,
            state.export_payload.clone(),
        )
    });

    if status != GNA2_STATUS_SUCCESS {
        return status;
    }

    let allocator = allocator_opt.expect("saved user allocator should exist");
    let buf = unsafe { allocator(payload.len() as u32) };
    if buf.is_null() {
        return -8;
    }

    unsafe {
        ptr::copy_nonoverlapping(payload.as_ptr(), buf as *mut u8, payload.len());
        if !export_buffer.is_null() {
            *export_buffer = buf;
        }
        if !export_buffer_size.is_null() {
            *export_buffer_size = payload.len() as u32;
        }
    }

    GNA2_STATUS_SUCCESS
}

pub(super) fn full_export_symbol_table() -> GnaSymbolTable {
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
        model_export_config_create: Some(mock_model_export_config_create),
        model_export_config_release: Some(mock_model_export_config_release),
        model_export_config_set_source: Some(mock_model_export_config_set_source),
        model_export_config_set_target: Some(mock_model_export_config_set_target),
        model_export: Some(mock_model_export),
        model_override_alignment: None,
        status_get_message: None,
        status_get_max_message_length: None,
    }
}

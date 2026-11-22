use std::alloc::{Layout, alloc_zeroed};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use nanai_gna_dll_load::types::{GNA2_STATUS_SUCCESS, Gna2DeviceVersion, Gna2Status};

#[derive(Clone, Debug)]

pub(super) struct MockState {
    pub(super) device_count: u32,
    pub(super) device_get_count_status: Gna2Status,
    pub(super) device_get_version_status: Gna2Status,
    pub(super) device_version_value: Gna2DeviceVersion,
    pub(super) device_open_status: Gna2Status,
    pub(super) device_close_status: Gna2Status,
    pub(super) device_create_for_export_status: Gna2Status,
    pub(super) device_create_for_export_index: u32,
    pub(super) device_set_threads_status: Gna2Status,
    pub(super) last_threads_arg: Option<(u32, u32)>,
    pub(super) close_calls: Vec<u32>,

    pub(super) memory_alloc_status: Gna2Status,
    pub(super) memory_alloc_return_null: bool,
    pub(super) memory_alloc_granted_override: Option<u32>,
    pub(super) memory_alloc_calls: Vec<u32>,

    pub(super) memory_alloc_for_device_status: Gna2Status,
    pub(super) memory_alloc_for_device_return_null: bool,
    pub(super) memory_alloc_for_device_calls: Vec<(u32, u32)>,

    pub(super) memory_free_status: Gna2Status,
    pub(super) memory_free_calls: Vec<usize>,

    pub(super) memory_set_tag_status: Gna2Status,
    pub(super) memory_set_tag_calls: Vec<(usize, u32)>,
}

impl Default for MockState {
    fn default() -> Self {
        Self {
            device_count: 1,
            device_get_count_status: GNA2_STATUS_SUCCESS,
            device_get_version_status: GNA2_STATUS_SUCCESS,
            device_version_value: Gna2DeviceVersion::V3_0,
            device_open_status: GNA2_STATUS_SUCCESS,
            device_close_status: GNA2_STATUS_SUCCESS,
            device_create_for_export_status: GNA2_STATUS_SUCCESS,
            device_create_for_export_index: 7,
            device_set_threads_status: GNA2_STATUS_SUCCESS,
            last_threads_arg: None,
            close_calls: Vec::new(),

            memory_alloc_status: GNA2_STATUS_SUCCESS,
            memory_alloc_return_null: false,
            memory_alloc_granted_override: None,
            memory_alloc_calls: Vec::new(),

            memory_alloc_for_device_status: GNA2_STATUS_SUCCESS,
            memory_alloc_for_device_return_null: false,
            memory_alloc_for_device_calls: Vec::new(),

            memory_free_status: GNA2_STATUS_SUCCESS,
            memory_free_calls: Vec::new(),

            memory_set_tag_status: GNA2_STATUS_SUCCESS,
            memory_set_tag_calls: Vec::new(),
        }
    }
}

thread_local! {
    static MOCK_STATE: RefCell<MockState> = RefCell::new(MockState::default());
}

pub(super) fn allocations() -> &'static Mutex<HashMap<usize, Layout>> {
    static ALLOCATIONS: OnceLock<Mutex<HashMap<usize, Layout>>> = OnceLock::new();
    ALLOCATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn reset_mock_state() {
    MOCK_STATE.with(|s| {
        *s.borrow_mut() = MockState::default();
    });
}

pub(super) fn with_mock_state<R>(f: impl FnOnce(&mut MockState) -> R) -> R {
    MOCK_STATE.with(|s| f(&mut s.borrow_mut()))
}

pub(super) fn read_mock_state<R>(f: impl FnOnce(&MockState) -> R) -> R {
    MOCK_STATE.with(|s| f(&s.borrow()))
}

pub(super) fn alloc_tracked(size_requested: u32) -> Option<*mut c_void> {
    let alloc_size = size_requested.max(1) as usize;
    let layout = Layout::from_size_align(alloc_size, 64).ok()?;
    let ptr = unsafe { alloc_zeroed(layout) };
    if ptr.is_null() {
        return None;
    }
    allocations().lock().unwrap().insert(ptr as usize, layout);
    Some(ptr as *mut c_void)
}

pub(super) unsafe extern "C" fn mock_device_get_count(device_count: *mut u32) -> Gna2Status {
    let (status, count) = read_mock_state(|s| (s.device_get_count_status, s.device_count));
    if status == GNA2_STATUS_SUCCESS && !device_count.is_null() {
        unsafe { *device_count = count };
    }
    status
}

pub(super) unsafe extern "C" fn mock_device_get_version(
    _device_index: u32,
    device_version: *mut Gna2DeviceVersion,
) -> Gna2Status {
    let (status, ver) = read_mock_state(|s| (s.device_get_version_status, s.device_version_value));
    if status == GNA2_STATUS_SUCCESS && !device_version.is_null() {
        unsafe { *device_version = ver };
    }
    status
}

pub(super) unsafe extern "C" fn mock_device_open(_device_index: u32) -> Gna2Status {
    read_mock_state(|s| s.device_open_status)
}

pub(super) unsafe extern "C" fn mock_device_close(device_index: u32) -> Gna2Status {
    with_mock_state(|s| {
        s.close_calls.push(device_index);
        s.device_close_status
    })
}

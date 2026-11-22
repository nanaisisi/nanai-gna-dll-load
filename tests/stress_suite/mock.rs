use std::cell::RefCell;
use std::ffi::c_void;
use std::time::Duration;

use nanai_gna_dll_load::{
    symbol_table::GnaSymbolTable,
    types::{
        GNA2_STATUS_SUCCESS, Gna2DeviceVersion, Gna2InstrumentationMode, Gna2InstrumentationPoint,
        Gna2InstrumentationUnit, Gna2Status,
    },
};

#[derive(Default)]
pub(super) struct MockStressState {
    pub next_req_id: u32,
    pub enqueue_calls: usize,
    pub wait_calls: usize,

    pub fail_enqueue_on_call: Option<usize>,
    pub enqueue_delays_ms: Vec<(usize, u64)>,

    pub fail_wait_on_call: Option<usize>,
    pub wait_delays_ms: Vec<(usize, u64)>,

    pub total_cycles: u64,
    pub stall_cycles: u64,
    pub exec_time: u64,
}

thread_local! {
    pub(super) static MOCK_STRESS: RefCell<MockStressState> = RefCell::new(MockStressState::default());
}

pub(super) fn reset_stress_state() {
    MOCK_STRESS.with(|s| {
        *s.borrow_mut() = MockStressState {
            next_req_id: 1,
            total_cycles: 1000,
            stall_cycles: 250,
            exec_time: 40,
            ..Default::default()
        };
    });
}

unsafe extern "C" fn stub_device_get_count(count: *mut u32) -> Gna2Status {
    if !count.is_null() {
        unsafe { *count = 1 };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn stub_device_get_version(
    _index: u32,
    version: *mut Gna2DeviceVersion,
) -> Gna2Status {
    if !version.is_null() {
        unsafe { *version = Gna2DeviceVersion::V3_0 };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn stub_device_open(_index: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn stub_device_close(_index: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn stub_memory_alloc(
    _size: u32,
    _granted: *mut u32,
    _addr: *mut *mut c_void,
) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn stub_memory_free(_ptr: *mut c_void) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_req_create(_model_id: u32, cfg_id: *mut u32) -> Gna2Status {
    if !cfg_id.is_null() {
        unsafe { *cfg_id = 1 };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_req_release(_cfg_id: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_enqueue(_cfg_id: u32, req_id: *mut u32) -> Gna2Status {
    let (should_fail, delay_ms, assigned_id) = MOCK_STRESS.with(|s| {
        let mut st = s.borrow_mut();
        st.enqueue_calls += 1;
        let call_idx = st.enqueue_calls;
        let should_fail = st.fail_enqueue_on_call == Some(call_idx);
        let delay_ms = st
            .enqueue_delays_ms
            .iter()
            .find_map(|&(idx, ms)| if idx == call_idx { Some(ms) } else { None })
            .unwrap_or(0);
        let id = st.next_req_id;
        st.next_req_id += 1;
        (should_fail, delay_ms, id)
    });

    if delay_ms > 0 {
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
    if should_fail {
        return -12;
    }
    if !req_id.is_null() {
        unsafe { *req_id = assigned_id };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_wait(_req_id: u32, _timeout_ms: u32) -> Gna2Status {
    let (should_fail, delay_ms) = MOCK_STRESS.with(|s| {
        let mut st = s.borrow_mut();
        st.wait_calls += 1;
        let call_idx = st.wait_calls;
        let should_fail = st.fail_wait_on_call == Some(call_idx);
        let delay_ms = st
            .wait_delays_ms
            .iter()
            .find_map(|&(idx, ms)| if idx == call_idx { Some(ms) } else { None })
            .unwrap_or(0);
        (should_fail, delay_ms)
    });

    if delay_ms > 0 {
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
    if should_fail {
        return -13;
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_inst_create(
    number_of_points: u32,
    selected_points: *const Gna2InstrumentationPoint,
    results: *mut u64,
    inst_id: *mut u32,
) -> Gna2Status {
    MOCK_STRESS.with(|s| {
        let st = s.borrow();
        if !inst_id.is_null() {
            unsafe { *inst_id = 9 };
        }
        if !selected_points.is_null() && !results.is_null() {
            let pts =
                unsafe { std::slice::from_raw_parts(selected_points, number_of_points as usize) };
            let res = unsafe { std::slice::from_raw_parts_mut(results, number_of_points as usize) };
            for (pt, slot) in pts.iter().zip(res.iter_mut()) {
                *slot = match pt {
                    Gna2InstrumentationPoint::HwTotalCycles => st.total_cycles,
                    Gna2InstrumentationPoint::HwStallCycles => st.stall_cycles,
                    Gna2InstrumentationPoint::LibExecution => st.exec_time,
                    _ => 0,
                };
            }
        }
    });
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_inst_assign(_inst_id: u32, _req_id: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_inst_set_unit(
    _inst_id: u32,
    _unit: Gna2InstrumentationUnit,
) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_inst_set_mode(
    _inst_id: u32,
    _mode: Gna2InstrumentationMode,
) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_inst_release(_inst_id: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

pub(super) fn full_stress_symbols() -> GnaSymbolTable {
    GnaSymbolTable::from_required(
        stub_device_get_count,
        stub_device_get_version,
        stub_device_open,
        stub_device_close,
        stub_memory_alloc,
        stub_memory_free,
    )
    .with_request_config_create(mock_req_create)
    .with_request_config_release(mock_req_release)
    .with_request_enqueue(mock_enqueue)
    .with_request_wait(mock_wait)
    .with_instrumentation_config_create(mock_inst_create)
    .with_instrumentation_config_assign_to_request_config(mock_inst_assign)
    .with_instrumentation_config_set_unit(mock_inst_set_unit)
    .with_instrumentation_config_set_mode(mock_inst_set_mode)
    .with_instrumentation_config_release(mock_inst_release)
}

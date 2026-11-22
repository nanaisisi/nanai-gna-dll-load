use std::cell::RefCell;
use std::ffi::c_void;

use nanai_gna_dll_load::symbol_table::GnaSymbolTable;
use nanai_gna_dll_load::types::{
    GNA2_STATUS_SUCCESS, Gna2DeviceVersion, Gna2InstrumentationMode, Gna2InstrumentationPoint,
    Gna2InstrumentationUnit, Gna2Status,
};

#[derive(Default)]
pub(super) struct MockInstState {
    pub next_id: u32,
    pub create_status: Gna2Status,
    pub assign_status: Gna2Status,
    pub set_unit_status: Gna2Status,
    pub set_mode_status: Gna2Status,

    pub total_cycles: u64,
    pub stall_cycles: u64,
    pub exec_time: u64,
    pub other_point_value: u64,

    pub last_assign: Option<(u32, u32)>,
    pub last_unit: Option<(u32, Gna2InstrumentationUnit)>,
    pub last_mode: Option<(u32, Gna2InstrumentationMode)>,
    pub released_ids: Vec<u32>,
}

thread_local! {
    pub(super) static MOCK_INST_STATE: RefCell<MockInstState> = RefCell::new(MockInstState::default());
}

pub(super) fn reset_mock_inst_state() {
    MOCK_INST_STATE.with(|s| {
        *s.borrow_mut() = MockInstState {
            next_id: 1,
            create_status: GNA2_STATUS_SUCCESS,
            assign_status: GNA2_STATUS_SUCCESS,
            set_unit_status: GNA2_STATUS_SUCCESS,
            set_mode_status: GNA2_STATUS_SUCCESS,
            total_cycles: 1000,
            stall_cycles: 250,
            exec_time: 42,
            other_point_value: 7,
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

pub(super) unsafe extern "C" fn mock_inst_create(
    number_of_points: u32,
    selected_points: *const Gna2InstrumentationPoint,
    results: *mut u64,
    instrumentation_config_id: *mut u32,
) -> Gna2Status {
    MOCK_INST_STATE.with(|s| {
        let mut st = s.borrow_mut();
        if st.create_status != GNA2_STATUS_SUCCESS {
            return st.create_status;
        }
        let id = st.next_id;
        st.next_id += 1;
        if !instrumentation_config_id.is_null() {
            unsafe { *instrumentation_config_id = id };
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
                    Gna2InstrumentationPoint::LibSubmission => 100,
                    Gna2InstrumentationPoint::LibCompletion => 100 + st.exec_time,
                    _ => st.other_point_value,
                };
            }
        }
        GNA2_STATUS_SUCCESS
    })
}

unsafe extern "C" fn mock_inst_assign(inst_id: u32, req_config_id: u32) -> Gna2Status {
    MOCK_INST_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.last_assign = Some((inst_id, req_config_id));
        st.assign_status
    })
}

unsafe extern "C" fn mock_inst_set_unit(inst_id: u32, unit: Gna2InstrumentationUnit) -> Gna2Status {
    MOCK_INST_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.last_unit = Some((inst_id, unit));
        st.set_unit_status
    })
}

unsafe extern "C" fn mock_inst_set_mode(inst_id: u32, mode: Gna2InstrumentationMode) -> Gna2Status {
    MOCK_INST_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.last_mode = Some((inst_id, mode));
        st.set_mode_status
    })
}

unsafe extern "C" fn mock_inst_release(inst_id: u32) -> Gna2Status {
    MOCK_INST_STATE.with(|s| {
        s.borrow_mut().released_ids.push(inst_id);
    });
    GNA2_STATUS_SUCCESS
}

pub(super) fn empty_symbols() -> GnaSymbolTable {
    GnaSymbolTable::from_required(
        stub_device_get_count,
        stub_device_get_version,
        stub_device_open,
        stub_device_close,
        stub_memory_alloc,
        stub_memory_free,
    )
}

pub(super) fn full_inst_symbols() -> GnaSymbolTable {
    empty_symbols()
        .with_instrumentation_config_create(mock_inst_create)
        .with_instrumentation_config_assign_to_request_config(mock_inst_assign)
        .with_instrumentation_config_set_unit(mock_inst_set_unit)
        .with_instrumentation_config_set_mode(mock_inst_set_mode)
        .with_instrumentation_config_release(mock_inst_release)
}

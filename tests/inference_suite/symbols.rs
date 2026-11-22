use std::ffi::c_void;

use nanai_gna_dll_load::symbol_table::GnaSymbolTable;
use nanai_gna_dll_load::types::{
    GNA2_STATUS_SUCCESS, Gna2DeviceVersion, Gna2InstrumentationMode, Gna2InstrumentationPoint,
    Gna2InstrumentationUnit, Gna2Status,
};

use super::mock::{
    MOCK_STATE, mock_request_config_create, mock_request_config_enable_active_list,
    mock_request_config_enable_hardware_consistency, mock_request_config_release,
    mock_request_config_set_acceleration_mode, mock_request_config_set_operand_buffer,
    mock_request_enqueue, mock_request_wait,
};

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

unsafe extern "C" fn mock_inst_config_create(
    number_of_points: u32,
    selected_points: *const Gna2InstrumentationPoint,
    results: *mut u64,
    instrumentation_config_id: *mut u32,
) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        if st.inst_create_status != GNA2_STATUS_SUCCESS {
            return st.inst_create_status;
        }
        let id = st.next_inst_id;
        st.next_inst_id += 1;
        if !instrumentation_config_id.is_null() {
            unsafe { *instrumentation_config_id = id };
        }
        if !selected_points.is_null() && !results.is_null() {
            let pts =
                unsafe { std::slice::from_raw_parts(selected_points, number_of_points as usize) };
            let res = unsafe { std::slice::from_raw_parts_mut(results, number_of_points as usize) };
            for (pt, slot) in pts.iter().zip(res.iter_mut()) {
                *slot = match pt {
                    Gna2InstrumentationPoint::HwTotalCycles => st.mock_total_cycles,
                    Gna2InstrumentationPoint::HwStallCycles => st.mock_stall_cycles,
                    Gna2InstrumentationPoint::LibExecution => st.mock_exec_time,
                    _ => 0,
                };
            }
        }
        GNA2_STATUS_SUCCESS
    })
}

unsafe extern "C" fn mock_inst_assign(_inst_id: u32, _req_id: u32) -> Gna2Status {
    MOCK_STATE.with(|s| s.borrow().inst_assign_status)
}

unsafe extern "C" fn mock_inst_set_unit(
    _inst_id: u32,
    _unit: Gna2InstrumentationUnit,
) -> Gna2Status {
    MOCK_STATE.with(|s| s.borrow().inst_set_unit_status)
}

unsafe extern "C" fn mock_inst_set_mode(
    _inst_id: u32,
    _mode: Gna2InstrumentationMode,
) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_inst_release(inst_id: u32) -> Gna2Status {
    MOCK_STATE.with(|s| {
        s.borrow_mut().released_inst_ids.push(inst_id);
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

pub(super) fn full_symbols() -> GnaSymbolTable {
    empty_symbols()
        .with_request_config_create(mock_request_config_create)
        .with_request_config_set_operand_buffer(mock_request_config_set_operand_buffer)
        .with_request_config_enable_active_list(mock_request_config_enable_active_list)
        .with_request_config_enable_hardware_consistency(
            mock_request_config_enable_hardware_consistency,
        )
        .with_request_config_set_acceleration_mode(mock_request_config_set_acceleration_mode)
        .with_request_config_release(mock_request_config_release)
        .with_request_enqueue(mock_request_enqueue)
        .with_request_wait(mock_request_wait)
        .with_instrumentation_config_create(mock_inst_config_create)
        .with_instrumentation_config_assign_to_request_config(mock_inst_assign)
        .with_instrumentation_config_set_unit(mock_inst_set_unit)
        .with_instrumentation_config_set_mode(mock_inst_set_mode)
        .with_instrumentation_config_release(mock_inst_release)
}

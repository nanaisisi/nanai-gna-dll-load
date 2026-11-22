use std::cell::RefCell;
use std::ffi::c_void;

use nanai_gna_dll_load::types::{
    GNA2_STATUS_SUCCESS, Gna2AccelerationMode, Gna2DeviceVersion, Gna2InstrumentationUnit,
    Gna2Status,
};

#[derive(Default)]
pub(super) struct MockInferenceState {
    pub next_config_id: u32,
    pub next_request_id: u32,
    pub next_inst_id: u32,

    pub create_status: Gna2Status,
    pub set_operand_buffer_status: Gna2Status,
    pub enable_active_list_status: Gna2Status,
    pub enable_hardware_consistency_status: Gna2Status,
    pub set_acceleration_mode_status: Gna2Status,
    pub enqueue_status: Gna2Status,
    pub wait_status: Gna2Status,

    pub inst_create_status: Gna2Status,
    pub inst_assign_status: Gna2Status,
    pub inst_set_unit_status: Gna2Status,

    pub mock_total_cycles: u64,
    pub mock_stall_cycles: u64,
    pub mock_exec_time: u64,

    pub last_model_id: Option<u32>,
    pub last_operand_buf: Option<(u32, u32, u32, usize)>,
    pub last_active_list: Option<(u32, u32, Vec<u32>)>,
    pub last_hw_consistency: Option<(u32, Gna2DeviceVersion)>,
    pub last_accel_mode: Option<(u32, Gna2AccelerationMode)>,
    pub last_enqueue_config_id: Option<u32>,
    pub last_wait_args: Option<(u32, u32)>,
    pub last_inst_unit: Option<(u32, Gna2InstrumentationUnit)>,
    pub released_config_ids: Vec<u32>,
    pub released_inst_ids: Vec<u32>,
}

thread_local! {
    pub(super) static MOCK_STATE: RefCell<MockInferenceState> = RefCell::new(MockInferenceState::default());
}

pub(super) fn reset_mock_state() {
    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        *st = MockInferenceState {
            next_config_id: 10,
            next_request_id: 100,
            next_inst_id: 50,
            create_status: GNA2_STATUS_SUCCESS,
            set_operand_buffer_status: GNA2_STATUS_SUCCESS,
            enable_active_list_status: GNA2_STATUS_SUCCESS,
            enable_hardware_consistency_status: GNA2_STATUS_SUCCESS,
            set_acceleration_mode_status: GNA2_STATUS_SUCCESS,
            enqueue_status: GNA2_STATUS_SUCCESS,
            wait_status: GNA2_STATUS_SUCCESS,
            inst_create_status: GNA2_STATUS_SUCCESS,
            inst_assign_status: GNA2_STATUS_SUCCESS,
            inst_set_unit_status: GNA2_STATUS_SUCCESS,
            mock_total_cycles: 2000,
            mock_stall_cycles: 500,
            mock_exec_time: 80,
            ..Default::default()
        };
    });
}

pub(super) unsafe extern "C" fn mock_request_config_create(
    model_id: u32,
    request_config_id: *mut u32,
) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.last_model_id = Some(model_id);
        if st.create_status != GNA2_STATUS_SUCCESS {
            return st.create_status;
        }
        let id = st.next_config_id;
        st.next_config_id += 1;
        if !request_config_id.is_null() {
            unsafe { *request_config_id = id };
        }
        GNA2_STATUS_SUCCESS
    })
}

pub(super) unsafe extern "C" fn mock_request_config_set_operand_buffer(
    request_config_id: u32,
    operation_index: u32,
    operand_index: u32,
    address: *mut c_void,
) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.last_operand_buf = Some((
            request_config_id,
            operation_index,
            operand_index,
            address as usize,
        ));
        st.set_operand_buffer_status
    })
}

pub(super) unsafe extern "C" fn mock_request_config_enable_active_list(
    request_config_id: u32,
    operation_index: u32,
    number_of_indices: u32,
    indices: *const u32,
) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        let slice = if indices.is_null() || number_of_indices == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(indices, number_of_indices as usize) }.to_vec()
        };
        st.last_active_list = Some((request_config_id, operation_index, slice));
        st.enable_active_list_status
    })
}

pub(super) unsafe extern "C" fn mock_request_config_enable_hardware_consistency(
    request_config_id: u32,
    device_version: Gna2DeviceVersion,
) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.last_hw_consistency = Some((request_config_id, device_version));
        st.enable_hardware_consistency_status
    })
}

pub(super) unsafe extern "C" fn mock_request_config_set_acceleration_mode(
    request_config_id: u32,
    acceleration_mode: Gna2AccelerationMode,
) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.last_accel_mode = Some((request_config_id, acceleration_mode));
        st.set_acceleration_mode_status
    })
}

pub(super) unsafe extern "C" fn mock_request_config_release(request_config_id: u32) -> Gna2Status {
    MOCK_STATE.with(|s| {
        s.borrow_mut().released_config_ids.push(request_config_id);
    });
    GNA2_STATUS_SUCCESS
}

pub(super) unsafe extern "C" fn mock_request_enqueue(
    request_config_id: u32,
    request_id: *mut u32,
) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.last_enqueue_config_id = Some(request_config_id);
        if st.enqueue_status != GNA2_STATUS_SUCCESS {
            return st.enqueue_status;
        }
        let id = st.next_request_id;
        st.next_request_id += 1;
        if !request_id.is_null() {
            unsafe { *request_id = id };
        }
        GNA2_STATUS_SUCCESS
    })
}

pub(super) unsafe extern "C" fn mock_request_wait(
    request_id: u32,
    timeout_milliseconds: u32,
) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.last_wait_args = Some((request_id, timeout_milliseconds));
        st.wait_status
    })
}

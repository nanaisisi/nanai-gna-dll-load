use std::cell::RefCell;
use std::ffi::c_void;

use nanai_gna_dll_load::error::GnaError;
use nanai_gna_dll_load::inference::GnaRequestConfig;
use nanai_gna_dll_load::instrumentation::GnaInstrumentationConfig;
use nanai_gna_dll_load::loader::GnaLibrary;
use nanai_gna_dll_load::symbol_table::GnaSymbolTable;
use nanai_gna_dll_load::types::{
    GNA2_STATUS_SUCCESS, Gna2AccelerationMode, Gna2DeviceVersion, Gna2InstrumentationMode,
    Gna2InstrumentationPoint, Gna2InstrumentationUnit, Gna2Status,
};

#[derive(Default)]
struct MockInferenceState {
    next_config_id: u32,
    next_request_id: u32,
    next_inst_id: u32,

    create_status: Gna2Status,
    set_operand_buffer_status: Gna2Status,
    enable_active_list_status: Gna2Status,
    enable_hardware_consistency_status: Gna2Status,
    set_acceleration_mode_status: Gna2Status,
    enqueue_status: Gna2Status,
    wait_status: Gna2Status,

    inst_create_status: Gna2Status,
    inst_assign_status: Gna2Status,
    inst_set_unit_status: Gna2Status,

    mock_total_cycles: u64,
    mock_stall_cycles: u64,
    mock_exec_time: u64,

    last_model_id: Option<u32>,
    last_operand_buf: Option<(u32, u32, u32, usize)>,
    last_active_list: Option<(u32, u32, Vec<u32>)>,
    last_hw_consistency: Option<(u32, Gna2DeviceVersion)>,
    last_accel_mode: Option<(u32, Gna2AccelerationMode)>,
    last_enqueue_config_id: Option<u32>,
    last_wait_args: Option<(u32, u32)>,
    released_config_ids: Vec<u32>,
    released_inst_ids: Vec<u32>,
}

thread_local! {
    static MOCK_STATE: RefCell<MockInferenceState> = RefCell::new(MockInferenceState::default());
}

fn reset_mock_state() {
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

unsafe extern "C" fn mock_request_config_create(
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

unsafe extern "C" fn mock_request_config_set_operand_buffer(
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

unsafe extern "C" fn mock_request_config_enable_active_list(
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

unsafe extern "C" fn mock_request_config_enable_hardware_consistency(
    request_config_id: u32,
    device_version: Gna2DeviceVersion,
) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.last_hw_consistency = Some((request_config_id, device_version));
        st.enable_hardware_consistency_status
    })
}

unsafe extern "C" fn mock_request_config_set_acceleration_mode(
    request_config_id: u32,
    acceleration_mode: Gna2AccelerationMode,
) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.last_accel_mode = Some((request_config_id, acceleration_mode));
        st.set_acceleration_mode_status
    })
}

unsafe extern "C" fn mock_request_config_release(request_config_id: u32) -> Gna2Status {
    MOCK_STATE.with(|s| {
        s.borrow_mut().released_config_ids.push(request_config_id);
    });
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_request_enqueue(
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

unsafe extern "C" fn mock_request_wait(request_id: u32, timeout_milliseconds: u32) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.last_wait_args = Some((request_id, timeout_milliseconds));
        st.wait_status
    })
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

fn empty_symbols() -> GnaSymbolTable {
    GnaSymbolTable {
        device_get_count: stub_device_get_count,
        device_get_version: stub_device_get_version,
        device_open: stub_device_open,
        device_create_for_export: None,
        device_close: stub_device_close,
        device_set_number_of_threads: None,
        get_library_version: None,
        memory_alloc: stub_memory_alloc,
        memory_alloc_for_device: None,
        memory_free: stub_memory_free,
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

fn full_symbols() -> GnaSymbolTable {
    GnaSymbolTable {
        request_config_create: Some(mock_request_config_create),
        request_config_set_operand_buffer: Some(mock_request_config_set_operand_buffer),
        request_config_enable_active_list: Some(mock_request_config_enable_active_list),
        request_config_enable_hardware_consistency: Some(
            mock_request_config_enable_hardware_consistency,
        ),
        request_config_set_acceleration_mode: Some(mock_request_config_set_acceleration_mode),
        request_config_release: Some(mock_request_config_release),
        request_enqueue: Some(mock_request_enqueue),
        request_wait: Some(mock_request_wait),
        instrumentation_config_create: Some(mock_inst_config_create),
        instrumentation_config_assign_to_request_config: Some(mock_inst_assign),
        instrumentation_config_set_unit: Some(mock_inst_set_unit),
        instrumentation_config_set_mode: Some(mock_inst_set_mode),
        instrumentation_config_release: Some(mock_inst_release),
        ..empty_symbols()
    }
}

#[test]
fn test_request_config_full_lifecycle_and_methods() {
    reset_mock_state();
    let lib = GnaLibrary::from_symbol_table(full_symbols());

    let mut cfg = GnaRequestConfig::create(&lib, 7).expect("create should succeed");
    assert_eq!(cfg.id(), 10);
    assert!(cfg.instrumentation().is_none());

    // Calling hw usage / stats before instrumentation is configured must error
    assert!(matches!(cfg.get_hw_usage(), Err(GnaError::Other(_))));
    assert!(matches!(
        cfg.get_performance_stats(),
        Err(GnaError::Other(_))
    ));

    let mut buffer = [1u8; 16];
    let buf_ptr = buffer.as_mut_ptr().cast::<c_void>();
    unsafe { cfg.set_operand_buffer(1, 2, buf_ptr) }.expect("set_operand_buffer should succeed");

    cfg.enable_active_list(3, &[0, 2, 5])
        .expect("enable_active_list should succeed");
    cfg.enable_hardware_consistency(Gna2DeviceVersion::V3_0)
        .expect("enable_hardware_consistency should succeed");
    cfg.set_acceleration_mode(Gna2AccelerationMode::Hardware)
        .expect("set_acceleration_mode should succeed");
    cfg.enable_performance_counter()
        .expect("enable_performance_counter should succeed");

    assert!(cfg.instrumentation().is_some());

    let req_id = cfg.enqueue().expect("enqueue should succeed");
    assert_eq!(req_id, 100);

    cfg.wait_request(req_id, 500)
        .expect("wait_request should succeed");
    MOCK_STATE.with(|s| {
        assert_eq!(s.borrow().last_wait_args, Some((100, 500)));
    });

    cfg.wait(req_id, 250).expect("wait should succeed");

    let hw_usage = cfg.get_hw_usage().expect("get_hw_usage should succeed");
    assert!((hw_usage - 0.75).abs() < 1e-6);

    let stats = cfg
        .get_performance_stats()
        .expect("get_performance_stats should succeed");
    assert_eq!(stats.total_cycles, 2000);
    assert_eq!(stats.stall_cycles, 500);
    assert_eq!(stats.active_cycles, 1500);
    assert!((stats.hw_usage_ratio - 0.75).abs() < 1e-6);
    assert_eq!(stats.execution_time, Some(80));

    MOCK_STATE.with(|s| {
        let st = s.borrow();
        assert_eq!(st.last_model_id, Some(7));
        assert_eq!(st.last_operand_buf, Some((10, 1, 2, buf_ptr as usize)));
        assert_eq!(st.last_active_list, Some((10, 3, vec![0, 2, 5])));
        assert_eq!(st.last_hw_consistency, Some((10, Gna2DeviceVersion::V3_0)));
        assert_eq!(
            st.last_accel_mode,
            Some((10, Gna2AccelerationMode::Hardware))
        );
        assert_eq!(st.last_enqueue_config_id, Some(10));
        assert_eq!(st.last_wait_args, Some((100, 250)));
    });

    drop(cfg);

    MOCK_STATE.with(|s| {
        let st = s.borrow();
        assert_eq!(st.released_config_ids, vec![10]);
        assert_eq!(st.released_inst_ids, vec![50]);
    });
}

#[test]
fn test_request_config_attach_instrumentation() {
    reset_mock_state();
    let lib = GnaLibrary::from_symbol_table(full_symbols());
    let mut cfg = GnaRequestConfig::create(&lib, 1).unwrap();

    let inst = GnaInstrumentationConfig::create(
        &lib,
        &[
            Gna2InstrumentationPoint::HwTotalCycles,
            Gna2InstrumentationPoint::HwStallCycles,
        ],
    )
    .unwrap();

    cfg.attach_instrumentation(inst)
        .expect("attach_instrumentation should succeed");
    assert!(cfg.instrumentation().is_some());
    assert!((cfg.get_hw_usage().unwrap() - 0.75).abs() < 1e-6);

    // Attach failure should propagate error and leave existing instrumentation untouched
    MOCK_STATE.with(|s| s.borrow_mut().inst_assign_status = -5);
    let inst_fail =
        GnaInstrumentationConfig::create(&lib, &[Gna2InstrumentationPoint::HwTotalCycles]).unwrap();
    let err = cfg.attach_instrumentation(inst_fail).unwrap_err();
    assert!(matches!(err, GnaError::StatusError { status: -5, .. }));
}

#[test]
fn test_request_config_enable_performance_counter_branches() {
    reset_mock_state();

    // Case 1: set_unit missing or failing is ignored by enable_performance_counter
    let mut syms_no_unit = full_symbols();
    syms_no_unit.instrumentation_config_set_unit = None;
    let lib_no_unit = GnaLibrary::from_symbol_table(syms_no_unit);
    let mut cfg = GnaRequestConfig::create(&lib_no_unit, 1).unwrap();
    cfg.enable_performance_counter()
        .expect("missing set_unit should be tolerated");

    // Case 2: assign_to_request_config missing -> Err
    let mut syms_no_assign = full_symbols();
    syms_no_assign.instrumentation_config_assign_to_request_config = None;
    let lib_no_assign = GnaLibrary::from_symbol_table(syms_no_assign);
    let mut cfg_no_assign = GnaRequestConfig::create(&lib_no_assign, 1).unwrap();
    assert!(matches!(
        cfg_no_assign.enable_performance_counter(),
        Err(GnaError::Other(_))
    ));

    // Case 3: instrumentation_config_create missing -> Err
    let mut syms_no_inst_create = full_symbols();
    syms_no_inst_create.instrumentation_config_create = None;
    let lib_no_inst = GnaLibrary::from_symbol_table(syms_no_inst_create);
    let mut cfg_no_inst = GnaRequestConfig::create(&lib_no_inst, 1).unwrap();
    assert!(matches!(
        cfg_no_inst.enable_performance_counter(),
        Err(GnaError::Other(_))
    ));

    // Case 4: assign returns error status -> Err
    let lib_full = GnaLibrary::from_symbol_table(full_symbols());
    let mut cfg_err = GnaRequestConfig::create(&lib_full, 1).unwrap();
    MOCK_STATE.with(|s| s.borrow_mut().inst_assign_status = -8);
    assert!(matches!(
        cfg_err.enable_performance_counter(),
        Err(GnaError::StatusError { status: -8, .. })
    ));
}

#[test]
fn test_request_config_missing_symbols() {
    reset_mock_state();

    // Missing create
    let lib_empty = GnaLibrary::from_symbol_table(empty_symbols());
    assert!(matches!(
        GnaRequestConfig::create(&lib_empty, 0),
        Err(GnaError::Other(_))
    ));

    // Has create and release only -> all other methods report missing symbol
    let syms_create_only = GnaSymbolTable {
        request_config_create: Some(mock_request_config_create),
        ..empty_symbols()
    };
    let lib = GnaLibrary::from_symbol_table(syms_create_only);
    let mut cfg = GnaRequestConfig::create(&lib, 0).unwrap();

    let mut buf = [0u8; 4];
    assert!(matches!(
        unsafe { cfg.set_operand_buffer(0, 0, buf.as_mut_ptr().cast()) },
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        cfg.enable_active_list(0, &[1]),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        cfg.enable_hardware_consistency(Gna2DeviceVersion::V3_0),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        cfg.set_acceleration_mode(Gna2AccelerationMode::Software),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(cfg.enqueue(), Err(GnaError::Other(_))));
    assert!(matches!(cfg.wait_request(1, 10), Err(GnaError::Other(_))));
    assert!(matches!(cfg.wait(1, 10), Err(GnaError::Other(_))));

    // Drop without request_config_release should be a safe no-op
    drop(cfg);
    MOCK_STATE.with(|s| {
        assert!(s.borrow().released_config_ids.is_empty());
    });
}

#[test]
fn test_request_config_error_statuses_and_inst_errors() {
    reset_mock_state();
    let lib = GnaLibrary::from_symbol_table(full_symbols());

    MOCK_STATE.with(|s| s.borrow_mut().create_status = -1);
    assert!(matches!(
        GnaRequestConfig::create(&lib, 2),
        Err(GnaError::StatusError { status: -1, .. })
    ));

    reset_mock_state();
    let mut cfg = GnaRequestConfig::create(&lib, 2).unwrap();

    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.set_operand_buffer_status = -6;
        st.enable_active_list_status = -5;
        st.enable_hardware_consistency_status = -11;
        st.set_acceleration_mode_status = -4;
        st.enqueue_status = -12;
        st.wait_status = -13;
    });

    let mut buf = [0u8; 4];
    assert!(matches!(
        unsafe { cfg.set_operand_buffer(0, 0, buf.as_mut_ptr().cast()) },
        Err(GnaError::StatusError { status: -6, .. })
    ));
    assert!(matches!(
        cfg.enable_active_list(0, &[0]),
        Err(GnaError::StatusError { status: -5, .. })
    ));
    assert!(matches!(
        cfg.enable_hardware_consistency(Gna2DeviceVersion::V2_0),
        Err(GnaError::StatusError { status: -11, .. })
    ));
    assert!(matches!(
        cfg.set_acceleration_mode(Gna2AccelerationMode::Hardware),
        Err(GnaError::StatusError { status: -4, .. })
    ));
    assert!(matches!(
        cfg.enqueue(),
        Err(GnaError::StatusError { status: -12, .. })
    ));
    assert!(matches!(
        cfg.wait_request(1, 10),
        Err(GnaError::StatusError { status: -13, .. })
    ));
    assert!(matches!(
        cfg.wait(1, 10),
        Err(GnaError::StatusError { status: -13, .. })
    ));

    // Propagate instrumentation errors through get_hw_usage / get_performance_stats
    let inst_missing_points =
        GnaInstrumentationConfig::create(&lib, &[Gna2InstrumentationPoint::LibExecution]).unwrap();
    cfg.attach_instrumentation(inst_missing_points).unwrap();
    assert!(matches!(cfg.get_hw_usage(), Err(GnaError::Other(_))));
    assert!(matches!(
        cfg.get_performance_stats(),
        Err(GnaError::Other(_))
    ));
}

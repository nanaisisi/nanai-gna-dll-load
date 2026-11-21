use std::cell::RefCell;
use std::ffi::c_void;

use nanai_gna_dll_load::error::GnaError;
use nanai_gna_dll_load::instrumentation::{
    GnaInstrumentationConfig, GnaPerformanceStats, GnaUsageMonitor,
};
use nanai_gna_dll_load::loader::GnaLibrary;
use nanai_gna_dll_load::symbol_table::GnaSymbolTable;
use nanai_gna_dll_load::types::{
    GNA2_STATUS_SUCCESS, Gna2DeviceVersion, Gna2InstrumentationMode, Gna2InstrumentationPoint,
    Gna2InstrumentationUnit, Gna2Status,
};

#[derive(Default)]
struct MockInstState {
    next_id: u32,
    create_status: Gna2Status,
    assign_status: Gna2Status,
    set_unit_status: Gna2Status,
    set_mode_status: Gna2Status,

    total_cycles: u64,
    stall_cycles: u64,
    exec_time: u64,
    other_point_value: u64,

    last_assign: Option<(u32, u32)>,
    last_unit: Option<(u32, Gna2InstrumentationUnit)>,
    last_mode: Option<(u32, Gna2InstrumentationMode)>,
    released_ids: Vec<u32>,
}

thread_local! {
    static MOCK_INST_STATE: RefCell<MockInstState> = RefCell::new(MockInstState::default());
}

fn reset_mock_inst_state() {
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

unsafe extern "C" fn mock_inst_create(
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

fn full_inst_symbols() -> GnaSymbolTable {
    GnaSymbolTable {
        instrumentation_config_create: Some(mock_inst_create),
        instrumentation_config_assign_to_request_config: Some(mock_inst_assign),
        instrumentation_config_set_unit: Some(mock_inst_set_unit),
        instrumentation_config_set_mode: Some(mock_inst_set_mode),
        instrumentation_config_release: Some(mock_inst_release),
        ..empty_symbols()
    }
}

#[test]
fn test_performance_stats_calculation() {
    let stats = GnaPerformanceStats {
        total_cycles: 10_000,
        stall_cycles: 2_500,
        active_cycles: 7_500,
        hw_usage_ratio: 0.75,
        execution_time: Some(150),
    };

    assert_eq!(stats.total_cycles, 10_000);
    assert_eq!(stats.stall_cycles, 2_500);
    assert_eq!(stats.active_cycles, 7_500);
    assert!((stats.hw_usage_ratio - 0.75).abs() < 1e-6);
    assert!((stats.hw_usage_percentage() - 75.0).abs() < 1e-6);
    assert_eq!(stats.execution_time, Some(150));
}

#[test]
fn test_usage_monitor_aggregation() {
    let mut monitor = GnaUsageMonitor::new();
    assert_eq!(monitor.inference_count(), 0);
    assert_eq!(monitor.cumulative_hw_usage_ratio(), 0.0);
    assert_eq!(monitor.average_execution_time(), None);

    let run1 = GnaPerformanceStats {
        total_cycles: 1000,
        stall_cycles: 200,
        active_cycles: 800,
        hw_usage_ratio: 0.8,
        execution_time: Some(50),
    };
    monitor.record(&run1);

    assert_eq!(monitor.inference_count(), 1);
    assert_eq!(monitor.cumulative_total_cycles(), 1000);
    assert_eq!(monitor.cumulative_stall_cycles(), 200);
    assert_eq!(monitor.cumulative_active_cycles(), 800);
    assert!((monitor.cumulative_hw_usage_ratio() - 0.8).abs() < 1e-6);
    assert!((monitor.cumulative_hw_usage_percentage() - 80.0).abs() < 1e-6);
    assert_eq!(monitor.average_execution_time(), Some(50.0));
    assert_eq!(monitor.last_stats(), Some(&run1));

    let run2 = GnaPerformanceStats {
        total_cycles: 3000,
        stall_cycles: 1800,
        active_cycles: 1200,
        hw_usage_ratio: 0.4,
        execution_time: Some(150),
    };
    monitor.record(&run2);

    assert_eq!(monitor.inference_count(), 2);
    // Weighted cycles: total = 4000, stall = 2000, active = 2000
    assert_eq!(monitor.cumulative_total_cycles(), 4000);
    assert_eq!(monitor.cumulative_stall_cycles(), 2000);
    assert_eq!(monitor.cumulative_active_cycles(), 2000);
    // Weighted usage: 2000 / 4000 = 0.5 (50.0%)
    assert!((monitor.cumulative_hw_usage_ratio() - 0.5).abs() < 1e-6);
    assert!((monitor.cumulative_hw_usage_percentage() - 50.0).abs() < 1e-6);
    // Average execution time: (50 + 150) / 2 = 100.0
    assert_eq!(monitor.average_execution_time(), Some(100.0));
    assert_eq!(monitor.last_stats(), Some(&run2));

    // Test reset
    monitor.reset();
    assert_eq!(monitor.inference_count(), 0);
    assert_eq!(monitor.cumulative_total_cycles(), 0);
    assert_eq!(monitor.cumulative_hw_usage_ratio(), 0.0);
    assert_eq!(monitor.average_execution_time(), None);
    assert_eq!(monitor.last_stats(), None);
}

#[test]
fn test_instrumentation_create_validation_and_missing_symbol() {
    reset_mock_inst_state();
    let lib = GnaLibrary::from_symbol_table(full_inst_symbols());

    // Empty points slice
    assert!(matches!(
        GnaInstrumentationConfig::create(&lib, &[]),
        Err(GnaError::Other(_))
    ));

    // Too many points (> 15)
    let too_many = [Gna2InstrumentationPoint::HwTotalCycles; 16];
    assert!(matches!(
        GnaInstrumentationConfig::create(&lib, &too_many),
        Err(GnaError::Other(_))
    ));

    // Max valid count (15)
    let max_points = [Gna2InstrumentationPoint::HwTotalCycles; 15];
    let inst_15 = GnaInstrumentationConfig::create(&lib, &max_points)
        .expect("15 instrumentation points should be valid");
    assert_eq!(inst_15.points().len(), 15);
    assert_eq!(inst_15.results().len(), 15);
    drop(inst_15);

    // Missing create symbol
    let empty_lib = GnaLibrary::from_symbol_table(empty_symbols());
    assert!(matches!(
        GnaInstrumentationConfig::create(&empty_lib, &[Gna2InstrumentationPoint::HwTotalCycles]),
        Err(GnaError::Other(_))
    ));

    // Non-success status from create
    MOCK_INST_STATE.with(|s| s.borrow_mut().create_status = -8);
    assert!(matches!(
        GnaInstrumentationConfig::create(&lib, &[Gna2InstrumentationPoint::HwTotalCycles]),
        Err(GnaError::StatusError { status: -8, .. })
    ));
}

#[test]
fn test_instrumentation_config_methods_success_and_drop() {
    reset_mock_inst_state();
    let lib = GnaLibrary::from_symbol_table(full_inst_symbols());

    let points = [
        Gna2InstrumentationPoint::LibPreprocessing,
        Gna2InstrumentationPoint::HwTotalCycles,
        Gna2InstrumentationPoint::HwStallCycles,
        Gna2InstrumentationPoint::LibExecution,
    ];
    let mut inst = GnaInstrumentationConfig::create(&lib, &points).expect("create should succeed");
    let inst_id = inst.id();
    assert_eq!(inst_id, 1);
    assert_eq!(inst.points(), &points);
    assert_eq!(inst.results(), &[7, 1000, 250, 42]);

    inst.assign_to_request_config(99)
        .expect("assign_to_request_config should succeed");
    inst.set_unit(Gna2InstrumentationUnit::Cycles)
        .expect("set_unit should succeed");
    inst.set_mode(Gna2InstrumentationMode::WaitForDmaCompletion)
        .expect("set_mode should succeed");

    let hw_usage = inst.compute_hw_usage().expect("hw usage should succeed");
    assert!((hw_usage - 0.75).abs() < 1e-6);

    let stats = inst
        .compute_performance_stats()
        .expect("compute_performance_stats should succeed");
    assert_eq!(stats.total_cycles, 1000);
    assert_eq!(stats.stall_cycles, 250);
    assert_eq!(stats.active_cycles, 750);
    assert!((stats.hw_usage_ratio - 0.75).abs() < 1e-6);
    assert_eq!(stats.execution_time, Some(42));

    MOCK_INST_STATE.with(|s| {
        let st = s.borrow();
        assert_eq!(st.last_assign, Some((inst_id, 99)));
        assert_eq!(
            st.last_unit,
            Some((inst_id, Gna2InstrumentationUnit::Cycles))
        );
        assert_eq!(
            st.last_mode,
            Some((inst_id, Gna2InstrumentationMode::WaitForDmaCompletion))
        );
    });

    drop(inst);

    MOCK_INST_STATE.with(|s| {
        assert_eq!(s.borrow().released_ids, vec![inst_id]);
    });
}

#[test]
fn test_instrumentation_config_missing_symbols_and_errors() {
    reset_mock_inst_state();

    // Create-only symbol table: assign, set_unit, set_mode, release are all None
    let create_only = GnaSymbolTable {
        instrumentation_config_create: Some(mock_inst_create),
        ..empty_symbols()
    };
    let lib_create_only = GnaLibrary::from_symbol_table(create_only);
    let mut inst_missing = GnaInstrumentationConfig::create(
        &lib_create_only,
        &[Gna2InstrumentationPoint::HwTotalCycles],
    )
    .unwrap();

    assert!(matches!(
        inst_missing.assign_to_request_config(1),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        inst_missing.set_unit(Gna2InstrumentationUnit::Microseconds),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        inst_missing.set_mode(Gna2InstrumentationMode::TotalStall),
        Err(GnaError::Other(_))
    ));

    // Drop without release symbol should safely do nothing
    drop(inst_missing);
    MOCK_INST_STATE.with(|s| assert!(s.borrow().released_ids.is_empty()));

    // Error status paths for assign, set_unit, set_mode
    let lib_full = GnaLibrary::from_symbol_table(full_inst_symbols());
    let mut inst_err =
        GnaInstrumentationConfig::create(&lib_full, &[Gna2InstrumentationPoint::HwTotalCycles])
            .unwrap();

    MOCK_INST_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.assign_status = -5;
        st.set_unit_status = -5;
        st.set_mode_status = -5;
    });

    assert!(matches!(
        inst_err.assign_to_request_config(1),
        Err(GnaError::StatusError { status: -5, .. })
    ));
    assert!(matches!(
        inst_err.set_unit(Gna2InstrumentationUnit::Milliseconds),
        Err(GnaError::StatusError { status: -5, .. })
    ));
    assert!(matches!(
        inst_err.set_mode(Gna2InstrumentationMode::Disabled),
        Err(GnaError::StatusError { status: -5, .. })
    ));
}

#[test]
fn test_instrumentation_edge_cases_zero_cycles_stall_gt_total_and_missing_points() {
    reset_mock_inst_state();
    let lib = GnaLibrary::from_symbol_table(full_inst_symbols());

    // Missing HwTotalCycles
    let inst_no_total =
        GnaInstrumentationConfig::create(&lib, &[Gna2InstrumentationPoint::HwStallCycles]).unwrap();
    assert!(matches!(
        inst_no_total.compute_hw_usage(),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        inst_no_total.compute_performance_stats(),
        Err(GnaError::Other(_))
    ));

    // Missing HwStallCycles
    let inst_no_stall =
        GnaInstrumentationConfig::create(&lib, &[Gna2InstrumentationPoint::HwTotalCycles]).unwrap();
    assert!(matches!(
        inst_no_stall.compute_hw_usage(),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        inst_no_stall.compute_performance_stats(),
        Err(GnaError::Other(_))
    ));

    // Zero total cycles and no LibExecution point
    MOCK_INST_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.total_cycles = 0;
        st.stall_cycles = 0;
    });
    let inst_zero = GnaInstrumentationConfig::create(
        &lib,
        &[
            Gna2InstrumentationPoint::HwTotalCycles,
            Gna2InstrumentationPoint::HwStallCycles,
        ],
    )
    .unwrap();
    assert!(matches!(
        inst_zero.compute_hw_usage(),
        Err(GnaError::Other(_))
    ));
    let zero_stats = inst_zero
        .compute_performance_stats()
        .expect("compute_performance_stats should succeed on zero cycles");
    assert_eq!(zero_stats.total_cycles, 0);
    assert_eq!(zero_stats.stall_cycles, 0);
    assert_eq!(zero_stats.active_cycles, 0);
    assert_eq!(zero_stats.hw_usage_ratio, 0.0);
    assert_eq!(zero_stats.execution_time, None);

    // Stall cycles greater than total cycles -> saturating_sub clamps to 0 active cycles
    MOCK_INST_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.total_cycles = 100;
        st.stall_cycles = 250;
    });
    let inst_stall_gt = GnaInstrumentationConfig::create(
        &lib,
        &[
            Gna2InstrumentationPoint::HwTotalCycles,
            Gna2InstrumentationPoint::HwStallCycles,
        ],
    )
    .unwrap();
    assert_eq!(inst_stall_gt.compute_hw_usage().unwrap(), 0.0);
    let stall_stats = inst_stall_gt.compute_performance_stats().unwrap();
    assert_eq!(stall_stats.total_cycles, 100);
    assert_eq!(stall_stats.stall_cycles, 250);
    assert_eq!(stall_stats.active_cycles, 0);
    assert_eq!(stall_stats.hw_usage_ratio, 0.0);
    assert_eq!(stall_stats.execution_time, None);
}

#[test]
fn test_usage_monitor_without_exec_time() {
    let mut monitor = GnaUsageMonitor::new();
    let stats = GnaPerformanceStats {
        total_cycles: 500,
        stall_cycles: 100,
        active_cycles: 400,
        hw_usage_ratio: 0.8,
        execution_time: None,
    };
    monitor.record(&stats);

    assert_eq!(monitor.inference_count(), 1);
    assert_eq!(monitor.cumulative_total_cycles(), 500);
    assert_eq!(monitor.cumulative_stall_cycles(), 100);
    assert_eq!(monitor.cumulative_active_cycles(), 400);
    assert!((monitor.cumulative_hw_usage_ratio() - 0.8).abs() < 1e-6);
    assert_eq!(monitor.average_execution_time(), None);
}

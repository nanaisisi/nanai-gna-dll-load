use std::cell::RefCell;
use std::ffi::c_void;
use std::time::Duration;

use nanai_gna_dll_load::{
    error::GnaError,
    inference::GnaRequestConfig,
    instrumentation::{GnaInstrumentationConfig, GnaUsageMonitor},
    loader::GnaLibrary,
    stress::{GnaLoadTestConfig, GnaLoadTestReport, GnaLoadTester},
    symbol_table::GnaSymbolTable,
    types::{
        GNA2_STATUS_SUCCESS, Gna2DeviceVersion, Gna2InstrumentationMode, Gna2InstrumentationPoint,
        Gna2InstrumentationUnit, Gna2Status,
    },
};

#[derive(Default)]
struct MockStressState {
    next_req_id: u32,
    enqueue_calls: usize,
    wait_calls: usize,

    fail_enqueue_on_call: Option<usize>,
    enqueue_delays_ms: Vec<(usize, u64)>,

    fail_wait_on_call: Option<usize>,
    wait_delays_ms: Vec<(usize, u64)>,

    total_cycles: u64,
    stall_cycles: u64,
    exec_time: u64,
}

thread_local! {
    static MOCK_STRESS: RefCell<MockStressState> = RefCell::new(MockStressState::default());
}

fn reset_stress_state() {
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

fn full_stress_symbols() -> GnaSymbolTable {
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
        request_config_create: Some(mock_req_create),
        request_config_set_operand_buffer: None,
        request_config_enable_active_list: None,
        request_config_enable_hardware_consistency: None,
        request_config_set_acceleration_mode: None,
        request_config_release: Some(mock_req_release),
        request_enqueue: Some(mock_enqueue),
        request_wait: Some(mock_wait),
        instrumentation_config_create: Some(mock_inst_create),
        instrumentation_config_assign_to_request_config: Some(mock_inst_assign),
        instrumentation_config_set_unit: Some(mock_inst_set_unit),
        instrumentation_config_set_mode: Some(mock_inst_set_mode),
        instrumentation_config_release: Some(mock_inst_release),
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

#[test]
fn test_load_test_config_builder() {
    let config = GnaLoadTestConfig::new()
        .with_iterations(500)
        .with_concurrency(4)
        .with_duration(Duration::from_secs(10))
        .with_timeout_ms(2000)
        .with_track_hw_usage(false);

    assert_eq!(config.iterations, Some(500));
    assert_eq!(config.concurrency, 4);
    assert_eq!(config.duration, Some(Duration::from_secs(10)));
    assert_eq!(config.timeout_per_request_ms, 2000);
    assert!(!config.track_hw_usage);
}

#[test]
fn test_load_test_report_metrics() {
    let report = GnaLoadTestReport {
        completed_inferences: 1000,
        failed_inferences: 0,
        elapsed_time: Duration::from_millis(500),
        inferences_per_second: 2000.0,
        latency_min: Duration::from_micros(400),
        latency_max: Duration::from_millis(2),
        latency_avg: Duration::from_micros(500),
        usage_monitor: GnaUsageMonitor::new(),
    };

    assert_eq!(report.completed_inferences, 1000);
    assert_eq!(report.failed_inferences, 0);
    assert!((report.inferences_per_second - 2000.0).abs() < 1e-6);
}

#[test]
fn test_load_test_config_defaults_and_clamping() {
    let def = GnaLoadTestConfig::default();
    assert_eq!(def.iterations, Some(1000));
    assert_eq!(def.duration, None);
    assert_eq!(def.concurrency, 1);
    assert_eq!(def.timeout_per_request_ms, 1000);
    assert!(def.track_hw_usage);

    let clamped = GnaLoadTestConfig::new().with_concurrency(0);
    assert_eq!(clamped.concurrency, 1);
}

#[test]
fn test_load_tester_iterations_mode_single_and_multi_concurrency() {
    reset_stress_state();
    // Vary wait latencies so both latency_min and latency_max branches update
    MOCK_STRESS.with(|s| {
        s.borrow_mut().wait_delays_ms = vec![(1, 5), (3, 10)];
    });

    let lib = GnaLibrary::from_symbol_table(full_stress_symbols());
    let mut req_cfg = GnaRequestConfig::create(&lib, 1).unwrap();

    // Concurrency = 1, track_hw_usage = true (auto-enables instrumentation)
    let tester_seq = GnaLoadTester::new(
        GnaLoadTestConfig::new()
            .with_iterations(4)
            .with_concurrency(1)
            .with_track_hw_usage(true),
    );
    let report_seq = tester_seq
        .run(&mut req_cfg)
        .expect("seq run should succeed");
    assert_eq!(report_seq.completed_inferences, 4);
    assert_eq!(report_seq.failed_inferences, 0);
    assert!(report_seq.inferences_per_second > 0.0);
    assert!(report_seq.latency_min <= report_seq.latency_avg);
    assert!(report_seq.latency_avg <= report_seq.latency_max);
    assert_eq!(report_seq.usage_monitor.inference_count(), 4);
    assert_eq!(report_seq.usage_monitor.cumulative_total_cycles(), 4000);
    assert_eq!(
        report_seq.usage_monitor.average_execution_time(),
        Some(40.0)
    );
    report_seq.print_summary();

    // Concurrency > 1, instrumentation already attached on req_cfg
    reset_stress_state();
    let tester_multi = GnaLoadTester::new(
        GnaLoadTestConfig::new()
            .with_iterations(7)
            .with_concurrency(3)
            .with_track_hw_usage(true),
    );
    let report_multi = tester_multi
        .run(&mut req_cfg)
        .expect("multi-concurrency run should succeed");
    assert_eq!(report_multi.completed_inferences, 7);
    assert_eq!(report_multi.failed_inferences, 0);
    assert_eq!(report_multi.usage_monitor.inference_count(), 7);
}

#[test]
fn test_load_tester_track_hw_usage_false_and_no_exec_time_summary() {
    reset_stress_state();
    let lib = GnaLibrary::from_symbol_table(full_stress_symbols());
    let mut req_cfg = GnaRequestConfig::create(&lib, 1).unwrap();

    // track_hw_usage = false -> usage_monitor stays empty; print_summary skips HW section
    let tester_no_hw = GnaLoadTester::new(
        GnaLoadTestConfig::new()
            .with_iterations(4)
            .with_concurrency(2)
            .with_track_hw_usage(false),
    );
    let report_no_hw = tester_no_hw.run(&mut req_cfg).unwrap();
    assert_eq!(report_no_hw.completed_inferences, 4);
    assert_eq!(report_no_hw.usage_monitor.inference_count(), 0);
    report_no_hw.print_summary();

    // Pre-attached instrumentation without LibExecution -> print_summary shows HW metrics without Avg Hardware Time
    let inst_no_exec = GnaInstrumentationConfig::create(
        &lib,
        &[
            Gna2InstrumentationPoint::HwTotalCycles,
            Gna2InstrumentationPoint::HwStallCycles,
        ],
    )
    .unwrap();
    req_cfg.attach_instrumentation(inst_no_exec).unwrap();

    let tester_no_exec = GnaLoadTester::new(
        GnaLoadTestConfig::new()
            .with_iterations(2)
            .with_concurrency(1)
            .with_track_hw_usage(true),
    );
    let report_no_exec = tester_no_exec.run(&mut req_cfg).unwrap();
    assert_eq!(report_no_exec.usage_monitor.inference_count(), 2);
    assert_eq!(report_no_exec.usage_monitor.average_execution_time(), None);
    report_no_exec.print_summary();

    // Missing instrumentation symbols with track_hw_usage = true -> falls back gracefully
    let mut syms_no_inst = full_stress_symbols();
    syms_no_inst.instrumentation_config_create = None;
    let lib_no_inst = GnaLibrary::from_symbol_table(syms_no_inst);
    let mut req_no_inst = GnaRequestConfig::create(&lib_no_inst, 1).unwrap();
    let tester_missing_inst = GnaLoadTester::new(
        GnaLoadTestConfig::new()
            .with_iterations(3)
            .with_concurrency(1)
            .with_track_hw_usage(true),
    );
    let report_missing_inst = tester_missing_inst.run(&mut req_no_inst).unwrap();
    assert_eq!(report_missing_inst.completed_inferences, 3);
    assert_eq!(report_missing_inst.usage_monitor.inference_count(), 0);
}

#[test]
fn test_load_tester_duration_mode_and_drain() {
    let lib = GnaLibrary::from_symbol_table(full_stress_symbols());
    let mut req_cfg = GnaRequestConfig::create(&lib, 1).unwrap();

    // Pure duration mode (iterations = None, concurrency = 1)
    reset_stress_state();
    let cfg_pure_dur = GnaLoadTestConfig {
        iterations: None,
        duration: Some(Duration::from_millis(25)),
        concurrency: 1,
        timeout_per_request_ms: 500,
        track_hw_usage: true,
    };
    let report_dur = GnaLoadTester::new(cfg_pure_dur).run(&mut req_cfg).unwrap();
    assert!(report_dur.completed_inferences >= 1);
    assert_eq!(report_dur.failed_inferences, 0);

    // Trigger duration break inside inner enqueue loop (enqueue #1 sleeps past duration)
    reset_stress_state();
    MOCK_STRESS.with(|s| {
        s.borrow_mut().enqueue_delays_ms = vec![(1, 35)];
    });
    let cfg_inner_break = GnaLoadTestConfig {
        iterations: None,
        duration: Some(Duration::from_millis(20)),
        concurrency: 3,
        timeout_per_request_ms: 500,
        track_hw_usage: false,
    };
    let report_inner = GnaLoadTester::new(cfg_inner_break)
        .run(&mut req_cfg)
        .unwrap();
    assert_eq!(report_inner.completed_inferences, 1);

    // Deterministic drain loop covering Ok (min & max latency updates) and Err paths
    reset_stress_state();
    MOCK_STRESS.with(|s| {
        let mut st = s.borrow_mut();
        st.enqueue_delays_ms = vec![(2, 12)];
        st.wait_delays_ms = vec![(1, 30), (3, 20)];
        st.fail_wait_on_call = Some(4);
    });
    let cfg_drain = GnaLoadTestConfig {
        iterations: None,
        duration: Some(Duration::from_millis(25)),
        concurrency: 4,
        timeout_per_request_ms: 500,
        track_hw_usage: true,
    };
    let report_drain = GnaLoadTester::new(cfg_drain).run(&mut req_cfg).unwrap();
    assert_eq!(report_drain.completed_inferences, 3);
    assert_eq!(report_drain.failed_inferences, 1);
    assert_eq!(report_drain.usage_monitor.inference_count(), 3);

    // Zero / tiny duration exits immediately with empty report
    reset_stress_state();
    let cfg_zero_dur = GnaLoadTestConfig {
        iterations: None,
        duration: Some(Duration::ZERO),
        concurrency: 1,
        timeout_per_request_ms: 100,
        track_hw_usage: false,
    };
    let report_zero_dur = GnaLoadTester::new(cfg_zero_dur).run(&mut req_cfg).unwrap();
    assert_eq!(report_zero_dur.completed_inferences, 0);
    assert_eq!(report_zero_dur.latency_min, Duration::ZERO);
    assert_eq!(report_zero_dur.latency_avg, Duration::ZERO);
}

#[test]
fn test_load_tester_zero_iterations_and_errors() {
    let lib = GnaLibrary::from_symbol_table(full_stress_symbols());
    let mut req_cfg = GnaRequestConfig::create(&lib, 1).unwrap();

    // Zero iterations
    reset_stress_state();
    let report_zero = GnaLoadTester::new(
        GnaLoadTestConfig::new()
            .with_iterations(0)
            .with_track_hw_usage(false),
    )
    .run(&mut req_cfg)
    .unwrap();
    assert_eq!(report_zero.completed_inferences, 0);
    assert_eq!(report_zero.failed_inferences, 0);
    assert_eq!(report_zero.latency_min, Duration::ZERO);
    assert_eq!(report_zero.latency_avg, Duration::ZERO);

    // Immediate enqueue failure with empty in_flight queue -> returns Err
    reset_stress_state();
    MOCK_STRESS.with(|s| {
        s.borrow_mut().fail_enqueue_on_call = Some(1);
    });
    let err = GnaLoadTester::new(
        GnaLoadTestConfig::new()
            .with_iterations(3)
            .with_track_hw_usage(false),
    )
    .run(&mut req_cfg)
    .unwrap_err();
    assert!(matches!(err, GnaError::Other(_)));

    // Enqueue failure while in_flight is non-empty -> records failure, breaks inner loop, finishes queued work
    reset_stress_state();
    MOCK_STRESS.with(|s| {
        s.borrow_mut().fail_enqueue_on_call = Some(2);
    });
    let report_partial_enqueue = GnaLoadTester::new(
        GnaLoadTestConfig::new()
            .with_iterations(2)
            .with_concurrency(2)
            .with_track_hw_usage(false),
    )
    .run(&mut req_cfg)
    .unwrap();
    assert_eq!(report_partial_enqueue.completed_inferences, 1);
    assert_eq!(report_partial_enqueue.failed_inferences, 1);

    // Wait failure in main loop -> counts failed inference and continues
    reset_stress_state();
    MOCK_STRESS.with(|s| {
        s.borrow_mut().fail_wait_on_call = Some(2);
    });
    let report_wait_err = GnaLoadTester::new(
        GnaLoadTestConfig::new()
            .with_iterations(3)
            .with_concurrency(1)
            .with_track_hw_usage(false),
    )
    .run(&mut req_cfg)
    .unwrap();
    assert_eq!(report_wait_err.completed_inferences, 2);
    assert_eq!(report_wait_err.failed_inferences, 1);
}

#[test]
fn test_load_tester_drain_updates_latency_min_and_max() {
    reset_stress_state();
    MOCK_STRESS.with(|s| {
        let mut st = s.borrow_mut();
        st.wait_delays_ms = vec![(1, 25), (3, 10)];
        st.fail_wait_on_call = Some(1);
    });

    let lib = GnaLibrary::from_symbol_table(full_stress_symbols());
    let mut req_cfg = GnaRequestConfig::create(&lib, 1).unwrap();

    let cfg = GnaLoadTestConfig {
        iterations: None,
        duration: Some(Duration::from_millis(15)),
        concurrency: 3,
        timeout_per_request_ms: 500,
        track_hw_usage: true,
    };
    let report = GnaLoadTester::new(cfg).run(&mut req_cfg).unwrap();

    assert_eq!(report.completed_inferences, 2);
    assert_eq!(report.failed_inferences, 1);
    assert!(report.latency_min > Duration::ZERO);
    assert!(report.latency_min <= report.latency_max);
    assert_eq!(report.usage_monitor.inference_count(), 2);
}

#[test]
fn test_load_tester_duration_boundary_and_zero_elapsed() {
    let lib = GnaLibrary::from_symbol_table(full_stress_symbols());
    let mut req_cfg = GnaRequestConfig::create(&lib, 1).unwrap();

    reset_stress_state();
    let durations_ns: Vec<u64> = (100..=2_000)
        .step_by(100)
        .chain((2_500..=10_000).step_by(500))
        .collect();
    for _ in 0..25 {
        for ns in &durations_ns {
            let report = GnaLoadTester::new(GnaLoadTestConfig {
                iterations: Some(1),
                duration: Some(Duration::from_nanos(*ns)),
                concurrency: 1,
                timeout_per_request_ms: 100,
                track_hw_usage: false,
            })
            .run(&mut req_cfg)
            .unwrap();
            assert!(report.completed_inferences <= 1);
            assert_eq!(report.failed_inferences, 0);
        }
    }

    reset_stress_state();
    let zero_tester = GnaLoadTester::new(
        GnaLoadTestConfig::new()
            .with_iterations(0)
            .with_track_hw_usage(false),
    );
    for _ in 0..2_000 {
        let report = zero_tester.run(&mut req_cfg).unwrap();
        assert_eq!(report.completed_inferences, 0);
        assert_eq!(report.inferences_per_second, 0.0);
        if report.elapsed_time.is_zero() {
            break;
        }
    }
}

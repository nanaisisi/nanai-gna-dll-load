use std::time::Duration;

use nanai_gna_dll_load::{
    inference::GnaRequestConfig,
    instrumentation::GnaInstrumentationConfig,
    loader::GnaLibrary,
    stress::{GnaLoadTestConfig, GnaLoadTester},
    types::Gna2InstrumentationPoint,
};

use super::mock::{MOCK_STRESS, full_stress_symbols, reset_stress_state};

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

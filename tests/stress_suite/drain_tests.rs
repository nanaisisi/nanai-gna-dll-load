use std::time::Duration;

use nanai_gna_dll_load::{
    inference::GnaRequestConfig,
    loader::GnaLibrary,
    stress::{GnaLoadTestConfig, GnaLoadTester},
};

use super::mock::{MOCK_STRESS, full_stress_symbols, reset_stress_state};

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
    for _ in 0..10_000 {
        let report = zero_tester.run(&mut req_cfg).unwrap();
        assert_eq!(report.completed_inferences, 0);
        assert_eq!(report.inferences_per_second, 0.0);
        if report.elapsed_time.is_zero() {
            break;
        }
    }
}

#[test]
fn test_load_tester_drain_latency_below_max() {
    reset_stress_state();
    // Iter 1 enqueues #1 and #2 at t=0, waits #1 (~25ms < 60ms).
    // Iter 2 enqueues #3 at t~25ms, waits #2 (~50ms -> finishes at t~75ms >= 60ms, latency_max ~75ms).
    // Drain waits #3 immediately (lat ~50ms < latency_max ~75ms), hitting lat > latency_max == false.
    MOCK_STRESS.with(|s| {
        s.borrow_mut().wait_delays_ms = vec![(1, 25), (2, 50)];
    });

    let lib = GnaLibrary::from_symbol_table(full_stress_symbols());
    let mut req_cfg = GnaRequestConfig::create(&lib, 1).unwrap();

    let cfg = GnaLoadTestConfig {
        iterations: None,
        duration: Some(Duration::from_millis(60)),
        concurrency: 2,
        timeout_per_request_ms: 500,
        track_hw_usage: true,
    };
    let report = GnaLoadTester::new(cfg).run(&mut req_cfg).unwrap();

    assert_eq!(report.completed_inferences, 3);
    assert_eq!(report.failed_inferences, 0);
    assert_eq!(report.usage_monitor.inference_count(), 3);
}

#[test]
fn test_load_tester_drain_track_hw_usage_false() {
    reset_stress_state();
    MOCK_STRESS.with(|s| {
        s.borrow_mut().wait_delays_ms = vec![(1, 45)];
    });

    let lib = GnaLibrary::from_symbol_table(full_stress_symbols());
    let mut req_cfg = GnaRequestConfig::create(&lib, 1).unwrap();

    let cfg = GnaLoadTestConfig {
        iterations: None,
        duration: Some(Duration::from_millis(25)),
        concurrency: 2,
        timeout_per_request_ms: 500,
        track_hw_usage: false,
    };
    let report = GnaLoadTester::new(cfg).run(&mut req_cfg).unwrap();

    assert_eq!(report.completed_inferences, 2);
    assert_eq!(report.failed_inferences, 0);
    assert_eq!(report.usage_monitor.inference_count(), 0);
}

#[test]
fn test_load_tester_drain_missing_instrumentation_stats_err() {
    reset_stress_state();
    MOCK_STRESS.with(|s| {
        s.borrow_mut().wait_delays_ms = vec![(1, 45)];
    });

    let mut syms_no_inst = full_stress_symbols();
    syms_no_inst.instrumentation_config_create = None;
    let lib_no_inst = GnaLibrary::from_symbol_table(syms_no_inst);
    let mut req_no_inst = GnaRequestConfig::create(&lib_no_inst, 1).unwrap();

    let cfg = GnaLoadTestConfig {
        iterations: None,
        duration: Some(Duration::from_millis(25)),
        concurrency: 2,
        timeout_per_request_ms: 500,
        track_hw_usage: true,
    };
    let report = GnaLoadTester::new(cfg).run(&mut req_no_inst).unwrap();

    assert_eq!(report.completed_inferences, 2);
    assert_eq!(report.failed_inferences, 0);
    assert_eq!(report.usage_monitor.inference_count(), 0);
}

use std::time::Duration;

use nanai_gna_dll_load::{
    error::GnaError,
    inference::GnaRequestConfig,
    loader::GnaLibrary,
    stress::{GnaLoadTestConfig, GnaLoadTester},
};

use super::mock::{MOCK_STRESS, full_stress_symbols, reset_stress_state};

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

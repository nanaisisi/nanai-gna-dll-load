use nanai_gna_dll_load::{
    instrumentation::GnaUsageMonitor,
    stress::{GnaLoadTestConfig, GnaLoadTestReport},
};
use std::time::Duration;

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

use std::time::Duration;

use nanai_gna_dll_load::stress::GnaLoadTestConfig;

#[test]
fn test_load_test_config_builder() {
    let config = GnaLoadTestConfig::new()
        .with_iterations(500)
        .with_concurrency(4)
        .with_duration(Duration::from_secs(10))
        .with_timeout_ms(2000)
        .with_track_hw_usage(false);

    assert_eq!(config.iterations(), Some(500));
    assert_eq!(config.concurrency(), 4);
    assert_eq!(config.duration(), Some(Duration::from_secs(10)));
    assert_eq!(config.timeout_per_request_ms(), 2000);
    assert!(!config.track_hw_usage());
}

#[test]
fn test_load_test_config_defaults_and_clamping() {
    let def = GnaLoadTestConfig::default();
    assert_eq!(def.iterations(), Some(1000));
    assert_eq!(def.duration(), None);
    assert_eq!(def.concurrency(), 1);
    assert_eq!(def.timeout_per_request_ms(), 1000);
    assert!(def.track_hw_usage());

    let clamped = GnaLoadTestConfig::new().with_concurrency(0);
    assert_eq!(clamped.concurrency(), 1);
}

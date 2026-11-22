use nanai_gna_dll_load::instrumentation::{GnaPerformanceStats, GnaUsageMonitor};

#[test]
fn test_performance_stats_calculation() {
    let stats = GnaPerformanceStats::from_cycles(10_000, 2_500).with_execution_time(Some(150));

    assert_eq!(stats.total_cycles(), 10_000);
    assert_eq!(stats.stall_cycles(), 2_500);
    assert_eq!(stats.active_cycles(), 7_500);
    assert!((stats.hw_usage_ratio() - 0.75).abs() < 1e-6);
    assert!((stats.hw_usage_percentage() - 75.0).abs() < 1e-6);
    assert_eq!(stats.execution_time(), Some(150));
}

#[test]
fn test_usage_monitor_aggregation() {
    let mut monitor = GnaUsageMonitor::new();
    assert_eq!(monitor.inference_count(), 0);
    assert_eq!(monitor.cumulative_hw_usage_ratio(), 0.0);
    assert_eq!(monitor.cumulative_execution_time(), None);
    assert_eq!(monitor.average_execution_time(), None);

    let run1 = GnaPerformanceStats::from_cycles(1000, 200).with_execution_time(Some(50));
    monitor.record(&run1);

    assert_eq!(monitor.inference_count(), 1);
    assert_eq!(monitor.cumulative_total_cycles(), 1000);
    assert_eq!(monitor.cumulative_stall_cycles(), 200);
    assert_eq!(monitor.cumulative_active_cycles(), 800);
    assert!((monitor.cumulative_hw_usage_ratio() - 0.8).abs() < 1e-6);
    assert!((monitor.cumulative_hw_usage_percentage() - 80.0).abs() < 1e-6);
    assert_eq!(monitor.cumulative_execution_time(), Some(50));
    assert_eq!(monitor.average_execution_time(), Some(50.0));
    assert_eq!(monitor.last_stats(), Some(&run1));

    let run2 = GnaPerformanceStats::from_cycles(3000, 1800).with_execution_time(Some(150));
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
    assert_eq!(monitor.cumulative_execution_time(), Some(200));
    assert_eq!(monitor.average_execution_time(), Some(100.0));
    assert_eq!(monitor.last_stats(), Some(&run2));

    // Test reset
    monitor.reset();
    assert_eq!(monitor.inference_count(), 0);
    assert_eq!(monitor.cumulative_total_cycles(), 0);
    assert_eq!(monitor.cumulative_hw_usage_ratio(), 0.0);
    assert_eq!(monitor.cumulative_execution_time(), None);
    assert_eq!(monitor.average_execution_time(), None);
    assert_eq!(monitor.last_stats(), None);
}

#[test]
fn test_usage_monitor_without_exec_time() {
    let mut monitor = GnaUsageMonitor::new();
    let stats = GnaPerformanceStats::from_cycles(500, 100);
    monitor.record(&stats);

    assert_eq!(monitor.inference_count(), 1);
    assert_eq!(monitor.cumulative_total_cycles(), 500);
    assert_eq!(monitor.cumulative_stall_cycles(), 100);
    assert_eq!(monitor.cumulative_active_cycles(), 400);
    assert!((monitor.cumulative_hw_usage_ratio() - 0.8).abs() < 1e-6);
    assert_eq!(monitor.average_execution_time(), None);
}

#[test]
fn test_device_profile_and_monitor_ratios() {
    use nanai_gna_dll_load::GnaDeviceProfile;
    use std::time::Duration;

    let profile = GnaDeviceProfile::new("GNA 3.0", 400_000_000).with_mac_units(32);
    assert_eq!(profile.name, "GNA 3.0");
    assert_eq!(profile.max_clock_hz, 400_000_000);
    assert_eq!(profile.mac_units, Some(32));

    let zero_stats = GnaPerformanceStats::from_cycles(0, 0);
    assert_eq!(zero_stats.stall_ratio(), 0.0);

    let stats = GnaPerformanceStats::from_cycles(1000, 250);
    assert!((stats.stall_ratio() - 0.25).abs() < 1e-6);

    let mut monitor = GnaUsageMonitor::new();
    assert_eq!(monitor.cumulative_stall_ratio(), 0.0);
    assert_eq!(
        monitor.effective_utilization_ratio(Duration::ZERO, &profile),
        0.0
    );

    monitor.record(&stats);
    assert!((monitor.cumulative_stall_ratio() - 0.25).abs() < 1e-6);

    let zero_clock_profile = GnaDeviceProfile::new("Zero", 0);
    assert_eq!(
        monitor.effective_utilization_ratio(Duration::from_secs(1), &zero_clock_profile),
        0.0
    );

    let eff = monitor.effective_utilization_ratio(Duration::from_millis(1), &profile);
    let expected = 750.0 / (400_000_000.0 * 0.001);
    assert!((eff - expected).abs() < 1e-9);
    assert!(
        (monitor.effective_utilization_percentage(Duration::from_millis(1), &profile)
            - expected * 100.0)
            .abs()
            < 1e-7
    );
}

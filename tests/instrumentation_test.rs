use nanai_gna_dll_load::{GnaPerformanceStats, GnaUsageMonitor};

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

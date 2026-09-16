#[cfg(test)]
mod tests {
    use nanai_gna_dll_load::MonitoringApi;

    // テスト用ヘルパー関数：APIのシングルトンインスタンスを取得する
    fn setup() -> &'static MonitoringApi {
        MonitoringApi::get_instance()
    }

    #[test]
    fn test_metric_store_averaging_logic() {
        let api = setup();
        let key = "latency_average";

        // 1. 最初の記録 (Count=1, Sum=100.0, Avg=100.0)
        let avg1 = api.record_measurement(key, 100.0);
        assert!((avg1 - 100.0).abs() < 0.001, "Average after first recording failed.");

        // 2. 二回目の記録 (Count=2, Sum=(100+50)=150.0, Avg=75.0)
        let avg2 = api.record_measurement(key, 50.0);
        assert!((avg2 - 75.0).abs() < 0.001, "Average after second recording failed.");

        // 3. 三回目の記録 (Count=3, Sum=(150+20)=170.0, Avg=170/3 ≈ 56.66...)
        let avg3 = api.record_measurement(key, 20.0);
        assert!((avg3 - 170.0 / 3.0).abs() < 0.001, "Average after third recording failed.");

        // 4. 全メトリクスの確認
        let metrics = api.store.get_all_metrics();
        assert_eq!(metrics.len(), 1, "Should only contain one metric key.");

        let final_metric = metrics.get(key).expect("Key must exist after recording.");
        assert_eq!(final_metric.count, 3, "Total count is incorrect.");
        assert!((final_metric.sum_of_values - 170.0).abs() < 0.001, "Total sum is incorrect.");

        println!("Success: Average tracking logic verified across multiple calls.");
    }
}

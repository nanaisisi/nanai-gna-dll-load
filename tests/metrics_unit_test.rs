#[cfg(test)]
mod tests {
    use nanai_gna_dll_load::MetricStore;
    use nanai_gna_dll_load::MonitoringApi;

    fn setup_metric_store() -> MetricStore {
        MetricStore::new()
    }

    #[test]
    fn test_metric_store_averaging_logic() {
        let api = MonitoringApi {
            store: MetricStore::new(),
        };
        let key = "latency_average";

        // 1. 最初の記録 (Count=1, Sum=100.0, Avg=100.0)
        let avg1 = api.record_measurement(key, 100.0);
        assert!(
            (avg1 - 100.0).abs() < 0.001,
            "Average after first recording failed."
        );

        // 2. 二回目の記録 (Count=2, Sum=(100+50)=150.0, Avg=75.0)
        let avg2 = api.record_measurement(key, 50.0);
        assert!(
            (avg2 - 75.0).abs() < 0.001,
            "Average after second recording failed."
        );

        // 3. 三回目の記録 (Count=3, Sum=(150+20)=170.0, Avg=170/3 ≈ 56.66...)
        let avg3 = api.record_measurement(key, 20.0);
        assert!(
            (avg3 - 170.0 / 3.0).abs() < 0.001,
            "Average after third recording failed."
        );

        // 4. 全メトリクスの確認
        let metrics = api.store.get_all_metrics();
        assert_eq!(metrics.len(), 1, "Should only contain one metric key.");

        let final_metric = metrics.get(key).expect("Key must exist after recording.");
        assert_eq!(final_metric.count, 3, "Total count is incorrect.");
        assert!(
            (final_metric.sum_of_values - 170.0).abs() < 0.001,
            "Total sum is incorrect."
        );

        println!("Success: Average tracking logic verified across multiple calls.");
    }
    #[test]
    fn test_metric_initialization_and_retrieval() {
        let store = setup_metric_store();
        let key1 = "latency";
        let initial_metric = store.get_or_init(key1);

        // 初期値の検証: Count=0, Sum=0, Value=0.0 (初期化ロジックが正しく機能しているか確認)
        assert_eq!(initial_metric.count, 0);
        assert_eq!(initial_metric.sum_of_values, 0.0);
    }

    #[test]
    fn test_record_single_value() {
        let store = setup_metric_store();
        let key = "request_count";
        let value1: f64 = 10.0;

        // 1回目の記録
        let avg1 = store.record(key, value1);

        assert_eq!(avg1, 10.0); // 平均値は単一値と同じ
        let metric = store.get_metric(key).expect("Metric should exist");
        assert_eq!(metric.count, 1);
        assert_eq!(metric.sum_of_values, 10.0);

        // 2回目の記録 (異なる値)
        let value2: f64 = 30.0;
        let avg2 = store.record(key, value2);

        // 新しい平均値の検証: (10 + 30) / 2 = 20.0
        assert_eq!(avg2, 20.0);
        let metric_after = store.get_metric(key).expect("Metric should exist");
        assert_eq!(metric_after.count, 2);
        // Sum: 10.0 + 30.0 = 40.0
        assert!((metric_after.sum_of_values - 40.0).abs() < 1e-9);
    }

    #[test]
    fn test_record_multiple_keys() {
        let store = setup_metric_store();

        // Key A: 10 -> Avg=10
        store.record("A", 10.0);
        assert!((store.get_metric("A").unwrap().sum_of_values - (1.0 * 10.0)).abs() < 1e-9);

        // Key B: 5 -> Avg=5
        store.record("B", 5.0);
        assert!((store.get_metric("B").unwrap().sum_of_values - (1.0 * 5.0)).abs() < 1e-9);

        // Key A: 20 -> Avg=(10+20)/2 = 15
        store.record("A", 20.0);
        assert!((store.get_metric("A").unwrap().sum_of_values - (2.0 * 15.0)).abs() < 1e-9);

        // Key B: 5 -> Avg=(5+5)/2 = 5
        store.record("B", 5.0);
        assert!((store.get_metric("B").unwrap().sum_of_values - (2.0 * 5.0)).abs() < 1e-9);

        // 全てのメトリクスを検証
        let all = store.get_all_metrics();
        assert_eq!(all.len(), 2);
        assert!((all.get("A").unwrap().sum_of_values - (2.0 * 15.0)).abs() < 1e-9);
        assert!((all.get("B").unwrap().sum_of_values - (2.0 * 5.0)).abs() < 1e-9);
    }

    #[test]
    fn test_monitoring_api_singleton() {
        let api1 = MonitoringApi::get_instance();
        let api2 = MonitoringApi::get_instance();
        // 同一インスタンスであることを確認 (メモリ上の同一性)
        assert!(
            std::ptr::eq(api1, api2),
            "MonitoringApi should be a singleton."
        );

        // 状態の検証: API1でレコードした値がAPI2からも取得できるか？
        let key = "test_singleton";
        let value = 50.0;

        // 1回目：API1を通じて記録
        api1.record_measurement(key, value);

        // 検証: API2から読み出し、値が保持されているか
        let retrieved_avg = api2.store.get_metric(key).unwrap().sum_of_values / 1.0;
        assert!(
            (retrieved_avg - 50.0).abs() < 1e-9,
            "Singleton state should persist across calls."
        );

        // 2回目：API2を通じて再記録
        api2.record_measurement(key, 100.0); // 新しい合計: 150.0 / Count: 2
        let final_avg = api2.store.get_metric(key).unwrap().sum_of_values / 2.0;
        assert!(
            (final_avg - 75.0).abs() < 1e-9,
            "Singleton state must correctly accumulate."
        );
    }
}

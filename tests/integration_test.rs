#[cfg(test)]
mod tests {
    use nanai_gna_dll_load::MonitoringApi;
    use std::thread;
    use std::time::Duration;

    // メトリクスストアのテスト用初期化ヘルパー
    fn setup() -> &'static MonitoringApi {
        MonitoringApi::get_instance()
    }

    #[test]
    fn test_metrics_api_lifecycle() {
        let api = setup();

        let key1 = "initialization_latency";
        let key2 = "request_count";

        // 1. 初期計測（start_measurement）のシミュレーション
        api.start_measurement(key1);

        // 2. 値の記録と更新（record_measurement）の実行 (成功時)
        api.record_measurement(key1, 50.0); // 最初のレイテンシをセット
        assert!(api.store.get_or_init(key1).value == 50.0);

        // 時間経過と再測定（更新）のシミュレーション
        thread::sleep(Duration::from_millis(1));
        api.record_measurement(key1, 65.0); // 更新されるべき値
        assert!(api.store.get_or_init(key1).value == 65.0);

        // 3. 新しいメトリクスの記録（異なる指標）
        api.record_measurement(key2, 1.0); // カウンタとして初期値1.0を設定
        assert!(api.store.get_or_init(key2).value == 1.0);

        // 4. 全メトリクス取得の検証
        let metrics = api.store.get_all_metrics();
        assert_eq!(metrics.len(), 2, "Should have exactly two unique metrics recorded.");
        assert!(metrics.contains_key(key1));
        assert!(metrics.contains_key(key2));

        println!("Metrics test completed successfully: Latency tracked and Request Count incremented (conceptually).");
    }
}

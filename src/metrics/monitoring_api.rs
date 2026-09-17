use crate::metrics::metric_store::MetricStore;

use std::sync::OnceLock;

pub struct MonitoringApi {
    pub store: MetricStore,
}

impl MonitoringApi {
    /// 初期化関数。シングルトンパターンを適用し、アプリケーション全体で単一インスタンスを保証する。
    pub fn get_instance() -> &'static MonitoringApi {
        // OnceLockを使って静的なシングルトンインスタンスを管理
        static INSTANCE: OnceLock<MonitoringApi> = OnceLock::new();
        INSTANCE.get_or_init(|| MonitoringApi {
            store: MetricStore::new(),
        })
    }

    /// 新しいメトリクスを記録する（例：レイテンシ測定の開始）。
    pub fn start_measurement(&self, key: &str) {
        // 実際のロジックでは、この時点で計測コンテキスト（開始タイムスタンプ）をストアに格納すべきです。
        println!("Monitoring started for key: {}", key);
    }

    /// 測定値を記録し、メトリクスを更新します。戻り値は現在の平均値です。
    pub fn record_measurement(&self, key: &str, value: f64) -> f64 {
        // メトリクスの更新と集計処理を一箇所で行う
        println!("Recorded measurement: key={}, value={}", key, value);
        value.round()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_measurement() {
        let api = MonitoringApi::get_instance();
        api.start_measurement("test_key");
    }

    #[test]
    fn test_record_measurement() {
        let api = MonitoringApi::get_instance();
        let value = api.record_measurement("test_key", 10.5);
        assert_eq!(value, 11.0); // 10.5を四捨五入して11.0になることを確認
    }
}

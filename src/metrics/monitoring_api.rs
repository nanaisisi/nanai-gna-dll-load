use crate::metrics::metric_store::MetricStore;

use std::sync::OnceLock;

pub struct MonitoringApi {
    pub(crate) store: MetricStore,
}

impl MonitoringApi {
    /// 新しい MonitoringApi インスタンスを生成します。
    pub fn new() -> Self {
        Self {
            store: MetricStore::new(),
        }
    }

    /// 内部の MetricStore への参照を返します。
    pub fn store(&self) -> &MetricStore {
        &self.store
    }

    /// 初期化関数。シングルトンパターンを適用し、アプリケーション全体で単一インスタンスを保証する。
    pub fn get_instance() -> &'static MonitoringApi {
        // OnceLockを使って静的なシングルトンインスタンスを管理
        static INSTANCE: OnceLock<MonitoringApi> = OnceLock::new();
        INSTANCE.get_or_init(MonitoringApi::new)
    }

    /// 新しいメトリクスを記録する（例：レイテンシ測定の開始）。
    pub fn start_measurement(&self, key: &str) {
        // 実際のロジックでは、この時点で計測コンテキスト（開始タイムスタンプ）をストアに格納すべきです。
        let _ = key;
    }

    /// 測定値を記録し、メトリクスを更新します。戻り値は現在の平均値です。
    pub fn record_measurement(&self, key: &str, value: f64) -> f64 {
        // メトリクスの更新と集計処理を一箇所で行う
        self.store.record(key, value)
    }
}

impl Default for MonitoringApi {
    fn default() -> Self {
        Self::new()
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
        let value = api.record_measurement("test_key_api", 10.5);
        assert_eq!(value, 10.5);
    }
}

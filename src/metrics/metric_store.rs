use chrono::Utc;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::metrics::metric::Metric;

// グローバルに共有されるメトリクスストア。スレッドセーフなアクセスを提供します。
pub struct MetricStore {
    store: Mutex<HashMap<String, Metric>>,
}

impl MetricStore {
    /// 新しいMetricStoreのインスタンスを作成するファクトリ関数。
    pub fn new() -> Self {
        MetricStore {
            store: Mutex::new(HashMap::new()),
        }
    }

    /// 指定されたキーのメトリクスを安全に取得し、存在しない場合は初期化します。
    pub fn get_or_init(&self, key: &str) -> Metric {
        let store = self.store.lock().unwrap();
        // 初期化時はSum=0, Count=0で初期化します。
        store.get(key).cloned().unwrap_or_else(|| Metric {
            last_updated: Utc::now(),
            sum_of_values: 0.0,
            count: 0,
            value: 0.0,
        })
    }

    /// メトリクスを記録し、その時点での平均値を返します。計測ロジックの唯一のエントリーポイントです。
    pub fn record(&self, key: &str, new_value: f64) -> f64 {
        let mut store = self.store.lock().unwrap();
        let metric = store
            .entry(key.to_string())
            .and_modify(|m| {
                m.sum_of_values += new_value;
                m.count += 1;
                m.value = new_value;
                m.last_updated = Utc::now();
            })
            .or_insert_with(|| Metric {
                last_updated: Utc::now(),
                sum_of_values: new_value,
                count: 1,
                value: new_value,
            });

        metric.sum_of_values / (metric.count as f64)
    }

    /// 指定されたキーの最新メトリクス（平均値を含む）を取得します。
    pub fn get_metric(&self, key: &str) -> Option<Metric> {
        let store = self.store.lock().unwrap();
        store.get(key).cloned()
    }

    /// 全てのメトリクスを読み取り専用で取得する（モニタリングエンドポイント用）。
    pub fn get_all_metrics(&self) -> HashMap<String, Metric> {
        let store = self.store.lock().unwrap();
        store.clone()
    }
}

impl Default for MetricStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_store() {
        let store = MetricStore::new();
        let avg = store.record("test", 10.0);
        assert_eq!(avg, 10.0);
    }
}

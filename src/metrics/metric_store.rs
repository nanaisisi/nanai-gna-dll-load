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
        if let Some(metric) = store.get_mut(key) {
            // 統計更新ロジック：合計に加算し、カウントを増やす。
            metric.sum_of_values += new_value;
            metric.count += 1;
            metric.last_updated = Utc::now();
        } else {
            let now = Utc::now();
            store.insert(
                key.to_string(),
                Metric {
                    last_updated: now,
                    sum_of_values: new_value, // 初回は値自体が合計となる
                    count: 1,
                    value: new_value,
                },
            );
        }

        // 計算された現在の平均値を返します。Countが0の場合は発生しないため、安全に計算できます。

        self.get_metric(key)
            .map(|m| m.sum_of_values / (m.count as f64))
            .unwrap_or(0.0)
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

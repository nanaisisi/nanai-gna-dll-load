use jiff::Timestamp;

/// モニタリングデータ構造。統計情報を追跡するための要素を持ちます。
#[derive(Debug, Clone)]
pub struct Metric {
    pub(crate) last_updated: Timestamp,
    // 統計量の追跡に必要なフィールド
    pub(crate) sum_of_values: f64, // 全計測値の合計 (Sum)
    pub(crate) count: u64,         // 計測回数 (Count)
    pub(crate) value: f64,
} // 最後に記録された値

impl Metric {
    pub fn new() -> Self {
        Metric {
            last_updated: Timestamp::now(),
            sum_of_values: 0.0,
            count: 0,
            value: 0.0,
        }
    }

    /// 最終更新タイムスタンプを取得します。
    pub fn last_updated(&self) -> Timestamp {
        self.last_updated
    }

    /// 計測値の合計を取得します。
    pub fn sum_of_values(&self) -> f64 {
        self.sum_of_values
    }

    /// 計測回数を取得します。
    pub fn count(&self) -> u64 {
        self.count
    }

    /// 最後に記録された値を取得します。
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Default for Metric {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let metric = Metric::new();
        assert_eq!(metric.sum_of_values, 0.0);
        assert_eq!(metric.count, 0);
        assert_eq!(metric.value, 0.0);
    }
}

use chrono::{DateTime, Utc};

/// モニタリングデータ構造。統計情報を追跡するための要素を持ちます。
#[derive(Debug, Clone)]
pub struct Metric {
    pub last_updated: DateTime<Utc>,
    // 統計量の追跡に必要なフィールド
    pub sum_of_values: f64, // 全計測値の合計 (Sum)
    pub count: u64,         // 計測回数 (Count)
    pub value: f64,
} // 最後に記録された値

impl Metric {
    pub fn new() -> Self {
        Metric {
            last_updated: Utc::now(),
            sum_of_values: 0.0,
            count: 0,
            value: 0.0,
        }
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

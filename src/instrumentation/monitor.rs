/// Detailed performance statistics for a single GNA inference execution.
#[derive(Debug, Clone, PartialEq)]
pub struct GnaPerformanceStats {
    /// Total hardware execution cycles.
    pub(crate) total_cycles: u64,
    /// Hardware stall cycles (memory waits, DMA wait, etc.).
    pub(crate) stall_cycles: u64,
    /// Active (non-stalled) hardware execution cycles (`total_cycles - stall_cycles`).
    pub(crate) active_cycles: u64,
    /// Hardware utilization ratio in range `[0.0, 1.0]` (`active_cycles / total_cycles`).
    pub(crate) hw_usage_ratio: f64,
    /// Execution time in configured unit (e.g. microseconds) if `LibExecution` was recorded.
    pub(crate) execution_time: Option<u64>,
}

impl GnaPerformanceStats {
    /// Construct performance statistics from total and stall cycle counts.
    pub fn from_cycles(total_cycles: u64, stall_cycles: u64) -> Self {
        let active_cycles = total_cycles.saturating_sub(stall_cycles);
        let hw_usage_ratio = if total_cycles > 0 {
            (active_cycles as f64) / (total_cycles as f64)
        } else {
            0.0
        };
        Self {
            total_cycles,
            stall_cycles,
            active_cycles,
            hw_usage_ratio,
            execution_time: None,
        }
    }

    /// Attach an optional execution time measurement.
    pub fn with_execution_time(mut self, execution_time: Option<u64>) -> Self {
        self.execution_time = execution_time;
        self
    }

    /// Total hardware execution cycles.
    pub fn total_cycles(&self) -> u64 {
        self.total_cycles
    }

    /// Hardware stall cycles.
    pub fn stall_cycles(&self) -> u64 {
        self.stall_cycles
    }

    /// Active (non-stalled) hardware execution cycles.
    pub fn active_cycles(&self) -> u64 {
        self.active_cycles
    }

    /// Hardware utilization ratio in range `[0.0, 1.0]`.
    pub fn hw_usage_ratio(&self) -> f64 {
        self.hw_usage_ratio
    }

    /// Execution time in configured unit, if recorded.
    pub fn execution_time(&self) -> Option<u64> {
        self.execution_time
    }

    /// Percentage representation of hardware utilization (0.0% to 100.0%).
    pub fn hw_usage_percentage(&self) -> f64 {
        self.hw_usage_ratio * 100.0
    }
}

/// Continuous monitor and aggregator for tracking GNA hardware usage and inference statistics over time.
#[derive(Debug, Clone, Default)]
pub struct GnaUsageMonitor {
    count: usize,
    cumulative_total_cycles: u64,
    cumulative_stall_cycles: u64,
    cumulative_active_cycles: u64,
    cumulative_exec_time: u64,
    exec_time_count: usize,
    last_stats: Option<GnaPerformanceStats>,
}

impl GnaUsageMonitor {
    /// Create a new empty usage monitor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a single inference performance measurement.
    pub fn record(&mut self, stats: &GnaPerformanceStats) {
        self.count += 1;
        self.cumulative_total_cycles = self
            .cumulative_total_cycles
            .saturating_add(stats.total_cycles);
        self.cumulative_stall_cycles = self
            .cumulative_stall_cycles
            .saturating_add(stats.stall_cycles);
        self.cumulative_active_cycles = self
            .cumulative_active_cycles
            .saturating_add(stats.active_cycles);

        if let Some(t) = stats.execution_time {
            self.cumulative_exec_time = self.cumulative_exec_time.saturating_add(t);
            self.exec_time_count += 1;
        }

        self.last_stats = Some(stats.clone());
    }

    /// Number of inference runs recorded.
    pub fn inference_count(&self) -> usize {
        self.count
    }

    /// Total hardware cycles accumulated across all recorded inferences.
    pub fn cumulative_total_cycles(&self) -> u64 {
        self.cumulative_total_cycles
    }

    /// Total stall cycles accumulated across all recorded inferences.
    pub fn cumulative_stall_cycles(&self) -> u64 {
        self.cumulative_stall_cycles
    }

    /// Total active execution cycles accumulated across all recorded inferences.
    pub fn cumulative_active_cycles(&self) -> u64 {
        self.cumulative_active_cycles
    }

    /// Cumulative weighted hardware utilization ratio across all runs (`cumulative_active / cumulative_total`).
    /// Returns 0.0 if cumulative total cycles is 0.
    pub fn cumulative_hw_usage_ratio(&self) -> f64 {
        if self.cumulative_total_cycles > 0 {
            (self.cumulative_active_cycles as f64) / (self.cumulative_total_cycles as f64)
        } else {
            0.0
        }
    }

    /// Cumulative weighted hardware utilization percentage (0.0% - 100.0%).
    pub fn cumulative_hw_usage_percentage(&self) -> f64 {
        self.cumulative_hw_usage_ratio() * 100.0
    }

    /// Average execution time per inference (if execution time was recorded).
    pub fn average_execution_time(&self) -> Option<f64> {
        if self.exec_time_count > 0 {
            Some((self.cumulative_exec_time as f64) / (self.exec_time_count as f64))
        } else {
            None
        }
    }

    /// Most recent performance stats recorded, if any.
    pub fn last_stats(&self) -> Option<&GnaPerformanceStats> {
        self.last_stats.as_ref()
    }

    /// Reset all accumulated metrics.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

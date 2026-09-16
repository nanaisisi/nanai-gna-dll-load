use crate::error::{GnaError, Result};
use crate::loader::GnaLibrary;
use crate::types::{
    Gna2InstrumentationMode, Gna2InstrumentationPoint, Gna2InstrumentationUnit,
    GNA2_STATUS_SUCCESS,
};

/// High-level safe wrapper for GNA Instrumentation Configuration.
pub struct GnaInstrumentationConfig {
    library: GnaLibrary,
    config_id: u32,
    points: Vec<Gna2InstrumentationPoint>,
    results: Vec<u64>,
}

impl GnaInstrumentationConfig {
    /// Create a new instrumentation configuration for the specified instrumentation points.
    pub fn create(
        library: &GnaLibrary,
        points: &[Gna2InstrumentationPoint],
    ) -> Result<Self> {
        if points.is_empty() || points.len() > 15 {
            return Err(GnaError::Other(
                "Instrumentation points count must be in range [1, 15]".into(),
            ));
        }

        let create_fn = library
            .symbols()
            .instrumentation_config_create
            .ok_or_else(|| {
                GnaError::Other("Gna2InstrumentationConfigCreate not supported".into())
            })?;

        let mut results = vec![0u64; points.len()];
        let mut config_id: u32 = 0;

        let status = unsafe {
            create_fn(
                points.len() as u32,
                points.as_ptr(),
                results.as_mut_ptr(),
                &mut config_id,
            )
        };

        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }

        Ok(Self {
            library: library.clone(),
            config_id,
            points: points.to_vec(),
            results,
        })
    }

    /// Assign this instrumentation configuration to a request configuration.
    pub fn assign_to_request_config(&self, request_config_id: u32) -> Result<()> {
        let assign_fn = self
            .library
            .symbols()
            .instrumentation_config_assign_to_request_config
            .ok_or_else(|| {
                GnaError::Other(
                    "Gna2InstrumentationConfigAssignToRequestConfig not supported".into(),
                )
            })?;

        let status = unsafe { assign_fn(self.config_id, request_config_id) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Set instrumentation measurement unit (Microseconds, Milliseconds, Cycles).
    pub fn set_unit(&mut self, unit: Gna2InstrumentationUnit) -> Result<()> {
        let set_unit_fn = self
            .library
            .symbols()
            .instrumentation_config_set_unit
            .ok_or_else(|| {
                GnaError::Other("Gna2InstrumentationConfigSetUnit not supported".into())
            })?;

        let status = unsafe { set_unit_fn(self.config_id, unit) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Set hardware instrumentation mode (TotalStall, WaitForDmaCompletion, etc.).
    pub fn set_mode(&mut self, mode: Gna2InstrumentationMode) -> Result<()> {
        let set_mode_fn = self
            .library
            .symbols()
            .instrumentation_config_set_mode
            .ok_or_else(|| {
                GnaError::Other("Gna2InstrumentationConfigSetMode not supported".into())
            })?;

        let status = unsafe { set_mode_fn(self.config_id, mode) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Access the slice of configured instrumentation points.
    pub fn points(&self) -> &[Gna2InstrumentationPoint] {
        &self.points
    }

    /// Access the results buffer populated after request execution.
    pub fn results(&self) -> &[u64] {
        &self.results
    }

    /// Calculate hardware utilization `(TotalCycles - StallCycles) / TotalCycles`.
    ///
    /// Requires `HwTotalCycles` and `HwStallCycles` to be included in points.
    pub fn compute_hw_usage(&self) -> Result<f64> {
        let mut total = None;
        let mut stall = None;

        for (&pt, &val) in self.points.iter().zip(self.results.iter()) {
            match pt {
                Gna2InstrumentationPoint::HwTotalCycles => total = Some(val),
                Gna2InstrumentationPoint::HwStallCycles => stall = Some(val),
                _ => {}
            }
        }

        let total: u64 = total.ok_or_else(|| {
            GnaError::Other("Missing HwTotalCycles instrumentation point".into())
        })?;
        let stall = stall.ok_or_else(|| {
            GnaError::Other("Missing HwStallCycles instrumentation point".into())
        })?;

        if total == 0 {
            return Err(GnaError::Other("HwTotalCycles is zero".into()));
        }

        let active = total.saturating_sub(stall);
        Ok((active as f64) / (total as f64))
    }

    /// Compute detailed performance statistics from instrumentation points and results.
    pub fn compute_performance_stats(&self) -> Result<GnaPerformanceStats> {
        let mut total = None;
        let mut stall = None;
        let mut exec_time :Option<u64> = None;

        for (&pt, &val) in self.points.iter().zip(self.results.iter()) {
            match pt {
                Gna2InstrumentationPoint::HwTotalCycles => total = Some(val),
                Gna2InstrumentationPoint::HwStallCycles => stall = Some(val),
                Gna2InstrumentationPoint::LibExecution => exec_time = Some(val),
                _ => {}
            }
        }

        let total_cycles = total.ok_or_else(|| {
            GnaError::Other("Missing HwTotalCycles instrumentation point".into())
        })?;
        let stall_cycles = stall.ok_or_else(|| {
            GnaError::Other("Missing HwStallCycles instrumentation point".into())
        })?;

        let active_cycles = total_cycles.saturating_sub(stall_cycles);
        let hw_usage_ratio = if total_cycles > 0 {
            (active_cycles as f64) / (total_cycles as f64)
        } else {
            0.0
        };

        Ok(GnaPerformanceStats {
            total_cycles,
            stall_cycles,
            active_cycles,
            hw_usage_ratio,
            execution_time: exec_time,
        })
    }

    /// Get configuration ID.
    pub fn id(&self) -> u32 {
        self.config_id
    }
}

/// Detailed performance statistics for a single GNA inference execution.
#[derive(Debug, Clone, PartialEq)]
pub struct GnaPerformanceStats {
    /// Total hardware execution cycles.
    pub total_cycles: u64,
    /// Hardware stall cycles (memory waits, DMA wait, etc.).
    pub stall_cycles: u64,
    /// Active (non-stalled) hardware execution cycles (`total_cycles - stall_cycles`).
    pub active_cycles: u64,
    /// Hardware utilization ratio in range `[0.0, 1.0]` (`active_cycles / total_cycles`).
    pub hw_usage_ratio: f64,
    /// Execution time in configured unit (e.g. microseconds) if `LibExecution` was recorded.
    pub execution_time: Option<u64>,
}

impl GnaPerformanceStats {
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
        self.cumulative_total_cycles = self.cumulative_total_cycles.saturating_add(stats.total_cycles);
        self.cumulative_stall_cycles = self.cumulative_stall_cycles.saturating_add(stats.stall_cycles);
        self.cumulative_active_cycles = self.cumulative_active_cycles.saturating_add(stats.active_cycles);

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

impl Drop for GnaInstrumentationConfig {
    fn drop(&mut self) {
        if let Some(release_fn) = self.library.symbols().instrumentation_config_release {
            unsafe {
                let _ = release_fn(self.config_id);
            }
        }
    }
}

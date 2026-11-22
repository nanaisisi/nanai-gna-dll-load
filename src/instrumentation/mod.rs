mod monitor;

pub use monitor::*;

use crate::error::GnaError;
use crate::loader::GnaLibrary;
use crate::metrics::monitoring_api::MonitoringApi;
use crate::types::{
    GNA2_STATUS_SUCCESS, Gna2InstrumentationMode, Gna2InstrumentationPoint, Gna2InstrumentationUnit,
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
    ) -> Result<Self, GnaError> {
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
    pub fn assign_to_request_config(&self, request_config_id: u32) -> Result<(), GnaError> {
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
    pub fn set_unit(&mut self, unit: Gna2InstrumentationUnit) -> Result<(), GnaError> {
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
    pub fn set_mode(&mut self, mode: Gna2InstrumentationMode) -> Result<(), GnaError> {
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

    fn extract_point_metrics(&self) -> Result<(u64, u64, Option<u64>), GnaError> {
        let mut total = None;
        let mut stall = None;
        let mut exec_time = None;

        for (&pt, &val) in self.points.iter().zip(self.results.iter()) {
            match pt {
                Gna2InstrumentationPoint::HwTotalCycles => total = Some(val),
                Gna2InstrumentationPoint::HwStallCycles => stall = Some(val),
                Gna2InstrumentationPoint::LibExecution => exec_time = Some(val),
                _ => {}
            }
        }

        let total_cycles = total
            .ok_or_else(|| GnaError::Other("Missing HwTotalCycles instrumentation point".into()))?;
        let stall_cycles = stall
            .ok_or_else(|| GnaError::Other("Missing HwStallCycles instrumentation point".into()))?;

        Ok((total_cycles, stall_cycles, exec_time))
    }

    /// Calculate hardware utilization `(TotalCycles - StallCycles) / TotalCycles`.
    ///
    /// Requires `HwTotalCycles` and `HwStallCycles` to be included in points.
    pub fn compute_hw_usage(&self) -> Result<f64, GnaError> {
        let (total, stall, _) = self.extract_point_metrics()?;
        if total == 0 {
            return Err(GnaError::Other("HwTotalCycles is zero".into()));
        }
        Ok((total.saturating_sub(stall) as f64) / (total as f64))
    }

    /// Compute detailed performance statistics from instrumentation points and results, AND record key metrics.
    pub fn compute_performance_stats(&self) -> Result<GnaPerformanceStats, GnaError> {
        let (total_cycles, stall_cycles, exec_time) = self.extract_point_metrics()?;

        let stats = GnaPerformanceStats::from_cycles(total_cycles, stall_cycles)
            .with_execution_time(exec_time);

        // --- Monitoring Integration Start ---
        let monitoring_api = MonitoringApi::get_instance();
        monitoring_api.record_measurement("gna_total_cycles", stats.total_cycles() as f64);
        monitoring_api.record_measurement("gna_stall_cycles", stats.stall_cycles() as f64);
        monitoring_api.record_measurement("gna_active_cycles", stats.active_cycles() as f64);
        if let Some(t) = stats.execution_time() {
            monitoring_api.record_measurement("gna_exec_time", t as f64);
        }
        // --- Monitoring Integration End ---

        Ok(stats)
    }

    /// Get configuration ID.
    pub fn id(&self) -> u32 {
        self.config_id
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

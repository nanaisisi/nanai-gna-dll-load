use std::ffi::c_void;

use crate::error::{GnaError, Result};
use crate::instrumentation::{GnaInstrumentationConfig, GnaPerformanceStats};
use crate::loader::GnaLibrary;
use crate::types::{
    GNA2_STATUS_SUCCESS, Gna2AccelerationMode, Gna2InstrumentationPoint, Gna2InstrumentationUnit,
};

/// High-level wrapper for GNA Request Configuration.
pub struct GnaRequestConfig {
    library: GnaLibrary,
    config_id: u32,
    instrumentation: Option<GnaInstrumentationConfig>,
}

impl GnaRequestConfig {
    /// Create a new request configuration for the given model ID.
    pub fn create(library: &GnaLibrary, model_id: u32) -> Result<Self> {
        let create_fn = library
            .symbols()
            .request_config_create
            .ok_or_else(|| GnaError::Other("Gna2RequestConfigCreate not supported".into()))?;

        let mut config_id: u32 = 0;
        let status = unsafe { create_fn(model_id, &mut config_id) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }

        Ok(Self {
            library: library.clone(),
            config_id,
            instrumentation: None,
        })
    }

    /// Set an operand buffer for this configuration.
    ///
    /// # Safety
    /// `buffer_ptr` must be a valid pointer to allocated memory of the expected size for the given operation operand,
    /// and it must remain valid and not be accessed concurrently in an invalid manner during inference execution.
    pub unsafe fn set_operand_buffer(
        &mut self,
        operation_index: u32,
        operand_index: u32,
        buffer_ptr: *mut c_void,
    ) -> Result<()> {
        let set_buffer_fn = self
            .library
            .symbols()
            .request_config_set_operand_buffer
            .ok_or_else(|| {
                GnaError::Other("Gna2RequestConfigSetOperandBuffer not supported".into())
            })?;

        let status =
            unsafe { set_buffer_fn(self.config_id, operation_index, operand_index, buffer_ptr) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Enable active outputs list for an operation operand.
    pub fn enable_active_list(&mut self, operation_index: u32, indices: &[u32]) -> Result<()> {
        let enable_list_fn = self
            .library
            .symbols()
            .request_config_enable_active_list
            .ok_or_else(|| {
                GnaError::Other("Gna2RequestConfigEnableActiveList not supported".into())
            })?;

        let status = unsafe {
            enable_list_fn(
                self.config_id,
                operation_index,
                indices.len() as u32,
                indices.as_ptr(),
            )
        };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Enable hardware consistency for the request configuration.
    pub fn enable_hardware_consistency(
        &mut self,
        device_version: crate::types::Gna2DeviceVersion,
    ) -> Result<()> {
        let consistency_fn = self
            .library
            .symbols()
            .request_config_enable_hardware_consistency
            .ok_or_else(|| {
                GnaError::Other("Gna2RequestConfigEnableHardwareConsistency not supported".into())
            })?;

        let status = unsafe { consistency_fn(self.config_id, device_version) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Set acceleration mode.
    pub fn set_acceleration_mode(&mut self, mode: Gna2AccelerationMode) -> Result<()> {
        let set_mode_fn = self
            .library
            .symbols()
            .request_config_set_acceleration_mode
            .ok_or_else(|| {
                GnaError::Other("Gna2RequestConfigSetAccelerationMode not supported".into())
            })?;

        let status = unsafe { set_mode_fn(self.config_id, mode) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Enqueue an asynchronous request using this configuration and return request ID.
    pub fn enqueue(&self) -> Result<u32> {
        let enqueue_fn = self
            .library
            .symbols()
            .request_enqueue
            .ok_or_else(|| GnaError::Other("Gna2RequestEnqueue not supported".into()))?;

        let mut request_id: u32 = 0;
        let status = unsafe { enqueue_fn(self.config_id, &mut request_id) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(request_id)
    }

    /// Wait for request completion with a timeout in milliseconds.
    pub fn wait_request(&self, request_id: u32, timeout_ms: u32) -> Result<()> {
        let wait_fn = self
            .library
            .symbols()
            .request_wait
            .ok_or_else(|| GnaError::Other("Gna2RequestWait not supported".into()))?;

        let status = unsafe { wait_fn(request_id, timeout_ms) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Alias for `wait_request`.
    pub fn wait(&self, request_id: u32, timeout_ms: u32) -> Result<()> {
        self.wait_request(request_id, timeout_ms)
    }

    /// Attach a custom instrumentation configuration to this request.
    pub fn attach_instrumentation(
        &mut self,
        instrumentation: GnaInstrumentationConfig,
    ) -> Result<()> {
        instrumentation.assign_to_request_config(self.config_id)?;
        self.instrumentation = Some(instrumentation);
        Ok(())
    }

    /// Enable performance counters for hardware usage measurement (`HwTotalCycles`, `HwStallCycles`, and `LibExecution`).
    ///
    /// This automatically creates and assigns a `GnaInstrumentationConfig` with cycle units.
    pub fn enable_performance_counter(&mut self) -> Result<()> {
        let points = [
            Gna2InstrumentationPoint::HwTotalCycles,
            Gna2InstrumentationPoint::HwStallCycles,
            Gna2InstrumentationPoint::LibExecution,
        ];
        let mut inst = GnaInstrumentationConfig::create(&self.library, &points)?;
        let _ = inst.set_unit(Gna2InstrumentationUnit::Cycles);
        inst.assign_to_request_config(self.config_id)?;
        self.instrumentation = Some(inst);
        Ok(())
    }

    /// Get a reference to the attached instrumentation configuration, if any.
    pub fn instrumentation(&self) -> Option<&GnaInstrumentationConfig> {
        self.instrumentation.as_ref()
    }

    /// Compute hardware usage ratio `[0.0, 1.0]` from the last execution.
    ///
    /// Requires `enable_performance_counter` or an instrumentation config containing
    /// `HwTotalCycles` and `HwStallCycles` to have been configured.
    pub fn get_hw_usage(&self) -> Result<f64> {
        let inst = self.instrumentation.as_ref().ok_or_else(|| {
            GnaError::Other("No instrumentation configured for request. Call enable_performance_counter() first.".into())
        })?;
        inst.compute_hw_usage()
    }

    /// Get detailed performance statistics (`GnaPerformanceStats`) from the last execution.
    pub fn get_performance_stats(&self) -> Result<GnaPerformanceStats> {
        let inst = self.instrumentation.as_ref().ok_or_else(|| {
            GnaError::Other("No instrumentation configured for request. Call enable_performance_counter() first.".into())
        })?;
        inst.compute_performance_stats()
    }

    /// Get configuration ID.
    pub fn id(&self) -> u32 {
        self.config_id
    }
}

impl Drop for GnaRequestConfig {
    fn drop(&mut self) {
        if let Some(release_fn) = self.library.symbols().request_config_release {
            unsafe {
                let _ = release_fn(self.config_id);
            }
        }
    }
}

use std::ffi::c_void;

use crate::error::{GnaError, Result};
use crate::loader::GnaLibrary;
use crate::types::{
    GNA2_STATUS_SUCCESS, Gna2DeviceVersion, Gna2ModelExportComponent, Gna2UserAllocator,
};

/// High-level safe wrapper for GNA Model Export configuration and execution.
pub struct GnaModelExportConfig {
    library: GnaLibrary,
    export_config_id: u32,
}

impl GnaModelExportConfig {
    /// Create a new model export configuration using a provided C-compatible allocator callback.
    pub fn create(library: &GnaLibrary, allocator: Gna2UserAllocator) -> Result<Self> {
        let create_fn = library
            .symbols()
            .model_export_config_create
            .ok_or_else(|| {
                GnaError::Other("Gna2ModelExportConfigCreate is not supported".into())
            })?;

        let mut export_config_id: u32 = 0;
        let status = unsafe { create_fn(allocator, &mut export_config_id) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }

        Ok(Self {
            library: library.clone(),
            export_config_id,
        })
    }

    /// Set source device index and source model ID for export.
    pub fn set_source(&mut self, source_device_index: u32, source_model_id: u32) -> Result<()> {
        let set_source_fn = self
            .library
            .symbols()
            .model_export_config_set_source
            .ok_or_else(|| {
                GnaError::Other("Gna2ModelExportConfigSetSource is not supported".into())
            })?;

        let status =
            unsafe { set_source_fn(self.export_config_id, source_device_index, source_model_id) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Set target device version for export.
    pub fn set_target(&mut self, target_device_version: Gna2DeviceVersion) -> Result<()> {
        let set_target_fn = self
            .library
            .symbols()
            .model_export_config_set_target
            .ok_or_else(|| {
                GnaError::Other("Gna2ModelExportConfigSetTarget is not supported".into())
            })?;

        let status = unsafe { set_target_fn(self.export_config_id, target_device_version) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Perform model component export.
    pub fn export(&self, component: Gna2ModelExportComponent) -> Result<(*mut c_void, u32)> {
        let export_fn = self
            .library
            .symbols()
            .model_export
            .ok_or_else(|| GnaError::Other("Gna2ModelExport is not supported".into()))?;

        let mut buffer: *mut c_void = std::ptr::null_mut();
        let mut size: u32 = 0;

        let status = unsafe { export_fn(self.export_config_id, component, &mut buffer, &mut size) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok((buffer, size))
    }

    /// Return export config ID.
    pub fn id(&self) -> u32 {
        self.export_config_id
    }
}

impl Drop for GnaModelExportConfig {
    fn drop(&mut self) {
        if let Some(release_fn) = self.library.symbols().model_export_config_release {
            unsafe {
                let _ = release_fn(self.export_config_id);
            }
        }
    }
}

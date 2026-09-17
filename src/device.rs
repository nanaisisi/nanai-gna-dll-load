use crate::error::{GnaError, Result};
use crate::loader::GnaLibrary;
use crate::memory::GnaBuffer;
use crate::types::{GNA2_STATUS_SUCCESS, Gna2DeviceVersion};

/// Represents an open GNA device handle.
///
/// Ensures safe RAII resource cleanup: when dropped, `Gna2DeviceClose` is automatically called.
pub struct GnaDevice {
    library: GnaLibrary,
    index: u32,
    version: Gna2DeviceVersion,
}

impl GnaDevice {
    /// Query the number of available GNA devices in the system.
    pub fn get_count(library: &GnaLibrary) -> Result<u32> {
        let mut count: u32 = 0;
        let status = unsafe { (library.symbols().device_get_count)(&mut count) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(count)
    }

    /// Query the version of a device by its index without opening it.
    pub fn get_version(library: &GnaLibrary, device_index: u32) -> Result<Gna2DeviceVersion> {
        let count = Self::get_count(library)?;
        if count == 0 {
            return Err(GnaError::NoDevicesAvailable);
        }
        if device_index >= count {
            return Err(GnaError::DeviceIndexOutOfRange(device_index, count));
        }

        let mut version = Gna2DeviceVersion::default();
        let status = unsafe { (library.symbols().device_get_version)(device_index, &mut version) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(version)
    }

    /// Open a GNA device by index (0-based).
    pub fn open(library: &GnaLibrary, device_index: u32) -> Result<Self> {
        let version = Self::get_version(library, device_index)?;

        let status = unsafe { (library.symbols().device_open)(device_index) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }

        Ok(Self {
            library: library.clone(),
            index: device_index,
            version,
        })
    }

    /// Open the first available GNA device (index 0).
    pub fn open_first(library: &GnaLibrary) -> Result<Self> {
        Self::open(library, 0)
    }

    /// Create and initialize a GNA software device for model export.
    pub fn create_for_export(
        library: &GnaLibrary,
        target_device_version: Gna2DeviceVersion,
    ) -> Result<Self> {
        let create_fn = library
            .symbols()
            .device_create_for_export
            .ok_or_else(|| GnaError::Other("Gna2DeviceCreateForExport is not supported".into()))?;

        let mut device_index: u32 = 0;
        let status = unsafe { create_fn(target_device_version, &mut device_index) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }

        Ok(Self {
            library: library.clone(),
            index: device_index,
            version: target_device_version,
        })
    }

    /// Set number of worker threads for this device.
    pub fn set_number_of_threads(&self, threads: u32) -> Result<()> {
        let set_threads_fn = self
            .library
            .symbols()
            .device_set_number_of_threads
            .ok_or_else(|| {
                GnaError::Other("Gna2DeviceSetNumberOfThreads is not supported".into())
            })?;

        let status = unsafe { set_threads_fn(self.index, threads) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Allocate a device-bound memory buffer.
    pub fn allocate_buffer(&self, size: usize) -> Result<GnaBuffer> {
        GnaBuffer::new_for_device(&self.library, self.index, size)
    }

    /// Get device index.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Get device version.
    pub fn version(&self) -> Gna2DeviceVersion {
        self.version
    }

    /// Access reference to parent GnaLibrary.
    pub fn library(&self) -> &GnaLibrary {
        &self.library
    }
}

impl Drop for GnaDevice {
    fn drop(&mut self) {
        println!(
            "    [DEBUG GnaDevice::drop] Closing device index {}",
            self.index
        );
        unsafe {
            let status = (self.library.symbols().device_close)(self.index);
            println!(
                "    [DEBUG GnaDevice::drop] device_close returned status {}",
                status
            );
        }
    }
}

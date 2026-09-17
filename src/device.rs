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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::ffi::c_void;
    use std::sync::Mutex;

    struct MockState {
        device_count: u32,
        close_call_count: u32,
    }

    thread_local! {
        static MOCK_STATE: RefCell<MockState> = RefCell::new(MockState {
            device_count: 1,
            close_call_count: 0,
        });
    }

    // Global tracker for memory allocations to safely deallocate with Rust's allocator
    static ALLOCATIONS: Mutex<Option<HashMap<usize, std::alloc::Layout>>> = Mutex::new(None);

    const MOCK_ERROR_STATUS: crate::types::Gna2Status = 100;

    unsafe extern "C" fn mock_device_get_count(device_count: *mut u32) -> crate::types::Gna2Status {
        if !device_count.is_null() {
            MOCK_STATE.with(|s| {
                unsafe { *device_count = s.borrow().device_count };
            });
        }
        GNA2_STATUS_SUCCESS
    }

    unsafe extern "C" fn mock_device_get_version(
        device_index: u32,
        device_version: *mut Gna2DeviceVersion,
    ) -> crate::types::Gna2Status {
        let is_valid = MOCK_STATE.with(|s| device_index < s.borrow().device_count);
        if !is_valid {
            return MOCK_ERROR_STATUS;
        }
        if !device_version.is_null() {
            unsafe { *device_version = Gna2DeviceVersion::SOFTWARE_EMULATION };
        }
        GNA2_STATUS_SUCCESS
    }

    unsafe extern "C" fn mock_device_open(_device_index: u32) -> crate::types::Gna2Status {
        GNA2_STATUS_SUCCESS
    }

    unsafe extern "C" fn mock_device_close(_device_index: u32) -> crate::types::Gna2Status {
        MOCK_STATE.with(|s| {
            s.borrow_mut().close_call_count += 1;
        });
        GNA2_STATUS_SUCCESS
    }

    unsafe extern "C" fn mock_device_create_for_export(
        _target_device_version: Gna2DeviceVersion,
        device_index: *mut u32,
    ) -> crate::types::Gna2Status {
        if !device_index.is_null() {
            unsafe { *device_index = 42 };
        }
        GNA2_STATUS_SUCCESS
    }

    unsafe extern "C" fn mock_device_set_number_of_threads(
        _device_index: u32,
        _number_of_threads: u32,
    ) -> crate::types::Gna2Status {
        GNA2_STATUS_SUCCESS
    }

    unsafe extern "C" fn mock_memory_alloc(
        size_requested: u32,
        size_granted: *mut u32,
        memory_address: *mut *mut c_void,
    ) -> crate::types::Gna2Status {
        if size_requested == 0 {
            return MOCK_ERROR_STATUS;
        }
        let layout = match std::alloc::Layout::from_size_align(size_requested as usize, 64) {
            Ok(l) => l,
            Err(_) => return MOCK_ERROR_STATUS,
        };
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
        if ptr.is_null() {
            return MOCK_ERROR_STATUS;
        }
        let mut map = ALLOCATIONS.lock().unwrap();
        map.get_or_insert_with(HashMap::new)
            .insert(ptr as usize, layout);

        if !size_granted.is_null() {
            unsafe { *size_granted = size_requested };
        }
        if !memory_address.is_null() {
            unsafe { *memory_address = ptr as *mut c_void };
        }
        GNA2_STATUS_SUCCESS
    }

    unsafe extern "C" fn mock_memory_alloc_for_device(
        _device_index: u32,
        size_requested: u32,
        size_granted: *mut u32,
        memory_address: *mut *mut c_void,
    ) -> crate::types::Gna2Status {
        unsafe { mock_memory_alloc(size_requested, size_granted, memory_address) }
    }

    unsafe extern "C" fn mock_memory_free(memory: *mut c_void) -> crate::types::Gna2Status {
        if !memory.is_null() {
            let mut map = ALLOCATIONS.lock().unwrap();
            if let Some(allocs) = map.as_mut() {
                if let Some(layout) = allocs.remove(&(memory as usize)) {
                    unsafe { std::alloc::dealloc(memory as *mut u8, layout) };
                }
            }
        }
        GNA2_STATUS_SUCCESS
    }

    fn mock_symbol_table() -> crate::symbol_table::GnaSymbolTable {
        crate::symbol_table::GnaSymbolTable {
            device_get_count: mock_device_get_count,
            device_get_version: mock_device_get_version,
            device_open: mock_device_open,
            device_create_for_export: None,
            device_close: mock_device_close,
            device_set_number_of_threads: Some(mock_device_set_number_of_threads),
            get_library_version: None,
            memory_alloc: mock_memory_alloc,
            memory_alloc_for_device: Some(mock_memory_alloc_for_device),
            memory_free: mock_memory_free,
            memory_set_tag: None,
            model_create: None,
            model_release: None,
            model_get_last_error: None,
            model_error_get_message: None,
            model_error_get_max_message_length: None,
            operation_init_fully_connected_affine: None,
            request_config_create: None,
            request_config_set_operand_buffer: None,
            request_config_enable_active_list: None,
            request_config_enable_hardware_consistency: None,
            request_config_set_acceleration_mode: None,
            request_config_release: None,
            request_enqueue: None,
            request_wait: None,
            instrumentation_config_create: None,
            instrumentation_config_assign_to_request_config: None,
            instrumentation_config_set_unit: None,
            instrumentation_config_set_mode: None,
            instrumentation_config_release: None,
            model_export_config_create: None,
            model_export_config_release: None,
            model_export_config_set_source: None,
            model_export_config_set_target: None,
            model_export: None,
            model_override_alignment: None,
            status_get_message: None,
            status_get_max_message_length: None,
        }
    }

    // Helper function to set up a mock GnaLibrary with default count of 1
    fn setup_mock_library() -> GnaLibrary {
        setup_mock_library_with_count(1)
    }

    // Helper function to set up a mock GnaLibrary returning specific device count
    fn setup_mock_library_with_count(count: u32) -> GnaLibrary {
        MOCK_STATE.with(|s| {
            let mut state = s.borrow_mut();
            state.device_count = count;
            state.close_call_count = 0;
        });
        GnaLibrary::from_symbol_table(mock_symbol_table())
    }

    // Helper function to set up a mock GnaLibrary supporting create for export
    fn setup_mock_library_for_export() -> GnaLibrary {
        let mut symbols = mock_symbol_table();
        symbols.device_create_for_export = Some(mock_device_create_for_export);
        GnaLibrary::from_symbol_table(symbols)
    }

    fn get_mock_close_count() -> u32 {
        MOCK_STATE.with(|s| s.borrow().close_call_count)
    }

    fn reset_mock_state() {
        MOCK_STATE.with(|s| {
            let mut state = s.borrow_mut();
            state.close_call_count = 0;
            state.device_count = 1;
        });
    }

    #[test]
    fn test_get_count() {
        // Setup mock library returning N devices
        let mock_library = setup_mock_library_with_count(3);
        assert_eq!(GnaDevice::get_count(&mock_library).unwrap(), 3);
    }

    #[test]
    fn test_get_version() {
        let mock_library = setup_mock_library_with_count(2);
        let version = GnaDevice::get_version(&mock_library, 0).expect("Failed to get version");
        assert_eq!(version, Gna2DeviceVersion::SOFTWARE_EMULATION);
        assert!(GnaDevice::get_version(&mock_library, 2).is_err());
    }

    #[test]
    fn test_open_device_success() {
        // Setup mock library for successful open at index 0
        let mock_library = setup_mock_library();
        let device = GnaDevice::open(&mock_library, 0).expect("Failed to open device");
        assert_eq!(device.index(), 0);
        assert_eq!(device.version(), Gna2DeviceVersion::SOFTWARE_EMULATION);
    }

    #[test]
    fn test_open_first() {
        // Setup mock library for successful opening of the first device
        let mock_library = setup_mock_library();
        let device = GnaDevice::open_first(&mock_library).expect("Failed to open first device");
        assert_eq!(device.index(), 0);
    }

    #[test]
    fn test_create_for_export() {
        // Setup mock library supporting create for export
        let mock_library = setup_mock_library_for_export();
        let dummy_version = Gna2DeviceVersion::default();
        let device = GnaDevice::create_for_export(&mock_library, dummy_version).unwrap();
        assert!(device.index() > 0); // Should assign a valid index
    }

    #[test]
    fn test_set_number_of_threads() {
        // Requires an already opened device instance to modify
        let mock_library = setup_mock_library();
        let device = GnaDevice::open(&mock_library, 0).unwrap();
        assert!(device.set_number_of_threads(8).is_ok());
    }

    #[test]
    fn test_allocate_buffer() {
        // Requires an open device instance
        let mock_library = setup_mock_library();
        let device = GnaDevice::open(&mock_library, 0).unwrap();
        let buffer = device
            .allocate_buffer(1024)
            .expect("Failed to allocate buffer");
        assert_eq!(buffer.len(), 1024);
    }

    #[test]
    fn test_raii_cleanup() {
        // This test confirms that when `device` goes out of scope,
        // the `Drop` implementation calls device_close.
        let mock_library = setup_mock_library();
        reset_mock_state();
        {
            let _device = GnaDevice::open(&mock_library, 0).expect("Setup failed");
            assert_eq!(get_mock_close_count(), 0);
        }
        assert_eq!(get_mock_close_count(), 1);
    }
}

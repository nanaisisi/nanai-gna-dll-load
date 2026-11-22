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
    static MOCK_STATE: RefCell<MockState> = const { RefCell::new(MockState {
        device_count: 1,
        close_call_count: 0,
    }) };
}

// Global tracker for memory allocations to safely deallocate with Rust's allocator
static ALLOCATIONS: Mutex<Option<HashMap<usize, std::alloc::Layout>>> = Mutex::new(None);

unsafe extern "C" fn mock_device_get_count(
    device_count: *mut u32,
) -> crate::types::device::Gna2Status {
    MOCK_STATE.with(|s| {
        unsafe { *device_count = s.borrow().device_count };
    });
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_get_version(
    _device_index: u32,
    device_version: *mut Gna2DeviceVersion,
) -> crate::types::device::Gna2Status {
    unsafe { *device_version = Gna2DeviceVersion::SOFTWARE_EMULATION };
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_open(_device_index: u32) -> crate::types::device::Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_close(_device_index: u32) -> crate::types::device::Gna2Status {
    MOCK_STATE.with(|s| {
        s.borrow_mut().close_call_count += 1;
    });
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_create_for_export(
    _target_device_version: Gna2DeviceVersion,
    device_index: *mut u32,
) -> crate::types::device::Gna2Status {
    unsafe { *device_index = 42 };
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_set_number_of_threads(
    _device_index: u32,
    _number_of_threads: u32,
) -> crate::types::device::Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_memory_alloc(
    size_requested: u32,
    size_granted: *mut u32,
    memory_address: *mut *mut c_void,
) -> crate::types::device::Gna2Status {
    let layout = std::alloc::Layout::from_size_align(size_requested as usize, 64).unwrap();
    let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
    let mut map = ALLOCATIONS.lock().unwrap();
    map.get_or_insert_with(HashMap::new)
        .insert(ptr as usize, layout);

    unsafe {
        *size_granted = size_requested;
        *memory_address = ptr as *mut c_void;
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_memory_free(memory: *mut c_void) -> crate::types::device::Gna2Status {
    let mut map = ALLOCATIONS.lock().unwrap();
    let layout = map
        .as_mut()
        .expect("ALLOCATIONS should be initialized")
        .remove(&(memory as usize))
        .expect("pointer should exist in ALLOCATIONS");
    unsafe { std::alloc::dealloc(memory as *mut u8, layout) };
    GNA2_STATUS_SUCCESS
}

fn mock_symbol_table() -> crate::symbol_table::GnaSymbolTable {
    crate::symbol_table::GnaSymbolTable::from_required(
        mock_device_get_count,
        mock_device_get_version,
        mock_device_open,
        mock_device_close,
        mock_memory_alloc,
        mock_memory_free,
    )
    .with_device_set_number_of_threads(mock_device_set_number_of_threads)
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
    let symbols = mock_symbol_table();
    let symbols = symbols.with_device_create_for_export(mock_device_create_for_export);
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

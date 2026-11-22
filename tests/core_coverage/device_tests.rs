use nanai_gna_dll_load::device::GnaDevice;
use nanai_gna_dll_load::error::GnaError;
use nanai_gna_dll_load::loader::GnaLibrary;
use nanai_gna_dll_load::types::{GNA2_STATUS_SUCCESS, Gna2DeviceVersion};

use super::mock_state::*;
use super::mock_table::*;

#[test]
fn test_device_get_count_success_and_error() {
    reset_mock_state();
    let lib = full_mock_library();

    with_mock_state(|s| s.device_count = 4);
    assert_eq!(GnaDevice::get_count(&lib).unwrap(), 4);

    with_mock_state(|s| s.device_get_count_status = -9);
    let err = GnaDevice::get_count(&lib).unwrap_err();
    assert!(matches!(err, GnaError::StatusError { status: -9, .. }));
}

#[test]
fn test_device_get_version_error_branches() {
    reset_mock_state();
    let lib = full_mock_library();

    // device_get_count returns non-success status (covers '?' error branch in get_version)
    with_mock_state(|s| s.device_get_count_status = -9);
    let err = GnaDevice::get_version(&lib, 0).unwrap_err();
    assert!(matches!(err, GnaError::StatusError { status: -9, .. }));
    with_mock_state(|s| s.device_get_count_status = GNA2_STATUS_SUCCESS);

    // count == 0 -> NoDevicesAvailable
    with_mock_state(|s| s.device_count = 0);
    let err = GnaDevice::get_version(&lib, 0).unwrap_err();
    assert!(matches!(err, GnaError::NoDevicesAvailable));

    // index >= count -> DeviceIndexOutOfRange
    with_mock_state(|s| s.device_count = 2);
    let err = GnaDevice::get_version(&lib, 2).unwrap_err();
    assert!(matches!(err, GnaError::DeviceIndexOutOfRange(2, 2)));

    // device_get_version returns non-success status
    with_mock_state(|s| s.device_get_version_status = -11);
    let err = GnaDevice::get_version(&lib, 1).unwrap_err();
    assert!(matches!(err, GnaError::StatusError { status: -11, .. }));

    // success
    with_mock_state(|s| {
        s.device_get_version_status = GNA2_STATUS_SUCCESS;
        s.device_version_value = Gna2DeviceVersion::V3_5;
    });
    assert_eq!(
        GnaDevice::get_version(&lib, 1).unwrap(),
        Gna2DeviceVersion::V3_5
    );
}

#[test]
fn test_device_open_error_and_success_and_library_accessor() {
    reset_mock_state();
    let lib = full_mock_library();

    // get_version fails -> open propagates error via '?'
    with_mock_state(|s| s.device_count = 0);
    let err = GnaDevice::open(&lib, 0).err().unwrap();
    assert!(matches!(err, GnaError::NoDevicesAvailable));
    with_mock_state(|s| s.device_count = 1);

    with_mock_state(|s| s.device_open_status = -10);
    let err = GnaDevice::open(&lib, 0).err().unwrap();
    assert!(matches!(err, GnaError::StatusError { status: -10, .. }));

    with_mock_state(|s| {
        s.device_open_status = GNA2_STATUS_SUCCESS;
        s.device_version_value = Gna2DeviceVersion::V2_0;
    });
    {
        let device = GnaDevice::open_first(&lib).unwrap();
        assert_eq!(device.index(), 0);
        assert_eq!(device.version(), Gna2DeviceVersion::V2_0);
        assert_eq!(device.library().path(), lib.path());
    }
    assert_eq!(read_mock_state(|s| s.close_calls.clone()), vec![0]);
}

#[test]
fn test_device_create_for_export_all_branches() {
    reset_mock_state();

    // Missing symbol
    let lib_no_export = GnaLibrary::from_symbol_table(base_mock_symbol_table());
    let err = GnaDevice::create_for_export(&lib_no_export, Gna2DeviceVersion::V3_0)
        .err()
        .unwrap();
    assert!(matches!(err, GnaError::Other(msg) if msg.contains("Gna2DeviceCreateForExport")));

    // Non-success status
    let lib = full_mock_library();
    with_mock_state(|s| s.device_create_for_export_status = -8);
    let err = GnaDevice::create_for_export(&lib, Gna2DeviceVersion::V3_0)
        .err()
        .unwrap();
    assert!(matches!(err, GnaError::StatusError { status: -8, .. }));

    // Success & Drop closes exported index
    with_mock_state(|s| {
        s.device_create_for_export_status = GNA2_STATUS_SUCCESS;
        s.device_create_for_export_index = 42;
    });
    {
        let device = GnaDevice::create_for_export(&lib, Gna2DeviceVersion::EMBEDDED_3_1).unwrap();
        assert_eq!(device.index(), 42);
        assert_eq!(device.version(), Gna2DeviceVersion::EMBEDDED_3_1);
        assert_eq!(device.library().path(), lib.path());
    }
    assert_eq!(read_mock_state(|s| s.close_calls.clone()), vec![42]);
}

#[test]
fn test_device_set_number_of_threads_all_branches() {
    reset_mock_state();

    // Missing symbol
    let lib_no_threads = GnaLibrary::from_symbol_table(base_mock_symbol_table());
    let dev_no_threads = GnaDevice::open(&lib_no_threads, 0).unwrap();
    let err = dev_no_threads.set_number_of_threads(4).unwrap_err();
    assert!(matches!(err, GnaError::Other(msg) if msg.contains("Gna2DeviceSetNumberOfThreads")));
    drop(dev_no_threads);

    // Error status & success status
    let lib = full_mock_library();
    let dev = GnaDevice::open(&lib, 0).unwrap();

    with_mock_state(|s| s.device_set_threads_status = -10);
    let err = dev.set_number_of_threads(99).unwrap_err();
    assert!(matches!(err, GnaError::StatusError { status: -10, .. }));
    assert_eq!(read_mock_state(|s| s.last_threads_arg), Some((0, 99)));

    with_mock_state(|s| s.device_set_threads_status = GNA2_STATUS_SUCCESS);
    dev.set_number_of_threads(4).unwrap();
    assert_eq!(read_mock_state(|s| s.last_threads_arg), Some((0, 4)));

    // Also cover allocate_buffer and non-success device_close in Drop
    let buf = dev.allocate_buffer(64).unwrap();
    assert_eq!(buf.len(), 64);
    with_mock_state(|s| s.device_close_status = -1);
    drop(buf);
    drop(dev);
    assert_eq!(read_mock_state(|s| s.close_calls.clone()), vec![0, 0]);
}

// ---------------------------------------------------------------------------
// src/memory.rs tests
// ---------------------------------------------------------------------------

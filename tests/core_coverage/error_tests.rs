use nanai_gna_dll_load::error::GnaError;
use nanai_gna_dll_load::types::{
    GNA2_STATUS_MODEL_ERROR_UNAVAILABLE, GNA2_STATUS_SUCCESS,
    GNA2_STATUS_WARNING_ARITHMETIC_SATURATION, GNA2_STATUS_WARNING_DEVICE_BUSY,
};

#[test]
fn test_error_from_status_all_known_codes() {
    let expected = [
        (GNA2_STATUS_SUCCESS, "Success"),
        (GNA2_STATUS_WARNING_DEVICE_BUSY, "Warning: Device busy"),
        (
            GNA2_STATUS_WARNING_ARITHMETIC_SATURATION,
            "Warning: Arithmetic saturation",
        ),
        (
            GNA2_STATUS_MODEL_ERROR_UNAVAILABLE,
            "Warning: Model error unavailable",
        ),
        (-1, "General error"),
        (-3, "Unknown error"),
        (-4, "Functionality not implemented yet"),
        (-5, "Item identifier is invalid"),
        (-6, "NULL argument is not allowed"),
        (-7, "NULL argument is required"),
        (-8, "Unable to create new resources"),
        (-9, "Device not available"),
        (-10, "Device failed to open, thread count is invalid"),
        (-11, "Device version is invalid"),
        (-12, "Queue can not create or enqueue more requests"),
        (
            -13,
            "Failed to receive communication from the device driver",
        ),
        (-14, "Failed to send communication to the device driver"),
        (-15, "Hardware device parameter out of range"),
        (-16, "Hardware device virtual address out of range"),
        (
            -17,
            "Hardware device unexpected completion during PCIe operation",
        ),
        (-18, "Hardware device DMA error during PCIe operation"),
        (-19, "Hardware device MMU error during PCIe operation"),
        (-20, "Hardware device breakpoint hit"),
        (
            -21,
            "Critical hardware device error occurred, device has been reset",
        ),
    ];

    for (code, msg) in expected {
        let err = GnaError::from_status(code);
        match &err {
            GnaError::StatusError { status, message } => {
                assert_eq!(*status, code);
                assert_eq!(message, msg);
            }
            other => panic!("unexpected variant: {other:?}"),
        }
        assert_eq!(
            err.to_string(),
            format!("GNA operation failed with status code {code}: {msg}")
        );
    }
}

#[test]
fn test_error_from_status_unrecognized_codes() {
    for code in [-2, 4, 42, -22, -999, i32::MIN, i32::MAX] {
        let err = GnaError::from_status(code);
        match &err {
            GnaError::StatusError { status, message } => {
                assert_eq!(*status, code);
                assert_eq!(message, "Unrecognized GNA status code");
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }
}

#[test]
fn test_error_variants_display_and_debug() {
    let cases = [
        (
            GnaError::DeviceIndexOutOfRange(2, 1),
            "Device index 2 out of range (total devices: 1)",
        ),
        (GnaError::NoDevicesAvailable, "No GNA devices available"),
        (
            GnaError::NullPointer,
            "Null pointer encountered unexpectedly",
        ),
        (
            GnaError::InvalidBufferSize(0),
            "Invalid buffer size requested: 0",
        ),
        (
            GnaError::EnvVarNotSet("GNA_LIB_PATH".into()),
            "Environment variable 'GNA_LIB_PATH' is not set",
        ),
        (
            GnaError::LibraryPathNotFound("missing.dll".into()),
            "Library file not found at path: missing.dll",
        ),
        (
            GnaError::ModelCreationError {
                status: -5,
                detail: "bad layer".into(),
            },
            "GNA model creation failed (status: -5): bad layer",
        ),
        (
            GnaError::Other("unsupported".into()),
            "Operation error: unsupported",
        ),
    ];

    for (err, expected) in cases {
        assert_eq!(err.to_string(), expected);
        assert!(!format!("{err:?}").is_empty());
    }

    let search_err = GnaError::LibrarySearchFailed {
        tried: vec!["a.dll".into()],
        last_error: Some("not found".into()),
    };
    let display = search_err.to_string();
    assert!(display.contains("a.dll"));
    assert!(display.contains("not found"));
}

// ---------------------------------------------------------------------------
// src/types.rs tests
// ---------------------------------------------------------------------------

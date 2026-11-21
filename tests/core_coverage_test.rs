use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use nanai_gna_dll_load::device::GnaDevice;
use nanai_gna_dll_load::error::GnaError;
use nanai_gna_dll_load::loader::GnaLibrary;
use nanai_gna_dll_load::memory::GnaBuffer;
use nanai_gna_dll_load::symbol_table::GnaSymbolTable;
use nanai_gna_dll_load::types::{
    GNA2_DEFAULT, GNA2_DISABLED, GNA2_STATUS_MODEL_ERROR_UNAVAILABLE, GNA2_STATUS_SUCCESS,
    GNA2_STATUS_WARNING_ARITHMETIC_SATURATION, GNA2_STATUS_WARNING_DEVICE_BUSY,
    Gna2AccelerationMode, Gna2BiasMode, Gna2DataType, Gna2DeviceGeneration, Gna2DeviceVersion,
    Gna2ErrorType, Gna2InstrumentationMode, Gna2InstrumentationPoint, Gna2InstrumentationUnit,
    Gna2ItemType, Gna2MemoryTag, Gna2Model, Gna2ModelError, Gna2ModelExportComponent,
    Gna2ModelItem, Gna2Operation, Gna2OperationType, Gna2Shape, Gna2Status, Gna2Tensor,
    Gna2TensorMode,
};

#[derive(Clone, Debug)]
struct MockState {
    device_count: u32,
    device_get_count_status: Gna2Status,
    device_get_version_status: Gna2Status,
    device_version_value: Gna2DeviceVersion,
    device_open_status: Gna2Status,
    device_close_status: Gna2Status,
    device_create_for_export_status: Gna2Status,
    device_create_for_export_index: u32,
    device_set_threads_status: Gna2Status,
    last_threads_arg: Option<(u32, u32)>,
    close_calls: Vec<u32>,

    memory_alloc_status: Gna2Status,
    memory_alloc_return_null: bool,
    memory_alloc_granted_override: Option<u32>,
    memory_alloc_calls: Vec<u32>,

    memory_alloc_for_device_status: Gna2Status,
    memory_alloc_for_device_return_null: bool,
    memory_alloc_for_device_calls: Vec<(u32, u32)>,

    memory_free_status: Gna2Status,
    memory_free_calls: Vec<usize>,

    memory_set_tag_status: Gna2Status,
    memory_set_tag_calls: Vec<(usize, u32)>,
}

impl Default for MockState {
    fn default() -> Self {
        Self {
            device_count: 1,
            device_get_count_status: GNA2_STATUS_SUCCESS,
            device_get_version_status: GNA2_STATUS_SUCCESS,
            device_version_value: Gna2DeviceVersion::V3_0,
            device_open_status: GNA2_STATUS_SUCCESS,
            device_close_status: GNA2_STATUS_SUCCESS,
            device_create_for_export_status: GNA2_STATUS_SUCCESS,
            device_create_for_export_index: 7,
            device_set_threads_status: GNA2_STATUS_SUCCESS,
            last_threads_arg: None,
            close_calls: Vec::new(),

            memory_alloc_status: GNA2_STATUS_SUCCESS,
            memory_alloc_return_null: false,
            memory_alloc_granted_override: None,
            memory_alloc_calls: Vec::new(),

            memory_alloc_for_device_status: GNA2_STATUS_SUCCESS,
            memory_alloc_for_device_return_null: false,
            memory_alloc_for_device_calls: Vec::new(),

            memory_free_status: GNA2_STATUS_SUCCESS,
            memory_free_calls: Vec::new(),

            memory_set_tag_status: GNA2_STATUS_SUCCESS,
            memory_set_tag_calls: Vec::new(),
        }
    }
}

thread_local! {
    static MOCK_STATE: RefCell<MockState> = RefCell::new(MockState::default());
}

fn allocations() -> &'static Mutex<HashMap<usize, Layout>> {
    static ALLOCATIONS: OnceLock<Mutex<HashMap<usize, Layout>>> = OnceLock::new();
    ALLOCATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn reset_mock_state() {
    MOCK_STATE.with(|s| {
        *s.borrow_mut() = MockState::default();
    });
}

fn with_mock_state<R>(f: impl FnOnce(&mut MockState) -> R) -> R {
    MOCK_STATE.with(|s| f(&mut s.borrow_mut()))
}

fn read_mock_state<R>(f: impl FnOnce(&MockState) -> R) -> R {
    MOCK_STATE.with(|s| f(&s.borrow()))
}

fn alloc_tracked(size_requested: u32) -> Option<*mut c_void> {
    let alloc_size = size_requested.max(1) as usize;
    let layout = Layout::from_size_align(alloc_size, 64).ok()?;
    let ptr = unsafe { alloc_zeroed(layout) };
    if ptr.is_null() {
        return None;
    }
    allocations().lock().unwrap().insert(ptr as usize, layout);
    Some(ptr as *mut c_void)
}

unsafe extern "C" fn mock_device_get_count(device_count: *mut u32) -> Gna2Status {
    let (status, count) = read_mock_state(|s| (s.device_get_count_status, s.device_count));
    if status == GNA2_STATUS_SUCCESS && !device_count.is_null() {
        unsafe { *device_count = count };
    }
    status
}

unsafe extern "C" fn mock_device_get_version(
    _device_index: u32,
    device_version: *mut Gna2DeviceVersion,
) -> Gna2Status {
    let (status, ver) = read_mock_state(|s| (s.device_get_version_status, s.device_version_value));
    if status == GNA2_STATUS_SUCCESS && !device_version.is_null() {
        unsafe { *device_version = ver };
    }
    status
}

unsafe extern "C" fn mock_device_open(_device_index: u32) -> Gna2Status {
    read_mock_state(|s| s.device_open_status)
}

unsafe extern "C" fn mock_device_close(device_index: u32) -> Gna2Status {
    with_mock_state(|s| {
        s.close_calls.push(device_index);
        s.device_close_status
    })
}

unsafe extern "C" fn mock_device_create_for_export(
    _target_device_version: Gna2DeviceVersion,
    device_index: *mut u32,
) -> Gna2Status {
    let (status, idx) = read_mock_state(|s| {
        (
            s.device_create_for_export_status,
            s.device_create_for_export_index,
        )
    });
    if status == GNA2_STATUS_SUCCESS && !device_index.is_null() {
        unsafe { *device_index = idx };
    }
    status
}

unsafe extern "C" fn mock_device_set_number_of_threads(
    device_index: u32,
    number_of_threads: u32,
) -> Gna2Status {
    with_mock_state(|s| {
        s.last_threads_arg = Some((device_index, number_of_threads));
        s.device_set_threads_status
    })
}

unsafe extern "C" fn mock_memory_alloc(
    size_requested: u32,
    size_granted: *mut u32,
    memory_address: *mut *mut c_void,
) -> Gna2Status {
    let (status, return_null, granted_override) = with_mock_state(|s| {
        s.memory_alloc_calls.push(size_requested);
        (
            s.memory_alloc_status,
            s.memory_alloc_return_null,
            s.memory_alloc_granted_override,
        )
    });

    if status != GNA2_STATUS_SUCCESS {
        return status;
    }

    let granted = granted_override.unwrap_or(size_requested);
    if !size_granted.is_null() {
        unsafe { *size_granted = granted };
    }

    if return_null {
        if !memory_address.is_null() {
            unsafe { *memory_address = std::ptr::null_mut() };
        }
        return GNA2_STATUS_SUCCESS;
    }

    let Some(ptr) = alloc_tracked(size_requested) else {
        return -8;
    };
    if !memory_address.is_null() {
        unsafe { *memory_address = ptr };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_memory_alloc_for_device(
    device_index: u32,
    size_requested: u32,
    size_granted: *mut u32,
    memory_address: *mut *mut c_void,
) -> Gna2Status {
    let (status, return_null) = with_mock_state(|s| {
        s.memory_alloc_for_device_calls
            .push((device_index, size_requested));
        (
            s.memory_alloc_for_device_status,
            s.memory_alloc_for_device_return_null,
        )
    });

    if status != GNA2_STATUS_SUCCESS {
        return status;
    }

    if !size_granted.is_null() {
        unsafe { *size_granted = size_requested };
    }

    if return_null {
        if !memory_address.is_null() {
            unsafe { *memory_address = std::ptr::null_mut() };
        }
        return GNA2_STATUS_SUCCESS;
    }

    let Some(ptr) = alloc_tracked(size_requested) else {
        return -8;
    };
    if !memory_address.is_null() {
        unsafe { *memory_address = ptr };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_memory_free(memory: *mut c_void) -> Gna2Status {
    if !memory.is_null()
        && let Some(layout) = allocations().lock().unwrap().remove(&(memory as usize))
    {
        unsafe { dealloc(memory as *mut u8, layout) };
    }
    with_mock_state(|s| {
        s.memory_free_calls.push(memory as usize);
        s.memory_free_status
    })
}

unsafe extern "C" fn mock_memory_set_tag(memory: *mut c_void, tag: u32) -> Gna2Status {
    with_mock_state(|s| {
        s.memory_set_tag_calls.push((memory as usize, tag));
        s.memory_set_tag_status
    })
}

fn base_mock_symbol_table() -> GnaSymbolTable {
    GnaSymbolTable {
        device_get_count: mock_device_get_count,
        device_get_version: mock_device_get_version,
        device_open: mock_device_open,
        device_create_for_export: None,
        device_close: mock_device_close,
        device_set_number_of_threads: None,
        get_library_version: None,
        memory_alloc: mock_memory_alloc,
        memory_alloc_for_device: None,
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

fn full_mock_library() -> GnaLibrary {
    let mut symbols = base_mock_symbol_table();
    symbols.device_create_for_export = Some(mock_device_create_for_export);
    symbols.device_set_number_of_threads = Some(mock_device_set_number_of_threads);
    symbols.memory_alloc_for_device = Some(mock_memory_alloc_for_device);
    symbols.memory_set_tag = Some(mock_memory_set_tag);
    GnaLibrary::from_symbol_table(symbols)
}

// ---------------------------------------------------------------------------
// src/error.rs tests
// ---------------------------------------------------------------------------

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

#[test]
fn test_device_version_as_str_all_variants() {
    assert_eq!(
        Gna2DeviceVersion::SOFTWARE_EMULATION.as_str(),
        "Software Emulation"
    );
    assert_eq!(Gna2DeviceVersion::GMM.as_str(), "GNA GMM");
    assert_eq!(Gna2DeviceVersion::V0_9.as_str(), "GNA 0.9");
    assert_eq!(Gna2DeviceVersion::V1_0.as_str(), "GNA 1.0");
    assert_eq!(Gna2DeviceVersion::V2_0.as_str(), "GNA 2.0");
    assert_eq!(Gna2DeviceVersion::V3_0.as_str(), "GNA 3.0");
    assert_eq!(Gna2DeviceVersion::V3_5.as_str(), "GNA 3.5");
    assert_eq!(Gna2DeviceVersion::EMBEDDED_1_0.as_str(), "GNA Embedded 1.0");
    assert_eq!(Gna2DeviceVersion::EMBEDDED_3_1.as_str(), "GNA Embedded 3.1");
    assert_eq!(Gna2DeviceVersion(0x9999).as_str(), "Unknown GNA Version");
    assert_eq!(
        Gna2DeviceVersion::default(),
        Gna2DeviceVersion::SOFTWARE_EMULATION
    );
}

#[test]
fn test_data_type_size_in_bytes_all_variants() {
    assert_eq!(Gna2DataType::Boolean.size_in_bytes(), 1);
    assert_eq!(Gna2DataType::Int8.size_in_bytes(), 1);
    assert_eq!(Gna2DataType::Uint8.size_in_bytes(), 1);
    assert_eq!(Gna2DataType::Int16.size_in_bytes(), 2);
    assert_eq!(Gna2DataType::Uint16.size_in_bytes(), 2);
    assert_eq!(Gna2DataType::Int32.size_in_bytes(), 4);
    assert_eq!(Gna2DataType::Uint32.size_in_bytes(), 4);
    assert_eq!(Gna2DataType::Int64.size_in_bytes(), 8);
    assert_eq!(Gna2DataType::Uint64.size_in_bytes(), 8);

    for dt in [
        Gna2DataType::None,
        Gna2DataType::Int4,
        Gna2DataType::Uint4,
        Gna2DataType::CompoundBias,
        Gna2DataType::PwlSegment,
        Gna2DataType::WeightScaleFactor,
    ] {
        assert_eq!(dt.size_in_bytes(), 1);
    }
    assert_eq!(Gna2DataType::default(), Gna2DataType::None);
}

#[test]
fn test_shape_tensor_operation_defaults_and_constructors() {
    let scalar = Gna2Shape::scalar();
    assert_eq!(scalar, Gna2Shape::default());
    assert_eq!(scalar.number_of_dimensions, 0);
    assert_eq!(scalar.total_elements(), 1);

    let s1 = Gna2Shape::d1(16);
    assert_eq!(s1.number_of_dimensions, 1);
    assert_eq!(s1.dimensions[0], 16);
    assert_eq!(s1.total_elements(), 16);

    let s2 = Gna2Shape::d2(3, 4);
    assert_eq!(s2.number_of_dimensions, 2);
    assert_eq!(s2.total_elements(), 12);

    let mut dummy_val: i16 = 42;
    let dummy_ptr = (&raw mut dummy_val).cast::<c_void>();

    let t_default = Gna2Tensor::default();
    assert_eq!(t_default.shape, Gna2Shape::default());
    assert_eq!(t_default.mode, Gna2TensorMode::Default);
    assert_eq!(t_default.layout, [0; 8]);
    assert_eq!(t_default.data_type, Gna2DataType::None);
    assert!(t_default.data.is_null());

    let t_disabled = Gna2Tensor::disabled();
    assert_eq!(t_disabled.mode, Gna2TensorMode::Disabled);
    assert!(t_disabled.data.is_null());

    let t_d1 = Gna2Tensor::d1(8, Gna2DataType::Int16, dummy_ptr);
    assert_eq!(t_d1.shape, Gna2Shape::d1(8));
    assert_eq!(t_d1.data_type, Gna2DataType::Int16);
    assert_eq!(t_d1.data, dummy_ptr);

    let t_d2 = Gna2Tensor::d2(2, 4, Gna2DataType::Int32, dummy_ptr);
    assert_eq!(t_d2.shape, Gna2Shape::d2(2, 4));
    assert_eq!(t_d2.data_type, Gna2DataType::Int32);
    assert_eq!(t_d2.data, dummy_ptr);

    let op = Gna2Operation::default();
    assert_eq!(op.operation_type, Gna2OperationType::None);
    assert!(op.operands.is_null());
    assert_eq!(op.number_of_operands, 0);
    assert!(op.parameters.is_null());
    assert_eq!(op.number_of_parameters, 0);

    let mut op_copy = op;
    let model = Gna2Model {
        number_of_operations: 1,
        operations: &raw mut op_copy,
    };
    assert_eq!(model.number_of_operations, 1);

    let item = Gna2ModelItem {
        item_type: Gna2ItemType::OPERATION_TYPE,
        operation_index: 0,
        operand_index: GNA2_DISABLED,
        parameter_index: GNA2_DEFAULT,
        shape_dimension_index: -1,
        properties: [0; 4],
    };
    let model_err = Gna2ModelError {
        source: item,
        reason: Gna2ErrorType::NOT_ALIGNED,
        value: 64,
    };
    assert_eq!(model_err.reason, Gna2ErrorType::NOT_ALIGNED);
    assert_eq!(Gna2AccelerationMode::default(), Gna2AccelerationMode::Auto);
    assert_eq!(
        Gna2InstrumentationUnit::default(),
        Gna2InstrumentationUnit::Microseconds
    );
    assert_eq!(
        Gna2InstrumentationMode::default(),
        Gna2InstrumentationMode::TotalStall
    );
    assert_eq!(
        Gna2ModelExportComponent::default(),
        Gna2ModelExportComponent::LayerDescriptors
    );
    assert_eq!(Gna2BiasMode::default(), Gna2BiasMode::Default);
    assert_eq!(Gna2DeviceGeneration::Gen3_0 as i32, 0x300);
    assert_eq!(Gna2InstrumentationPoint::LibExecution as i32, 3);
    assert_eq!(Gna2MemoryTag::Input as i32, 0x0200);
}

// ---------------------------------------------------------------------------
// src/device.rs tests
// ---------------------------------------------------------------------------

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

#[test]
fn test_buffer_new_error_branches() {
    reset_mock_state();
    let lib = full_mock_library();

    // Zero size
    let err = GnaBuffer::new(&lib, 0).err().unwrap();
    assert!(matches!(err, GnaError::InvalidBufferSize(0)));
    assert!(read_mock_state(|s| s.memory_alloc_calls.is_empty()));

    // Non-success status
    with_mock_state(|s| s.memory_alloc_status = -8);
    let err = GnaBuffer::new(&lib, 256).err().unwrap();
    assert!(matches!(err, GnaError::StatusError { status: -8, .. }));

    // Null pointer returned with success status
    with_mock_state(|s| {
        s.memory_alloc_status = GNA2_STATUS_SUCCESS;
        s.memory_alloc_return_null = true;
    });
    let err = GnaBuffer::new(&lib, 256).err().unwrap();
    assert!(matches!(err, GnaError::NullPointer));
}

#[test]
fn test_buffer_new_success_slices_deref_and_drop() {
    reset_mock_state();
    let lib = full_mock_library();

    let raw_addr;
    {
        let mut buf = GnaBuffer::new(&lib, 128).unwrap();
        assert_eq!(buf.len(), 128);
        assert!(!buf.is_empty());
        raw_addr = buf.as_raw_ptr() as usize;
        assert_ne!(raw_addr, 0);

        // Mutate via as_mut_slice and read via as_slice
        buf.as_mut_slice()[0] = 0xAB;
        buf.as_mut_slice()[127] = 0xCD;
        assert_eq!(buf.as_slice()[0], 0xAB);
        assert_eq!(buf.as_slice()[127], 0xCD);

        // Mutate via DerefMut and read via Deref
        buf[1] = 0xEF;
        assert_eq!(buf[1], 0xEF);
        assert_eq!(buf.len(), 128);
        assert!(read_mock_state(|s| s.memory_free_calls.is_empty()));
    }

    assert_eq!(
        read_mock_state(|s| s.memory_free_calls.clone()),
        vec![raw_addr]
    );
}

#[test]
fn test_buffer_is_empty_when_granted_zero() {
    reset_mock_state();
    let lib = full_mock_library();

    with_mock_state(|s| s.memory_alloc_granted_override = Some(0));
    let raw_addr;
    {
        let buf = GnaBuffer::new(&lib, 64).unwrap();
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
        assert!(buf.as_slice().is_empty());
        raw_addr = buf.as_raw_ptr() as usize;
    }
    assert_eq!(
        read_mock_state(|s| s.memory_free_calls.clone()),
        vec![raw_addr]
    );
}

#[test]
fn test_buffer_new_for_device_all_branches() {
    reset_mock_state();
    let lib = full_mock_library();

    // Zero size
    let err = GnaBuffer::new_for_device(&lib, 0, 0).err().unwrap();
    assert!(matches!(err, GnaError::InvalidBufferSize(0)));

    // Fallback to memory_alloc when memory_alloc_for_device is None
    let lib_no_dev_alloc = GnaLibrary::from_symbol_table(base_mock_symbol_table());
    {
        let buf = GnaBuffer::new_for_device(&lib_no_dev_alloc, 3, 64).unwrap();
        assert_eq!(buf.len(), 64);
        assert_eq!(read_mock_state(|s| s.memory_alloc_calls.clone()), vec![64]);
        assert!(read_mock_state(|s| s
            .memory_alloc_for_device_calls
            .is_empty()));
    }

    // Error status from memory_alloc_for_device
    reset_mock_state();
    with_mock_state(|s| s.memory_alloc_for_device_status = -9);
    let err = GnaBuffer::new_for_device(&lib, 2, 128).err().unwrap();
    assert!(matches!(err, GnaError::StatusError { status: -9, .. }));
    assert_eq!(
        read_mock_state(|s| s.memory_alloc_for_device_calls.clone()),
        vec![(2, 128)]
    );

    // Null pointer returned with success status
    reset_mock_state();
    with_mock_state(|s| s.memory_alloc_for_device_return_null = true);
    let err = GnaBuffer::new_for_device(&lib, 1, 64).err().unwrap();
    assert!(matches!(err, GnaError::NullPointer));

    // Success
    reset_mock_state();
    let buf = GnaBuffer::new_for_device(&lib, 1, 96).unwrap();
    assert_eq!(buf.len(), 96);
    assert_eq!(
        read_mock_state(|s| s.memory_alloc_for_device_calls.clone()),
        vec![(1, 96)]
    );
}

#[test]
fn test_buffer_set_tag_all_branches() {
    reset_mock_state();

    // Missing symbol (covers library_symbol! None branch)
    let lib_no_tag = GnaLibrary::from_symbol_table(base_mock_symbol_table());
    let mut buf_no_tag = GnaBuffer::new(&lib_no_tag, 32).unwrap();
    let err = buf_no_tag.set_tag(Gna2MemoryTag::Input as u32).unwrap_err();
    assert!(matches!(err, GnaError::Other(msg) if msg.contains("memory_set_tag not supported")));
    drop(buf_no_tag);

    // Error status from memory_set_tag
    let lib = full_mock_library();
    let mut buf = GnaBuffer::new(&lib, 32).unwrap();
    let ptr_addr = buf.as_raw_ptr() as usize;

    with_mock_state(|s| s.memory_set_tag_status = -5);
    let err = buf.set_tag(Gna2MemoryTag::Output as u32).unwrap_err();
    assert!(matches!(err, GnaError::StatusError { status: -5, .. }));

    // Success status
    with_mock_state(|s| s.memory_set_tag_status = GNA2_STATUS_SUCCESS);
    buf.set_tag(Gna2MemoryTag::Scratch as u32).unwrap();
    assert_eq!(
        read_mock_state(|s| s.memory_set_tag_calls.clone()),
        vec![
            (ptr_addr, Gna2MemoryTag::Output as u32),
            (ptr_addr, Gna2MemoryTag::Scratch as u32),
        ]
    );
}

#[test]
fn test_buffer_send_across_threads() {
    reset_mock_state();
    let lib = full_mock_library();
    let mut buf = GnaBuffer::new(&lib, 32).unwrap();
    buf[0] = 0x55;

    let handle = std::thread::spawn(move || {
        assert_eq!(buf[0], 0x55);
        buf[31] = 0xAA;
        assert_eq!(buf.as_slice()[31], 0xAA);
        drop(buf);
    });
    handle.join().unwrap();
}

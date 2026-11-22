use std::ffi::c_void;
use std::ptr;

use super::*;
use crate::device::GnaDevice;
use crate::error::GnaError;
use crate::types::device::{Gna2DeviceVersion, Gna2Status};
use crate::types::model::{Gna2Model, Gna2Tensor};

unsafe extern "C" fn null_allocator(_size: u32) -> *mut c_void {
    ptr::null_mut()
}

unsafe extern "C" fn heap_fca_allocator(_size: u32) -> *mut c_void {
    unsafe { std::alloc::alloc(std::alloc::Layout::new::<[*const Gna2Tensor; 5]>()) as *mut c_void }
}

unsafe extern "C" fn mock_device_get_count(device_count: *mut u32) -> Gna2Status {
    unsafe { *device_count = 1 };
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_get_version(
    _device_index: u32,
    device_version: *mut Gna2DeviceVersion,
) -> Gna2Status {
    unsafe { *device_version = Gna2DeviceVersion::SOFTWARE_EMULATION };
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_open(_device_index: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_close(_device_index: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_memory_alloc(
    _size_requested: u32,
    _size_granted: *mut u32,
    _memory_address: *mut *mut c_void,
) -> Gna2Status {
    -4
}

unsafe extern "C" fn mock_memory_free(_memory: *mut c_void) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_model_create(
    _device_index: u32,
    model: *const Gna2Model,
    model_id: *mut u32,
) -> Gna2Status {
    unsafe {
        let op = &*(*model).operations;
        std::alloc::dealloc(
            op.operands as *mut u8,
            std::alloc::Layout::new::<[*const Gna2Tensor; 5]>(),
        );
        *model_id = 1;
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
        device_set_number_of_threads: None,
        get_library_version: None,
        memory_alloc: mock_memory_alloc,
        memory_alloc_for_device: None,
        memory_free: mock_memory_free,
        memory_set_tag: None,
        model_create: Some(mock_model_create),
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

#[test]
fn test_allocate_fca_operands_null_allocator_returns_err() {
    let fca = FullyConnectedAffineLayer {
        input: Gna2Tensor::disabled(),
        output: Gna2Tensor::disabled(),
        weights: Gna2Tensor::disabled(),
        biases: Gna2Tensor::disabled(),
        activation: Gna2Tensor::disabled(),
    };
    let err = unsafe { allocate_fca_operands(null_allocator, &fca) }.unwrap_err();
    assert_eq!(err.to_string(), GnaError::NullPointer.to_string());
}

#[test]
fn test_build_with_null_allocator_returns_err() {
    let library = GnaLibrary::from_symbol_table(mock_symbol_table());
    let device = GnaDevice::open(&library, 0).expect("open device");
    let err = GnaModelBuilder::new()
        .add_fully_connected_affine(
            Gna2Tensor::disabled(),
            Gna2Tensor::disabled(),
            Gna2Tensor::disabled(),
            Gna2Tensor::disabled(),
            None,
        )
        .build_with_allocator(&device, null_allocator)
        .err()
        .expect("expected NullPointer error");
    assert_eq!(err.to_string(), GnaError::NullPointer.to_string());
    assert_eq!(
        unsafe { (library.symbols().memory_alloc)(0, ptr::null_mut(), ptr::null_mut()) },
        -4
    );
    assert_eq!(
        unsafe { (library.symbols().memory_free)(ptr::null_mut()) },
        GNA2_STATUS_SUCCESS
    );
}

#[test]
fn test_build_with_non_null_allocator_succeeds() {
    let library = GnaLibrary::from_symbol_table(mock_symbol_table());
    let device = GnaDevice::open(&library, 0).expect("open device");
    let model = GnaModelBuilder::new()
        .add_fully_connected_affine(
            Gna2Tensor::disabled(),
            Gna2Tensor::disabled(),
            Gna2Tensor::disabled(),
            Gna2Tensor::disabled(),
            None,
        )
        .build_with_allocator(&device, heap_fca_allocator)
        .expect("build model");
    assert_eq!(model.id(), 1);
}

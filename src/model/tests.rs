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
        let op = &*(*model).operations();
        std::alloc::dealloc(
            op.operands() as *mut u8,
            std::alloc::Layout::new::<[*const Gna2Tensor; 5]>(),
        );
        *model_id = 1;
    }
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
    .with_model_create(mock_model_create)
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
        unsafe { (library.symbols().memory_alloc())(0, ptr::null_mut(), ptr::null_mut()) },
        -4
    );
    assert_eq!(
        unsafe { (library.symbols().memory_free())(ptr::null_mut()) },
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

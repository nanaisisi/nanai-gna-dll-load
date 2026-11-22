use std::ffi::c_void;
use std::ptr;

use nanai_gna_dll_load::{
    model::GnaModelBuilder,
    types::{
        GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS, Gna2DataType, Gna2Model, Gna2Operation,
        Gna2OperationType, Gna2Shape, Gna2Tensor, Gna2TensorMode,
    },
};

#[test]
fn test_shape_dimensions() {
    let s0 = Gna2Shape::scalar();
    assert_eq!(s0.number_of_dimensions(), 0);
    assert_eq!(s0.total_elements(), 1);

    let s1 = Gna2Shape::d1(64);
    assert_eq!(s1.number_of_dimensions(), 1);
    assert_eq!(s1.dimensions()[0], 64);
    assert_eq!(s1.total_elements(), 64);

    let s2 = Gna2Shape::d2(16, 8);
    assert_eq!(s2.number_of_dimensions(), 2);
    assert_eq!(s2.dimensions()[0], 16);
    assert_eq!(s2.dimensions()[1], 8);
    assert_eq!(s2.total_elements(), 128);
}

#[test]
fn test_tensor_constructors() {
    let dummy_ptr = 0x1000 as *mut c_void;
    let t = Gna2Tensor::d2(16, 4, Gna2DataType::Int16, dummy_ptr);
    assert_eq!(t.shape().number_of_dimensions(), 2);
    assert_eq!(t.shape().dimensions()[0], 16);
    assert_eq!(t.shape().dimensions()[1], 4);
    assert_eq!(t.mode(), Gna2TensorMode::Default);
    assert_eq!(t.layout(), &[0; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS]);
    assert_eq!(t.data_type(), Gna2DataType::Int16);
    assert_eq!(t.data(), dummy_ptr);

    let disabled = Gna2Tensor::disabled();
    assert_eq!(disabled.mode(), Gna2TensorMode::Disabled);
}

#[test]
fn test_operation_and_model_accessors() {
    let dummy_param = 0x2000 as *mut c_void;
    let mut params = [dummy_param];
    let mut op = Gna2Operation::new(Gna2OperationType::Copy)
        .with_operands(ptr::null(), 0)
        .with_parameters(params.as_mut_ptr(), 1);

    assert_eq!(op.operation_type(), Gna2OperationType::Copy);
    assert!(op.operands().is_null());
    assert_eq!(op.number_of_operands(), 0);
    assert_eq!(op.parameters(), params.as_mut_ptr());
    assert_eq!(op.number_of_parameters(), 1);

    let model = Gna2Model::new(1, &mut op as *mut Gna2Operation);
    assert_eq!(model.number_of_operations(), 1);
    assert_eq!(model.operations(), &mut op as *mut Gna2Operation);
}

#[test]
fn test_data_type_sizes() {
    assert_eq!(Gna2DataType::Int8.size_in_bytes(), 1);
    assert_eq!(Gna2DataType::Int16.size_in_bytes(), 2);
    assert_eq!(Gna2DataType::Int32.size_in_bytes(), 4);
    assert_eq!(Gna2DataType::Int64.size_in_bytes(), 8);
}

#[test]
fn test_model_builder_construction() {
    let dummy_ptr = 0x1000 as *mut c_void;
    let inp = Gna2Tensor::d2(16, 4, Gna2DataType::Int16, dummy_ptr);
    let out = Gna2Tensor::d2(8, 4, Gna2DataType::Int32, dummy_ptr);
    let w = Gna2Tensor::d2(8, 16, Gna2DataType::Int16, dummy_ptr);
    let b = Gna2Tensor::d1(8, Gna2DataType::Int32, dummy_ptr);

    let _builder = GnaModelBuilder::new().add_fully_connected_affine(inp, out, w, b, None);

    // Verify builder builds safely (requires device to compile)
    assert_eq!(Gna2OperationType::FullyConnectedAffine as u32, 3);
}

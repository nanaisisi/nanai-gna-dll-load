use std::ffi::c_void;

use nanai_gna_dll_load::types::{
    GNA2_DEFAULT, GNA2_DISABLED, Gna2AccelerationMode, Gna2BiasMode, Gna2DataType,
    Gna2DeviceGeneration, Gna2DeviceVersion, Gna2ErrorType, Gna2InstrumentationMode,
    Gna2InstrumentationPoint, Gna2InstrumentationUnit, Gna2ItemType, Gna2MemoryTag, Gna2Model,
    Gna2ModelError, Gna2ModelExportComponent, Gna2ModelItem, Gna2Operation, Gna2OperationType,
    Gna2Shape, Gna2Tensor, Gna2TensorMode,
};

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

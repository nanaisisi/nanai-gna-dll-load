use super::GNA2_MODEL_ITEM_NUMBER_OF_PROPERTIES;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Gna2ItemType(pub i32);

impl Gna2ItemType {
    pub const NONE: Self = Self(-1);
    pub const MODEL_NUMBER_OF_OPERATIONS: Self = Self(0);
    pub const MODEL_OPERATIONS: Self = Self(1);
    pub const OPERATION_TYPE: Self = Self(3);
    pub const OPERATION_OPERANDS: Self = Self(4);
    pub const OPERATION_NUMBER_OF_OPERANDS: Self = Self(5);
    pub const OPERATION_PARAMETERS: Self = Self(6);
    pub const OPERATION_NUMBER_OF_PARAMETERS: Self = Self(7);
    pub const OPERAND_MODE: Self = Self(8);
    pub const OPERAND_LAYOUT: Self = Self(9);
    pub const OPERAND_TYPE: Self = Self(10);
    pub const OPERAND_DATA: Self = Self(11);
    pub const PARAMETER: Self = Self(12);
    pub const SHAPE_NUMBER_OF_DIMENSIONS: Self = Self(13);
    pub const SHAPE_DIMENSIONS: Self = Self(14);
    pub const INTERNAL: Self = Self(15);
    pub const OPERATION_HARDWARE_DESCRIPTOR: Self = Self(16);
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Gna2ModelItem {
    pub item_type: Gna2ItemType,
    pub operation_index: i32,
    pub operand_index: i32,
    pub parameter_index: i32,
    pub shape_dimension_index: i32,
    pub properties: [i32; GNA2_MODEL_ITEM_NUMBER_OF_PROPERTIES],
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Gna2ErrorType(pub i32);

impl Gna2ErrorType {
    pub const NONE: Self = Self(0);
    pub const NOT_TRUE: Self = Self(-1);
    pub const NOT_FALSE: Self = Self(-2);
    pub const NULL_NOT_ALLOWED: Self = Self(-3);
    pub const NULL_REQUIRED: Self = Self(-4);
    pub const BELOW_RANGE: Self = Self(-5);
    pub const ABOVE_RANGE: Self = Self(-6);
    pub const NOT_EQUAL: Self = Self(-7);
    pub const NOT_GT_ZERO: Self = Self(-8);
    pub const NOT_ZERO: Self = Self(-9);
    pub const NOT_ONE: Self = Self(-10);
    pub const NOT_IN_SET: Self = Self(-11);
    pub const NOT_MULTIPLICITY: Self = Self(-12);
    pub const NOT_SUCCESS: Self = Self(-13);
    pub const NOT_ALIGNED: Self = Self(-14);
    pub const ARGUMENT_MISSING: Self = Self(-15);
    pub const ARGUMENT_INVALID: Self = Self(-16);
    pub const RUNTIME: Self = Self(-17);
    pub const NO_HARDWARE_COMPLIANT_OPERATION: Self = Self(-18);
    pub const OTHER: Self = Self(-19);
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Gna2ModelError {
    pub source: Gna2ModelItem,
    pub reason: Gna2ErrorType,
    pub value: i64,
}

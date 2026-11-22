use super::GNA2_MODEL_ITEM_NUMBER_OF_PROPERTIES;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Gna2ItemType(pub(crate) i32);

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

    pub const fn new(raw: i32) -> Self {
        Self(raw)
    }

    pub const fn raw(&self) -> i32 {
        self.0
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Gna2ModelItem {
    pub(crate) item_type: Gna2ItemType,
    pub(crate) operation_index: i32,
    pub(crate) operand_index: i32,
    pub(crate) parameter_index: i32,
    pub(crate) shape_dimension_index: i32,
    pub(crate) properties: [i32; GNA2_MODEL_ITEM_NUMBER_OF_PROPERTIES],
}

impl Gna2ModelItem {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn with_item_type(mut self, item_type: Gna2ItemType) -> Self {
        self.item_type = item_type;
        self
    }

    pub const fn with_operation_index(mut self, operation_index: i32) -> Self {
        self.operation_index = operation_index;
        self
    }

    pub const fn with_operand_index(mut self, operand_index: i32) -> Self {
        self.operand_index = operand_index;
        self
    }

    pub const fn with_parameter_index(mut self, parameter_index: i32) -> Self {
        self.parameter_index = parameter_index;
        self
    }

    pub const fn with_shape_dimension_index(mut self, shape_dimension_index: i32) -> Self {
        self.shape_dimension_index = shape_dimension_index;
        self
    }

    pub const fn with_properties(
        mut self,
        properties: [i32; GNA2_MODEL_ITEM_NUMBER_OF_PROPERTIES],
    ) -> Self {
        self.properties = properties;
        self
    }

    pub const fn item_type(&self) -> Gna2ItemType {
        self.item_type
    }

    pub const fn operation_index(&self) -> i32 {
        self.operation_index
    }

    pub const fn operand_index(&self) -> i32 {
        self.operand_index
    }

    pub const fn parameter_index(&self) -> i32 {
        self.parameter_index
    }

    pub const fn shape_dimension_index(&self) -> i32 {
        self.shape_dimension_index
    }

    pub const fn properties(&self) -> &[i32; GNA2_MODEL_ITEM_NUMBER_OF_PROPERTIES] {
        &self.properties
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Gna2ErrorType(pub(crate) i32);

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

    pub const fn new(raw: i32) -> Self {
        Self(raw)
    }

    pub const fn raw(&self) -> i32 {
        self.0
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Gna2ModelError {
    pub(crate) source: Gna2ModelItem,
    pub(crate) reason: Gna2ErrorType,
    pub(crate) value: i64,
}

impl Gna2ModelError {
    pub const fn new(source: Gna2ModelItem, reason: Gna2ErrorType, value: i64) -> Self {
        Self {
            source,
            reason,
            value,
        }
    }

    pub const fn source(&self) -> &Gna2ModelItem {
        &self.source
    }

    pub const fn reason(&self) -> Gna2ErrorType {
        self.reason
    }

    pub const fn value(&self) -> i64 {
        self.value
    }
}

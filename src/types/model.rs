pub const GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS: usize = 8;
pub const GNA2_MODEL_ITEM_NUMBER_OF_PROPERTIES: usize = 4;
pub const GNA2_DISABLED: i32 = -1;
pub const GNA2_DEFAULT: i32 = 0;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Gna2OperationType {
    #[default]
    None = 0,
    Convolution = 1,
    Copy = 2,
    FullyConnectedAffine = 3,
    ElementWiseAffine = 4,
    Gmm = 5,
    Recurrent = 6,
    Transposition = 7,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Gna2DataType {
    #[default]
    None = 0,
    Boolean = 1,
    Int4 = 2,
    Int8 = 3,
    Int16 = 4,
    Int32 = 5,
    Int64 = 6,
    Uint4 = 7,
    Uint8 = 8,
    Uint16 = 9,
    Uint32 = 10,
    Uint64 = 11,
    CompoundBias = 12,
    PwlSegment = 13,
    WeightScaleFactor = 14,
}

impl Gna2DataType {
    pub fn size_in_bytes(&self) -> usize {
        match *self {
            Self::Boolean => 1,
            Self::Int8 | Self::Uint8 => 1,
            Self::Int16 | Self::Uint16 => 2,
            Self::Int32 | Self::Uint32 => 4,
            Self::Int64 | Self::Uint64 => 8,
            _ => 1,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Gna2TensorMode {
    #[default]
    Default = 0,
    ConstantScalar = 0x010000,
    ExternalBuffer = 0x001000,
    Disabled = -1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Gna2BiasMode {
    #[default]
    Default = 0,
    PerStride = 1,
    Grouping = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gna2Shape {
    pub number_of_dimensions: u32,
    pub dimensions: [u32; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
}

impl Default for Gna2Shape {
    fn default() -> Self {
        Self {
            number_of_dimensions: 0,
            dimensions: [0; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
        }
    }
}

impl Gna2Shape {
    pub fn scalar() -> Self {
        Self::default()
    }

    pub fn d1(x: u32) -> Self {
        let mut s = Self {
            number_of_dimensions: 1,
            dimensions: [0; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
        };
        s.dimensions[0] = x;
        s
    }

    pub fn d2(x: u32, y: u32) -> Self {
        let mut s = Self {
            number_of_dimensions: 2,
            dimensions: [0; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
        };
        s.dimensions[0] = x;
        s.dimensions[1] = y;
        s
    }

    pub fn total_elements(&self) -> usize {
        if self.number_of_dimensions == 0 {
            return 1;
        }
        let mut total = 1;
        for &dimension in self.dimensions[..]
            .iter()
            .take(self.number_of_dimensions as usize)
        {
            total *= dimension as usize;
        }
        total
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Gna2Tensor {
    pub shape: Gna2Shape,
    pub mode: Gna2TensorMode,
    pub layout: [std::ffi::c_char; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
    pub data_type: Gna2DataType,
    pub data: *mut std::ffi::c_void,
}

impl Default for Gna2Tensor {
    fn default() -> Self {
        Self {
            shape: Gna2Shape::default(),
            mode: Gna2TensorMode::Default,
            layout: [0; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
            data_type: Gna2DataType::None,
            data: std::ptr::null_mut(),
        }
    }
}

impl Gna2Tensor {
    pub fn disabled() -> Self {
        Self {
            shape: Gna2Shape::default(),
            mode: Gna2TensorMode::Disabled,
            layout: [0; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
            data_type: Gna2DataType::None,
            data: std::ptr::null_mut(),
        }
    }

    pub fn d1(x: u32, data_type: Gna2DataType, data: *mut std::ffi::c_void) -> Self {
        Self {
            shape: Gna2Shape::d1(x),
            mode: Gna2TensorMode::Default,
            layout: [0; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
            data_type,
            data,
        }
    }

    pub fn d2(x: u32, y: u32, data_type: Gna2DataType, data: *mut std::ffi::c_void) -> Self {
        Self {
            shape: Gna2Shape::d2(x, y),
            mode: Gna2TensorMode::Default,
            layout: [0; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
            data_type,
            data,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Gna2Operation {
    pub operation_type: Gna2OperationType,
    pub operands: *const *const Gna2Tensor,
    pub number_of_operands: u32,
    pub parameters: *mut *mut std::ffi::c_void,
    pub number_of_parameters: u32,
}

impl Default for Gna2Operation {
    fn default() -> Self {
        Self {
            operation_type: Gna2OperationType::None,
            operands: std::ptr::null(),
            number_of_operands: 0,
            parameters: std::ptr::null_mut(),
            number_of_parameters: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Gna2Model {
    pub number_of_operations: u32,
    pub operations: *mut Gna2Operation,
}

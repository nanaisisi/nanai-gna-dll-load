pub type Gna2Status = i32;

pub const GNA2_STATUS_SUCCESS: Gna2Status = 0;
pub const GNA2_STATUS_WARNING_DEVICE_BUSY: Gna2Status = 1;
pub const GNA2_STATUS_WARNING_ARITHMETIC_SATURATION: Gna2Status = 2;
pub const GNA2_STATUS_MODEL_ERROR_UNAVAILABLE: Gna2Status = 3;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Gna2DeviceVersion(pub u32);

impl Gna2DeviceVersion {
    pub const SOFTWARE_EMULATION: Self = Self(0);
    pub const GMM: Self = Self(0x01);
    pub const V0_9: Self = Self(0x09);
    pub const V1_0: Self = Self(0x10);
    pub const V2_0: Self = Self(0x20);
    pub const V3_0: Self = Self(0x30);
    pub const V3_5: Self = Self(0x35);
    pub const EMBEDDED_1_0: Self = Self(0x10E);
    pub const EMBEDDED_3_1: Self = Self(0x310E);

    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::SOFTWARE_EMULATION => "Software Emulation",
            Self::GMM => "GNA GMM",
            Self::V0_9 => "GNA 0.9",
            Self::V1_0 => "GNA 1.0",
            Self::V2_0 => "GNA 2.0",
            Self::V3_0 => "GNA 3.0",
            Self::V3_5 => "GNA 3.5",
            Self::EMBEDDED_1_0 => "GNA Embedded 1.0",
            Self::EMBEDDED_3_1 => "GNA Embedded 3.1",
            _ => "Unknown GNA Version",
        }
    }
}

/// GNA Device Generations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gna2DeviceGeneration {
    Gmm = 0x010,
    Gen0_9 = 0x090,
    Gen1_0 = 0x100,
    Gen2_0 = 0x200,
    Gen3_0 = 0x300,
    Gen3_1 = 0x310,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Gna2AccelerationMode {
    #[default]
    Auto = 0,
    Software = 1,
    Hardware = 2,
    Avx2 = 3,
    Avx1 = 4,
    Sse4x2 = 5,
    Generic = 6,
    HardwareWithSoftwareFallback = 7,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gna2InstrumentationPoint {
    LibPreprocessing = 0,
    LibSubmission = 1,
    LibProcessing = 2,
    LibExecution = 3,
    LibDeviceRequestReady = 4,
    LibDeviceRequestSent = 5,
    LibDeviceRequestCompleted = 6,
    LibCompletion = 7,
    LibReceived = 8,
    DrvPreprocessing = 9,
    DrvProcessing = 10,
    DrvDeviceRequestCompleted = 11,
    DrvCompletion = 12,
    HwTotalCycles = 13,
    HwStallCycles = 14,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Gna2InstrumentationUnit {
    #[default]
    Microseconds = 0,
    Milliseconds = 1,
    Cycles = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Gna2InstrumentationMode {
    #[default]
    TotalStall = 0,
    WaitForDmaCompletion = 1,
    WaitForMmuTranslation = 2,
    DescriptorFetchTime = 3,
    InputBufferFillFromMemory = 4,
    OutputBufferFullStall = 5,
    OutputBufferWaitForIosfStall = 6,
    Disabled = -1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Gna2ModelExportComponent {
    #[default]
    LayerDescriptors = 0,
    LayerDescriptorHeader = 1,
    LegacySueCreekDump = 2,
    LegacySueCreekHeader = 3,
    ReadOnlyDump = 4,
    ScratchDump = 6,
    StateDump = 7,
    InputDump = 11,
    OutputDump = 12,
    ExternalBufferInputDump = 21,
    ExternalBufferOutputDump = 22,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gna2MemoryTag {
    ReadWrite = 0x0100,
    Input = 0x0200,
    Output = 0x0400,
    ReadOnly = 0x0800,
    ExternalBufferInput = 0x1000,
    ExternalBufferOutput = 0x2000,
    Scratch = 0x4000,
    State = 0x8000,
}

pub type Gna2UserAllocator = unsafe extern "C" fn(size: u32) -> *mut std::ffi::c_void;

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

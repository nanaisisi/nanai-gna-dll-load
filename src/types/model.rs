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
    number_of_dimensions: u32,
    dimensions: [u32; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
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
    pub const fn number_of_dimensions(&self) -> u32 {
        self.number_of_dimensions
    }

    pub const fn dimensions(&self) -> &[u32; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS] {
        &self.dimensions
    }

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
        Self::from_dims(&[x, y])
    }

    pub fn d3(x: u32, y: u32, z: u32) -> Self {
        Self::from_dims(&[x, y, z])
    }

    pub fn d4(x: u32, y: u32, z: u32, w: u32) -> Self {
        Self::from_dims(&[x, y, z, w])
    }

    pub fn from_dims(dims: &[u32]) -> Self {
        let count = dims.len().min(GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS);
        let mut s = Self {
            number_of_dimensions: count as u32,
            dimensions: [0; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
        };
        s.dimensions[..count].copy_from_slice(&dims[..count]);
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
    shape: Gna2Shape,
    mode: Gna2TensorMode,
    layout: [std::ffi::c_char; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
    data_type: Gna2DataType,
    data: *mut std::ffi::c_void,
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
    pub const fn shape(&self) -> &Gna2Shape {
        &self.shape
    }

    pub const fn mode(&self) -> Gna2TensorMode {
        self.mode
    }

    pub const fn layout(&self) -> &[std::ffi::c_char; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS] {
        &self.layout
    }

    pub const fn data_type(&self) -> Gna2DataType {
        self.data_type
    }

    pub const fn data(&self) -> *mut std::ffi::c_void {
        self.data
    }

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
        Self::from_shape(Gna2Shape::d1(x), data_type, data)
    }

    pub fn d3(
        x: u32,
        y: u32,
        z: u32,
        data_type: Gna2DataType,
        data: *mut std::ffi::c_void,
    ) -> Self {
        Self::from_shape(Gna2Shape::d3(x, y, z), data_type, data)
    }

    pub fn d4(
        x: u32,
        y: u32,
        z: u32,
        w: u32,
        data_type: Gna2DataType,
        data: *mut std::ffi::c_void,
    ) -> Self {
        Self::from_shape(Gna2Shape::d4(x, y, z, w), data_type, data)
    }

    pub fn from_shape(
        shape: Gna2Shape,
        data_type: Gna2DataType,
        data: *mut std::ffi::c_void,
    ) -> Self {
        Self {
            shape,
            mode: Gna2TensorMode::Default,
            layout: [0; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS],
            data_type,
            data,
        }
    }

    pub fn with_layout(mut self, layout: &str) -> Self {
        self.layout = [0; GNA2_SHAPE_MAXIMUM_NUMBER_OF_DIMENSIONS];
        for (dst, &b) in self.layout.iter_mut().zip(layout.as_bytes().iter()) {
            *dst = b as std::ffi::c_char;
        }
        self
    }

    /// Returns `true` if `data` is null or aligned to `alignment` bytes.
    pub fn is_data_aligned(&self, alignment: usize) -> bool {
        self.data.is_null() || (alignment > 0 && (self.data as usize).is_multiple_of(alignment))
    }

    /// Validate that `data` is aligned to `alignment` bytes (e.g. 64 or 4096).
    pub fn validate_alignment(&self, alignment: usize) -> crate::Result<()> {
        if !self.is_data_aligned(alignment) {
            return Err(crate::error::GnaError::Other(format!(
                "Tensor data pointer {:p} is not aligned to {} bytes",
                self.data, alignment
            )));
        }
        Ok(())
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
    operation_type: Gna2OperationType,
    operands: *const *const Gna2Tensor,
    number_of_operands: u32,
    parameters: *mut *mut std::ffi::c_void,
    number_of_parameters: u32,
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

impl Gna2Operation {
    pub fn new(operation_type: Gna2OperationType) -> Self {
        Self {
            operation_type,
            ..Self::default()
        }
    }

    pub fn with_operands(
        mut self,
        operands: *const *const Gna2Tensor,
        number_of_operands: u32,
    ) -> Self {
        self.operands = operands;
        self.number_of_operands = number_of_operands;
        self
    }

    pub fn with_parameters(
        mut self,
        parameters: *mut *mut std::ffi::c_void,
        number_of_parameters: u32,
    ) -> Self {
        self.parameters = parameters;
        self.number_of_parameters = number_of_parameters;
        self
    }

    pub const fn operation_type(&self) -> Gna2OperationType {
        self.operation_type
    }

    pub const fn operands(&self) -> *const *const Gna2Tensor {
        self.operands
    }

    pub const fn number_of_operands(&self) -> u32 {
        self.number_of_operands
    }

    pub const fn parameters(&self) -> *mut *mut std::ffi::c_void {
        self.parameters
    }

    pub const fn number_of_parameters(&self) -> u32 {
        self.number_of_parameters
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Gna2Model {
    number_of_operations: u32,
    operations: *mut Gna2Operation,
}

impl Gna2Model {
    pub const fn new(number_of_operations: u32, operations: *mut Gna2Operation) -> Self {
        Self {
            number_of_operations,
            operations,
        }
    }

    pub const fn number_of_operations(&self) -> u32 {
        self.number_of_operations
    }

    pub const fn operations(&self) -> *mut Gna2Operation {
        self.operations
    }
}

/// Piecewise linear (PWL) segment matching Intel GNA2 `Gna2PwlSegment`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Gna2PwlSegment {
    x_base: i32,
    y_base: i16,
    slope: i16,
}

impl Gna2PwlSegment {
    pub const fn new(x_base: i32, y_base: i16, slope: i16) -> Self {
        Self {
            x_base,
            y_base,
            slope,
        }
    }

    pub const fn x_base(&self) -> i32 {
        self.x_base
    }

    pub const fn y_base(&self) -> i16 {
        self.y_base
    }

    pub const fn slope(&self) -> i16 {
        self.slope
    }
}

/// Helper owning PWL activation segments and exposing a `Gna2Tensor` view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GnaPwlActivation {
    segments: Vec<Gna2PwlSegment>,
}

impl GnaPwlActivation {
    /// Create a PWL activation from custom segments.
    pub fn from_segments(segments: Vec<Gna2PwlSegment>) -> Self {
        Self { segments }
    }

    /// Create a ReLU PWL activation (`0` for `x < 0`, identity slope for `x >= 0`).
    pub fn relu() -> Self {
        Self::leaky_relu(0)
    }

    /// Create a Leaky-ReLU PWL activation with the given negative-region slope.
    pub fn leaky_relu(negative_slope: i16) -> Self {
        Self {
            segments: vec![
                Gna2PwlSegment {
                    x_base: i32::MIN,
                    y_base: 0,
                    slope: negative_slope,
                },
                Gna2PwlSegment {
                    x_base: 0,
                    y_base: 0,
                    slope: 1,
                },
            ],
        }
    }

    /// Access the underlying PWL segments.
    pub fn segments(&self) -> &[Gna2PwlSegment] {
        &self.segments
    }

    /// Build a `Gna2Tensor` (`Gna2DataType::PwlSegment`) pointing to this activation's segments.
    pub fn as_tensor(&mut self) -> Gna2Tensor {
        Gna2Tensor::d1(
            self.segments.len() as u32,
            Gna2DataType::PwlSegment,
            self.segments.as_mut_ptr() as *mut std::ffi::c_void,
        )
    }
}

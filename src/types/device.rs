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

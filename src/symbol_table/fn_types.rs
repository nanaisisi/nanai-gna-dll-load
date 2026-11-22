use std::ffi::c_char;
use std::ffi::c_void;

use crate::types::{
    Gna2AccelerationMode, Gna2DeviceVersion, Gna2InstrumentationMode, Gna2InstrumentationPoint,
    Gna2InstrumentationUnit, Gna2ModelExportComponent, Gna2Status, Gna2UserAllocator,
};

// Raw C function signatures
pub type FnGna2DeviceGetCount = unsafe extern "C" fn(device_count: *mut u32) -> Gna2Status;
pub type FnGna2DeviceGetVersion =
    unsafe extern "C" fn(device_index: u32, device_version: *mut Gna2DeviceVersion) -> Gna2Status;
pub type FnGna2DeviceOpen = unsafe extern "C" fn(device_index: u32) -> Gna2Status;
pub type FnGna2DeviceCreateForExport = unsafe extern "C" fn(
    target_device_version: Gna2DeviceVersion,
    device_index: *mut u32,
) -> Gna2Status;
pub type FnGna2DeviceClose = unsafe extern "C" fn(device_index: u32) -> Gna2Status;
pub type FnGna2DeviceSetNumberOfThreads =
    unsafe extern "C" fn(device_index: u32, number_of_threads: u32) -> Gna2Status;

pub type FnGna2GetLibraryVersion =
    unsafe extern "C" fn(version_buffer: *mut c_char, version_buffer_size: u32) -> Gna2Status;

pub type FnGna2MemoryAlloc = unsafe extern "C" fn(
    size_requested: u32,
    size_granted: *mut u32,
    memory_address: *mut *mut c_void,
) -> Gna2Status;
pub type FnGna2MemoryAllocForDevice = unsafe extern "C" fn(
    device_index: u32,
    size_requested: u32,
    size_granted: *mut u32,
    memory_address: *mut *mut c_void,
) -> Gna2Status;
pub type FnGna2MemoryFree = unsafe extern "C" fn(memory: *mut c_void) -> Gna2Status;
pub type FnGna2MemorySetTag = unsafe extern "C" fn(memory: *mut c_void, tag: u32) -> Gna2Status;

pub type FnGna2RequestConfigCreate =
    unsafe extern "C" fn(model_id: u32, request_config_id: *mut u32) -> Gna2Status;
pub type FnGna2RequestConfigSetOperandBuffer = unsafe extern "C" fn(
    request_config_id: u32,
    operation_index: u32,
    operand_index: u32,
    address: *mut c_void,
) -> Gna2Status;
pub type FnGna2RequestConfigEnableActiveList = unsafe extern "C" fn(
    request_config_id: u32,
    operation_index: u32,
    number_of_indices: u32,
    indices: *const u32,
) -> Gna2Status;
pub type FnGna2RequestConfigEnableHardwareConsistency =
    unsafe extern "C" fn(request_config_id: u32, device_version: Gna2DeviceVersion) -> Gna2Status;
pub type FnGna2RequestConfigSetAccelerationMode = unsafe extern "C" fn(
    request_config_id: u32,
    acceleration_mode: Gna2AccelerationMode,
) -> Gna2Status;
pub type FnGna2RequestConfigRelease = unsafe extern "C" fn(request_config_id: u32) -> Gna2Status;
pub type FnGna2RequestEnqueue =
    unsafe extern "C" fn(request_config_id: u32, request_id: *mut u32) -> Gna2Status;
pub type FnGna2RequestWait =
    unsafe extern "C" fn(request_id: u32, timeout_milliseconds: u32) -> Gna2Status;

pub type FnGna2InstrumentationConfigCreate = unsafe extern "C" fn(
    number_of_points: u32,
    selected_points: *const Gna2InstrumentationPoint,
    results: *mut u64,
    instrumentation_config_id: *mut u32,
) -> Gna2Status;
pub type FnGna2InstrumentationConfigAssignToRequestConfig =
    unsafe extern "C" fn(instrumentation_config_id: u32, request_config_id: u32) -> Gna2Status;
pub type FnGna2InstrumentationConfigSetUnit = unsafe extern "C" fn(
    instrumentation_config_id: u32,
    instrumentation_unit: Gna2InstrumentationUnit,
) -> Gna2Status;
pub type FnGna2InstrumentationConfigSetMode = unsafe extern "C" fn(
    instrumentation_config_id: u32,
    instrumentation_mode: Gna2InstrumentationMode,
) -> Gna2Status;
pub type FnGna2InstrumentationConfigRelease =
    unsafe extern "C" fn(instrumentation_config_id: u32) -> Gna2Status;

pub type FnGna2ModelExportConfigCreate = unsafe extern "C" fn(
    user_allocator: Gna2UserAllocator,
    export_config_id: *mut u32,
) -> Gna2Status;
pub type FnGna2ModelExportConfigRelease = unsafe extern "C" fn(export_config_id: u32) -> Gna2Status;
pub type FnGna2ModelExportConfigSetSource = unsafe extern "C" fn(
    export_config_id: u32,
    source_device_index: u32,
    source_model_id: u32,
) -> Gna2Status;
pub type FnGna2ModelExportConfigSetTarget = unsafe extern "C" fn(
    export_config_id: u32,
    target_device_version: Gna2DeviceVersion,
) -> Gna2Status;
pub type FnGna2ModelExport = unsafe extern "C" fn(
    export_config_id: u32,
    component_type: Gna2ModelExportComponent,
    export_buffer: *mut *mut c_void,
    export_buffer_size: *mut u32,
) -> Gna2Status;
pub type FnGna2ModelOverrideAlignment = unsafe extern "C" fn(new_alignment: u32) -> Gna2Status;

pub type FnGna2ModelCreate = unsafe extern "C" fn(
    device_index: u32,
    model: *const crate::types::Gna2Model,
    model_id: *mut u32,
) -> Gna2Status;
pub type FnGna2ModelRelease = unsafe extern "C" fn(model_id: u32) -> Gna2Status;
pub type FnGna2ModelGetLastError =
    unsafe extern "C" fn(error: *mut crate::types::Gna2ModelError) -> Gna2Status;
pub type FnGna2ModelErrorGetMessage = unsafe extern "C" fn(
    error: *const crate::types::Gna2ModelError,
    message_buffer: *mut c_char,
    message_buffer_size: u32,
) -> Gna2Status;
pub type FnGna2ModelErrorGetMaxMessageLength = unsafe extern "C" fn() -> u32;

pub type FnGna2OperationInitFullyConnectedAffine = unsafe extern "C" fn(
    operation: *mut crate::types::Gna2Operation,
    user_allocator: Gna2UserAllocator,
    inputs: *mut crate::types::Gna2Tensor,
    outputs: *mut crate::types::Gna2Tensor,
    weights: *mut crate::types::Gna2Tensor,
    biases: *mut crate::types::Gna2Tensor,
    activation: *mut crate::types::Gna2Tensor,
) -> Gna2Status;

pub type FnGna2StatusGetMessage =
    unsafe extern "C" fn(status: Gna2Status, buffer: *mut c_char, buffer_size: u32) -> Gna2Status;
pub type FnGna2StatusGetMaxMessageLength = unsafe extern "C" fn() -> u32;

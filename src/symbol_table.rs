use std::ffi::c_char;
use std::ffi::c_void;

use crate::error::{GnaError, Result};
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

/// Dynamically loaded symbol table from the GNA DLL.
pub struct GnaSymbolTable {
    pub device_get_count: FnGna2DeviceGetCount,
    pub device_get_version: FnGna2DeviceGetVersion,
    pub device_open: FnGna2DeviceOpen,
    pub device_create_for_export: Option<FnGna2DeviceCreateForExport>,
    pub device_close: FnGna2DeviceClose,
    pub device_set_number_of_threads: Option<FnGna2DeviceSetNumberOfThreads>,

    pub get_library_version: Option<FnGna2GetLibraryVersion>,

    pub memory_alloc: FnGna2MemoryAlloc,
    pub memory_alloc_for_device: Option<FnGna2MemoryAllocForDevice>,
    pub memory_free: FnGna2MemoryFree,
    pub memory_set_tag: Option<FnGna2MemorySetTag>,

    pub model_create: Option<FnGna2ModelCreate>,
    pub model_release: Option<FnGna2ModelRelease>,
    pub model_get_last_error: Option<FnGna2ModelGetLastError>,
    pub model_error_get_message: Option<FnGna2ModelErrorGetMessage>,
    pub model_error_get_max_message_length: Option<FnGna2ModelErrorGetMaxMessageLength>,
    pub operation_init_fully_connected_affine: Option<FnGna2OperationInitFullyConnectedAffine>,

    pub request_config_create: Option<FnGna2RequestConfigCreate>,
    pub request_config_set_operand_buffer: Option<FnGna2RequestConfigSetOperandBuffer>,
    pub request_config_enable_active_list: Option<FnGna2RequestConfigEnableActiveList>,
    pub request_config_enable_hardware_consistency:
        Option<FnGna2RequestConfigEnableHardwareConsistency>,
    pub request_config_set_acceleration_mode: Option<FnGna2RequestConfigSetAccelerationMode>,
    pub request_config_release: Option<FnGna2RequestConfigRelease>,
    pub request_enqueue: Option<FnGna2RequestEnqueue>,
    pub request_wait: Option<FnGna2RequestWait>,

    pub instrumentation_config_create: Option<FnGna2InstrumentationConfigCreate>,
    pub instrumentation_config_assign_to_request_config:
        Option<FnGna2InstrumentationConfigAssignToRequestConfig>,
    pub instrumentation_config_set_unit: Option<FnGna2InstrumentationConfigSetUnit>,
    pub instrumentation_config_set_mode: Option<FnGna2InstrumentationConfigSetMode>,
    pub instrumentation_config_release: Option<FnGna2InstrumentationConfigRelease>,

    pub model_export_config_create: Option<FnGna2ModelExportConfigCreate>,
    pub model_export_config_release: Option<FnGna2ModelExportConfigRelease>,
    pub model_export_config_set_source: Option<FnGna2ModelExportConfigSetSource>,
    pub model_export_config_set_target: Option<FnGna2ModelExportConfigSetTarget>,
    pub model_export: Option<FnGna2ModelExport>,
    pub model_override_alignment: Option<FnGna2ModelOverrideAlignment>,

    pub status_get_message: Option<FnGna2StatusGetMessage>,
    pub status_get_max_message_length: Option<FnGna2StatusGetMaxMessageLength>,
}

impl GnaSymbolTable {
    /// Load required and optional function symbols from the loaded library.
    pub fn load(lib: &libloading::Library) -> Result<Self> {
        fn load_required<T: Copy>(
            lib: &libloading::Library,
            name: &'static [u8],
            symbol_str: &'static str,
        ) -> Result<T> {
            unsafe {
                match lib.get(name) {
                    Ok(sym) => Ok(*sym),
                    Err(err) => Err(GnaError::SymbolNotFound {
                        symbol: symbol_str,
                        source: err,
                    }),
                }
            }
        }

        fn load_optional<T: Copy>(lib: &libloading::Library, name: &[u8]) -> Option<T> {
            unsafe { lib.get(name).ok().map(|sym| *sym) }
        }

        let device_get_count: FnGna2DeviceGetCount =
            load_required(lib, b"Gna2DeviceGetCount\0", "Gna2DeviceGetCount")?;
        let device_get_version: FnGna2DeviceGetVersion =
            load_required(lib, b"Gna2DeviceGetVersion\0", "Gna2DeviceGetVersion")?;
        let device_open: FnGna2DeviceOpen =
            load_required(lib, b"Gna2DeviceOpen\0", "Gna2DeviceOpen")?;
        let device_create_for_export = load_optional(lib, b"Gna2DeviceCreateForExport\0");
        let device_close: FnGna2DeviceClose =
            load_required(lib, b"Gna2DeviceClose\0", "Gna2DeviceClose")?;

        let device_set_number_of_threads = load_optional(lib, b"Gna2DeviceSetNumberOfThreads\0");

        let get_library_version = load_optional(lib, b"Gna2GetLibraryVersion\0");

        let memory_alloc: FnGna2MemoryAlloc =
            load_required(lib, b"Gna2MemoryAlloc\0", "Gna2MemoryAlloc")?;
        let memory_alloc_for_device = load_optional(lib, b"Gna2MemoryAllocForDevice\0");
        let memory_free: FnGna2MemoryFree =
            load_required(lib, b"Gna2MemoryFree\0", "Gna2MemoryFree")?;
        let memory_set_tag = load_optional(lib, b"Gna2MemorySetTag\0");

        let model_create = load_optional(lib, b"Gna2ModelCreate\0");
        let model_release = load_optional(lib, b"Gna2ModelRelease\0");
        let model_get_last_error = load_optional(lib, b"Gna2ModelGetLastError\0");
        let model_error_get_message = load_optional(lib, b"Gna2ModelErrorGetMessage\0");
        let model_error_get_max_message_length =
            load_optional(lib, b"Gna2ModelErrorGetMaxMessageLength\0");
        let operation_init_fully_connected_affine =
            load_optional(lib, b"Gna2OperationInitFullyConnectedAffine\0");

        let request_config_create = load_optional(lib, b"Gna2RequestConfigCreate\0");
        let request_config_set_operand_buffer =
            load_optional(lib, b"Gna2RequestConfigSetOperandBuffer\0");
        let request_config_enable_active_list =
            load_optional(lib, b"Gna2RequestConfigEnableActiveList\0");
        let request_config_enable_hardware_consistency =
            load_optional(lib, b"Gna2RequestConfigEnableHardwareConsistency\0");
        let request_config_set_acceleration_mode =
            load_optional(lib, b"Gna2RequestConfigSetAccelerationMode\0");
        let request_config_release = load_optional(lib, b"Gna2RequestConfigRelease\0");
        let request_enqueue = load_optional(lib, b"Gna2RequestEnqueue\0");
        let request_wait = load_optional(lib, b"Gna2RequestWait\0");

        let instrumentation_config_create =
            load_optional(lib, b"Gna2InstrumentationConfigCreate\0");
        let instrumentation_config_assign_to_request_config =
            load_optional(lib, b"Gna2InstrumentationConfigAssignToRequestConfig\0");
        let instrumentation_config_set_unit =
            load_optional(lib, b"Gna2InstrumentationConfigSetUnit\0");
        let instrumentation_config_set_mode =
            load_optional(lib, b"Gna2InstrumentationConfigSetMode\0");
        let instrumentation_config_release =
            load_optional(lib, b"Gna2InstrumentationConfigRelease\0");

        let model_export_config_create = load_optional(lib, b"Gna2ModelExportConfigCreate\0");
        let model_export_config_release = load_optional(lib, b"Gna2ModelExportConfigRelease\0");
        let model_export_config_set_source =
            load_optional(lib, b"Gna2ModelExportConfigSetSource\0");
        let model_export_config_set_target =
            load_optional(lib, b"Gna2ModelExportConfigSetTarget\0");
        let model_export = load_optional(lib, b"Gna2ModelExport\0");
        let model_override_alignment = load_optional(lib, b"Gna2ModelOverrideAlignment\0");

        let status_get_message = load_optional(lib, b"Gna2StatusGetMessage\0");
        let status_get_max_message_length = load_optional(lib, b"Gna2StatusGetMaxMessageLength\0");

        Ok(Self {
            device_get_count,
            device_get_version,
            device_open,
            device_create_for_export,
            device_close,
            device_set_number_of_threads,
            get_library_version,
            memory_alloc,
            memory_alloc_for_device,
            memory_free,
            memory_set_tag,
            model_create,
            model_release,
            model_get_last_error,
            model_error_get_message,
            model_error_get_max_message_length,
            operation_init_fully_connected_affine,
            request_config_create,
            request_config_set_operand_buffer,
            request_config_enable_active_list,
            request_config_enable_hardware_consistency,
            request_config_set_acceleration_mode,
            request_config_release,
            request_enqueue,
            request_wait,
            instrumentation_config_create,
            instrumentation_config_assign_to_request_config,
            instrumentation_config_set_unit,
            instrumentation_config_set_mode,
            instrumentation_config_release,
            model_export_config_create,
            model_export_config_release,
            model_export_config_set_source,
            model_export_config_set_target,
            model_export,
            model_override_alignment,
            status_get_message,
            status_get_max_message_length,
        })
    }
}

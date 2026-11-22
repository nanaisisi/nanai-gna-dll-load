mod accessors;
mod fn_types;
mod getters;
mod loader;

pub use fn_types::*;

/// Dynamically loaded symbol table from the GNA DLL.
pub struct GnaSymbolTable {
    device_get_count: FnGna2DeviceGetCount,
    device_get_version: FnGna2DeviceGetVersion,
    device_open: FnGna2DeviceOpen,
    device_create_for_export: Option<FnGna2DeviceCreateForExport>,
    device_close: FnGna2DeviceClose,
    device_set_number_of_threads: Option<FnGna2DeviceSetNumberOfThreads>,

    get_library_version: Option<FnGna2GetLibraryVersion>,

    memory_alloc: FnGna2MemoryAlloc,
    memory_alloc_for_device: Option<FnGna2MemoryAllocForDevice>,
    memory_free: FnGna2MemoryFree,
    memory_set_tag: Option<FnGna2MemorySetTag>,

    model_create: Option<FnGna2ModelCreate>,
    model_release: Option<FnGna2ModelRelease>,
    model_get_last_error: Option<FnGna2ModelGetLastError>,
    model_error_get_message: Option<FnGna2ModelErrorGetMessage>,
    model_error_get_max_message_length: Option<FnGna2ModelErrorGetMaxMessageLength>,
    operation_init_fully_connected_affine: Option<FnGna2OperationInitFullyConnectedAffine>,

    request_config_create: Option<FnGna2RequestConfigCreate>,
    request_config_set_operand_buffer: Option<FnGna2RequestConfigSetOperandBuffer>,
    request_config_enable_active_list: Option<FnGna2RequestConfigEnableActiveList>,
    request_config_enable_hardware_consistency:
        Option<FnGna2RequestConfigEnableHardwareConsistency>,
    request_config_set_acceleration_mode: Option<FnGna2RequestConfigSetAccelerationMode>,
    request_config_release: Option<FnGna2RequestConfigRelease>,
    request_enqueue: Option<FnGna2RequestEnqueue>,
    request_wait: Option<FnGna2RequestWait>,

    instrumentation_config_create: Option<FnGna2InstrumentationConfigCreate>,
    instrumentation_config_assign_to_request_config:
        Option<FnGna2InstrumentationConfigAssignToRequestConfig>,
    instrumentation_config_set_unit: Option<FnGna2InstrumentationConfigSetUnit>,
    instrumentation_config_set_mode: Option<FnGna2InstrumentationConfigSetMode>,
    instrumentation_config_release: Option<FnGna2InstrumentationConfigRelease>,

    model_export_config_create: Option<FnGna2ModelExportConfigCreate>,
    model_export_config_release: Option<FnGna2ModelExportConfigRelease>,
    model_export_config_set_source: Option<FnGna2ModelExportConfigSetSource>,
    model_export_config_set_target: Option<FnGna2ModelExportConfigSetTarget>,
    model_export: Option<FnGna2ModelExport>,
    model_override_alignment: Option<FnGna2ModelOverrideAlignment>,

    status_get_message: Option<FnGna2StatusGetMessage>,
    status_get_max_message_length: Option<FnGna2StatusGetMaxMessageLength>,
}

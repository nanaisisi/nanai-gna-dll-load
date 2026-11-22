mod fn_types;
mod loader;

pub use fn_types::*;

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

mod accessors;
mod fn_types;
mod getters;
mod loader;

pub use fn_types::*;

/// Dynamically loaded symbol table from the GNA DLL.
pub struct GnaSymbolTable {
    pub(crate) device_get_count: FnGna2DeviceGetCount,
    pub(crate) device_get_version: FnGna2DeviceGetVersion,
    pub(crate) device_open: FnGna2DeviceOpen,
    pub(crate) device_create_for_export: Option<FnGna2DeviceCreateForExport>,
    pub(crate) device_close: FnGna2DeviceClose,
    pub(crate) device_set_number_of_threads: Option<FnGna2DeviceSetNumberOfThreads>,

    pub(crate) get_library_version: Option<FnGna2GetLibraryVersion>,

    pub(crate) memory_alloc: FnGna2MemoryAlloc,
    pub(crate) memory_alloc_for_device: Option<FnGna2MemoryAllocForDevice>,
    pub(crate) memory_free: FnGna2MemoryFree,
    pub(crate) memory_set_tag: Option<FnGna2MemorySetTag>,

    pub(crate) model_create: Option<FnGna2ModelCreate>,
    pub(crate) model_release: Option<FnGna2ModelRelease>,
    pub(crate) model_get_last_error: Option<FnGna2ModelGetLastError>,
    pub(crate) model_error_get_message: Option<FnGna2ModelErrorGetMessage>,
    pub(crate) model_error_get_max_message_length: Option<FnGna2ModelErrorGetMaxMessageLength>,
    pub(crate) operation_init_fully_connected_affine:
        Option<FnGna2OperationInitFullyConnectedAffine>,

    pub(crate) request_config_create: Option<FnGna2RequestConfigCreate>,
    pub(crate) request_config_set_operand_buffer: Option<FnGna2RequestConfigSetOperandBuffer>,
    pub(crate) request_config_enable_active_list: Option<FnGna2RequestConfigEnableActiveList>,
    pub(crate) request_config_enable_hardware_consistency:
        Option<FnGna2RequestConfigEnableHardwareConsistency>,
    pub(crate) request_config_set_acceleration_mode: Option<FnGna2RequestConfigSetAccelerationMode>,
    pub(crate) request_config_release: Option<FnGna2RequestConfigRelease>,
    pub(crate) request_enqueue: Option<FnGna2RequestEnqueue>,
    pub(crate) request_wait: Option<FnGna2RequestWait>,

    pub(crate) instrumentation_config_create: Option<FnGna2InstrumentationConfigCreate>,
    pub(crate) instrumentation_config_assign_to_request_config:
        Option<FnGna2InstrumentationConfigAssignToRequestConfig>,
    pub(crate) instrumentation_config_set_unit: Option<FnGna2InstrumentationConfigSetUnit>,
    pub(crate) instrumentation_config_set_mode: Option<FnGna2InstrumentationConfigSetMode>,
    pub(crate) instrumentation_config_release: Option<FnGna2InstrumentationConfigRelease>,

    pub(crate) model_export_config_create: Option<FnGna2ModelExportConfigCreate>,
    pub(crate) model_export_config_release: Option<FnGna2ModelExportConfigRelease>,
    pub(crate) model_export_config_set_source: Option<FnGna2ModelExportConfigSetSource>,
    pub(crate) model_export_config_set_target: Option<FnGna2ModelExportConfigSetTarget>,
    pub(crate) model_export: Option<FnGna2ModelExport>,
    pub(crate) model_override_alignment: Option<FnGna2ModelOverrideAlignment>,

    pub(crate) status_get_message: Option<FnGna2StatusGetMessage>,
    pub(crate) status_get_max_message_length: Option<FnGna2StatusGetMaxMessageLength>,
}

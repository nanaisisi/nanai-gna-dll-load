use super::GnaSymbolTable;
use super::fn_types::*;

macro_rules! impl_optional_setters {
    ($(($method:ident, $field:ident, $ty:ty)),* $(,)?) => {
        impl GnaSymbolTable {
            $(
                pub fn $method(mut self, f: $ty) -> Self {
                    self.$field = Some(f);
                    self
                }
            )*
        }
    };
}

impl GnaSymbolTable {
    /// Creates a symbol table from the 6 required GNA symbols, leaving all optional symbols `None`.
    pub fn from_required(
        device_get_count: FnGna2DeviceGetCount,
        device_get_version: FnGna2DeviceGetVersion,
        device_open: FnGna2DeviceOpen,
        device_close: FnGna2DeviceClose,
        memory_alloc: FnGna2MemoryAlloc,
        memory_free: FnGna2MemoryFree,
    ) -> Self {
        Self {
            device_get_count,
            device_get_version,
            device_open,
            device_create_for_export: None,
            device_close,
            device_set_number_of_threads: None,
            get_library_version: None,
            memory_alloc,
            memory_alloc_for_device: None,
            memory_free,
            memory_set_tag: None,
            model_create: None,
            model_release: None,
            model_get_last_error: None,
            model_error_get_message: None,
            model_error_get_max_message_length: None,
            operation_init_fully_connected_affine: None,
            request_config_create: None,
            request_config_set_operand_buffer: None,
            request_config_enable_active_list: None,
            request_config_enable_hardware_consistency: None,
            request_config_set_acceleration_mode: None,
            request_config_release: None,
            request_enqueue: None,
            request_wait: None,
            instrumentation_config_create: None,
            instrumentation_config_assign_to_request_config: None,
            instrumentation_config_set_unit: None,
            instrumentation_config_set_mode: None,
            instrumentation_config_release: None,
            model_export_config_create: None,
            model_export_config_release: None,
            model_export_config_set_source: None,
            model_export_config_set_target: None,
            model_export: None,
            model_override_alignment: None,
            status_get_message: None,
            status_get_max_message_length: None,
        }
    }
}

impl_optional_setters!(
    (
        with_device_create_for_export,
        device_create_for_export,
        FnGna2DeviceCreateForExport
    ),
    (
        with_device_set_number_of_threads,
        device_set_number_of_threads,
        FnGna2DeviceSetNumberOfThreads
    ),
    (
        with_get_library_version,
        get_library_version,
        FnGna2GetLibraryVersion
    ),
    (
        with_memory_alloc_for_device,
        memory_alloc_for_device,
        FnGna2MemoryAllocForDevice
    ),
    (with_memory_set_tag, memory_set_tag, FnGna2MemorySetTag),
    (with_model_create, model_create, FnGna2ModelCreate),
    (with_model_release, model_release, FnGna2ModelRelease),
    (
        with_model_get_last_error,
        model_get_last_error,
        FnGna2ModelGetLastError
    ),
    (
        with_model_error_get_message,
        model_error_get_message,
        FnGna2ModelErrorGetMessage
    ),
    (
        with_model_error_get_max_message_length,
        model_error_get_max_message_length,
        FnGna2ModelErrorGetMaxMessageLength
    ),
    (
        with_operation_init_fully_connected_affine,
        operation_init_fully_connected_affine,
        FnGna2OperationInitFullyConnectedAffine
    ),
    (
        with_request_config_create,
        request_config_create,
        FnGna2RequestConfigCreate
    ),
    (
        with_request_config_set_operand_buffer,
        request_config_set_operand_buffer,
        FnGna2RequestConfigSetOperandBuffer
    ),
    (
        with_request_config_enable_active_list,
        request_config_enable_active_list,
        FnGna2RequestConfigEnableActiveList
    ),
    (
        with_request_config_enable_hardware_consistency,
        request_config_enable_hardware_consistency,
        FnGna2RequestConfigEnableHardwareConsistency
    ),
    (
        with_request_config_set_acceleration_mode,
        request_config_set_acceleration_mode,
        FnGna2RequestConfigSetAccelerationMode
    ),
    (
        with_request_config_release,
        request_config_release,
        FnGna2RequestConfigRelease
    ),
    (with_request_enqueue, request_enqueue, FnGna2RequestEnqueue),
    (with_request_wait, request_wait, FnGna2RequestWait),
    (
        with_instrumentation_config_create,
        instrumentation_config_create,
        FnGna2InstrumentationConfigCreate
    ),
    (
        with_instrumentation_config_assign_to_request_config,
        instrumentation_config_assign_to_request_config,
        FnGna2InstrumentationConfigAssignToRequestConfig
    ),
    (
        with_instrumentation_config_set_unit,
        instrumentation_config_set_unit,
        FnGna2InstrumentationConfigSetUnit
    ),
    (
        with_instrumentation_config_set_mode,
        instrumentation_config_set_mode,
        FnGna2InstrumentationConfigSetMode
    ),
    (
        with_instrumentation_config_release,
        instrumentation_config_release,
        FnGna2InstrumentationConfigRelease
    ),
    (
        with_model_export_config_create,
        model_export_config_create,
        FnGna2ModelExportConfigCreate
    ),
    (
        with_model_export_config_release,
        model_export_config_release,
        FnGna2ModelExportConfigRelease
    ),
    (
        with_model_export_config_set_source,
        model_export_config_set_source,
        FnGna2ModelExportConfigSetSource
    ),
    (
        with_model_export_config_set_target,
        model_export_config_set_target,
        FnGna2ModelExportConfigSetTarget
    ),
    (with_model_export, model_export, FnGna2ModelExport),
    (
        with_model_override_alignment,
        model_override_alignment,
        FnGna2ModelOverrideAlignment
    ),
    (
        with_status_get_message,
        status_get_message,
        FnGna2StatusGetMessage
    ),
    (
        with_status_get_max_message_length,
        status_get_max_message_length,
        FnGna2StatusGetMaxMessageLength
    ),
);

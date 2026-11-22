use super::GnaSymbolTable;
use super::fn_types::*;

macro_rules! impl_required_getters {
    ($(($field:ident, $ty:ty)),* $(,)?) => {
        impl GnaSymbolTable {
            $(
                pub fn $field(&self) -> $ty {
                    self.$field
                }
            )*
        }
    };
}

macro_rules! impl_optional_getters {
    ($(($field:ident, $ty:ty)),* $(,)?) => {
        impl GnaSymbolTable {
            $(
                pub fn $field(&self) -> Option<$ty> {
                    self.$field
                }
            )*
        }
    };
}

macro_rules! impl_optional_clearers {
    ($(($method:ident, $field:ident)),* $(,)?) => {
        impl GnaSymbolTable {
            $(
                pub fn $method(&mut self) {
                    self.$field = None;
                }
            )*
        }
    };
}

impl_required_getters!(
    (device_get_count, FnGna2DeviceGetCount),
    (device_get_version, FnGna2DeviceGetVersion),
    (device_open, FnGna2DeviceOpen),
    (device_close, FnGna2DeviceClose),
    (memory_alloc, FnGna2MemoryAlloc),
    (memory_free, FnGna2MemoryFree),
);

impl_optional_getters!(
    (device_create_for_export, FnGna2DeviceCreateForExport),
    (device_set_number_of_threads, FnGna2DeviceSetNumberOfThreads),
    (get_library_version, FnGna2GetLibraryVersion),
    (memory_alloc_for_device, FnGna2MemoryAllocForDevice),
    (memory_set_tag, FnGna2MemorySetTag),
    (model_create, FnGna2ModelCreate),
    (model_release, FnGna2ModelRelease),
    (model_get_last_error, FnGna2ModelGetLastError),
    (model_error_get_message, FnGna2ModelErrorGetMessage),
    (
        model_error_get_max_message_length,
        FnGna2ModelErrorGetMaxMessageLength
    ),
    (
        operation_init_fully_connected_affine,
        FnGna2OperationInitFullyConnectedAffine
    ),
    (request_config_create, FnGna2RequestConfigCreate),
    (
        request_config_set_operand_buffer,
        FnGna2RequestConfigSetOperandBuffer
    ),
    (
        request_config_enable_active_list,
        FnGna2RequestConfigEnableActiveList
    ),
    (
        request_config_enable_hardware_consistency,
        FnGna2RequestConfigEnableHardwareConsistency
    ),
    (
        request_config_set_acceleration_mode,
        FnGna2RequestConfigSetAccelerationMode
    ),
    (request_config_release, FnGna2RequestConfigRelease),
    (request_enqueue, FnGna2RequestEnqueue),
    (request_wait, FnGna2RequestWait),
    (
        instrumentation_config_create,
        FnGna2InstrumentationConfigCreate
    ),
    (
        instrumentation_config_assign_to_request_config,
        FnGna2InstrumentationConfigAssignToRequestConfig
    ),
    (
        instrumentation_config_set_unit,
        FnGna2InstrumentationConfigSetUnit
    ),
    (
        instrumentation_config_set_mode,
        FnGna2InstrumentationConfigSetMode
    ),
    (
        instrumentation_config_release,
        FnGna2InstrumentationConfigRelease
    ),
    (model_export_config_create, FnGna2ModelExportConfigCreate),
    (model_export_config_release, FnGna2ModelExportConfigRelease),
    (
        model_export_config_set_source,
        FnGna2ModelExportConfigSetSource
    ),
    (
        model_export_config_set_target,
        FnGna2ModelExportConfigSetTarget
    ),
    (model_export, FnGna2ModelExport),
    (model_override_alignment, FnGna2ModelOverrideAlignment),
    (status_get_message, FnGna2StatusGetMessage),
    (
        status_get_max_message_length,
        FnGna2StatusGetMaxMessageLength
    ),
);

impl_optional_clearers!(
    (clear_get_library_version, get_library_version),
    (clear_model_create, model_create),
    (clear_model_release, model_release),
    (clear_model_get_last_error, model_get_last_error),
    (clear_model_error_get_message, model_error_get_message),
    (
        clear_model_error_get_max_message_length,
        model_error_get_max_message_length
    ),
    (
        clear_operation_init_fully_connected_affine,
        operation_init_fully_connected_affine
    ),
    (
        clear_instrumentation_config_create,
        instrumentation_config_create
    ),
    (
        clear_instrumentation_config_assign_to_request_config,
        instrumentation_config_assign_to_request_config
    ),
    (
        clear_instrumentation_config_set_unit,
        instrumentation_config_set_unit
    ),
    (clear_model_export_config_create, model_export_config_create),
    (
        clear_model_export_config_release,
        model_export_config_release
    ),
    (
        clear_model_export_config_set_source,
        model_export_config_set_source
    ),
    (
        clear_model_export_config_set_target,
        model_export_config_set_target
    ),
    (clear_model_export, model_export),
    (clear_model_override_alignment, model_override_alignment),
    (clear_status_get_message, status_get_message),
);

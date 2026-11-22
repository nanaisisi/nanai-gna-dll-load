use crate::error::{GnaError, Result};

use super::{
    FnGna2DeviceClose, FnGna2DeviceGetCount, FnGna2DeviceGetVersion, FnGna2DeviceOpen,
    FnGna2MemoryAlloc, FnGna2MemoryFree, GnaSymbolTable,
};

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

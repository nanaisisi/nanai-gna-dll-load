use crate::error::{GnaError, Result};

use super::GnaSymbolTable;

impl GnaSymbolTable {
    /// Load required and optional function symbols from the loaded library.
    pub fn load(lib: &libloading::Library) -> Result<Self> {
        fn load_required<T: Copy>(
            lib: &libloading::Library,
            name_with_nul: &'static [u8],
            symbol_str: &'static str,
        ) -> Result<T> {
            unsafe {
                match lib.get(name_with_nul) {
                    Ok(sym) => Ok(*sym),
                    Err(err) => Err(GnaError::SymbolNotFound {
                        symbol: symbol_str,
                        source: err,
                    }),
                }
            }
        }

        fn load_optional<T: Copy>(lib: &libloading::Library, name_with_nul: &[u8]) -> Option<T> {
            unsafe { lib.get(name_with_nul).ok().map(|sym| *sym) }
        }

        macro_rules! req {
            ($name:literal) => {
                load_required(lib, concat!($name, "\0").as_bytes(), $name)?
            };
        }

        macro_rules! opt {
            ($name:literal) => {
                load_optional(lib, concat!($name, "\0").as_bytes())
            };
        }

        Ok(Self {
            device_get_count: req!("Gna2DeviceGetCount"),
            device_get_version: req!("Gna2DeviceGetVersion"),
            device_open: req!("Gna2DeviceOpen"),
            device_create_for_export: opt!("Gna2DeviceCreateForExport"),
            device_close: req!("Gna2DeviceClose"),
            device_set_number_of_threads: opt!("Gna2DeviceSetNumberOfThreads"),
            get_library_version: opt!("Gna2GetLibraryVersion"),
            memory_alloc: req!("Gna2MemoryAlloc"),
            memory_alloc_for_device: opt!("Gna2MemoryAllocForDevice"),
            memory_free: req!("Gna2MemoryFree"),
            memory_set_tag: opt!("Gna2MemorySetTag"),
            model_create: opt!("Gna2ModelCreate"),
            model_release: opt!("Gna2ModelRelease"),
            model_get_last_error: opt!("Gna2ModelGetLastError"),
            model_error_get_message: opt!("Gna2ModelErrorGetMessage"),
            model_error_get_max_message_length: opt!("Gna2ModelErrorGetMaxMessageLength"),
            operation_init_fully_connected_affine: opt!("Gna2OperationInitFullyConnectedAffine"),
            request_config_create: opt!("Gna2RequestConfigCreate"),
            request_config_set_operand_buffer: opt!("Gna2RequestConfigSetOperandBuffer"),
            request_config_enable_active_list: opt!("Gna2RequestConfigEnableActiveList"),
            request_config_enable_hardware_consistency: opt!(
                "Gna2RequestConfigEnableHardwareConsistency"
            ),
            request_config_set_acceleration_mode: opt!("Gna2RequestConfigSetAccelerationMode"),
            request_config_release: opt!("Gna2RequestConfigRelease"),
            request_enqueue: opt!("Gna2RequestEnqueue"),
            request_wait: opt!("Gna2RequestWait"),
            instrumentation_config_create: opt!("Gna2InstrumentationConfigCreate"),
            instrumentation_config_assign_to_request_config: opt!(
                "Gna2InstrumentationConfigAssignToRequestConfig"
            ),
            instrumentation_config_set_unit: opt!("Gna2InstrumentationConfigSetUnit"),
            instrumentation_config_set_mode: opt!("Gna2InstrumentationConfigSetMode"),
            instrumentation_config_release: opt!("Gna2InstrumentationConfigRelease"),
            model_export_config_create: opt!("Gna2ModelExportConfigCreate"),
            model_export_config_release: opt!("Gna2ModelExportConfigRelease"),
            model_export_config_set_source: opt!("Gna2ModelExportConfigSetSource"),
            model_export_config_set_target: opt!("Gna2ModelExportConfigSetTarget"),
            model_export: opt!("Gna2ModelExport"),
            model_override_alignment: opt!("Gna2ModelOverrideAlignment"),
            status_get_message: opt!("Gna2StatusGetMessage"),
            status_get_max_message_length: opt!("Gna2StatusGetMaxMessageLength"),
        })
    }
}

use super::GnaSymbolTable;
use super::fn_types::*;

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

    pub fn with_device_create_for_export(mut self, f: FnGna2DeviceCreateForExport) -> Self {
        self.device_create_for_export = Some(f);
        self
    }

    pub fn with_device_set_number_of_threads(mut self, f: FnGna2DeviceSetNumberOfThreads) -> Self {
        self.device_set_number_of_threads = Some(f);
        self
    }

    pub fn with_get_library_version(mut self, f: FnGna2GetLibraryVersion) -> Self {
        self.get_library_version = Some(f);
        self
    }

    pub fn with_memory_alloc_for_device(mut self, f: FnGna2MemoryAllocForDevice) -> Self {
        self.memory_alloc_for_device = Some(f);
        self
    }

    pub fn with_memory_set_tag(mut self, f: FnGna2MemorySetTag) -> Self {
        self.memory_set_tag = Some(f);
        self
    }

    pub fn with_model_create(mut self, f: FnGna2ModelCreate) -> Self {
        self.model_create = Some(f);
        self
    }

    pub fn with_model_release(mut self, f: FnGna2ModelRelease) -> Self {
        self.model_release = Some(f);
        self
    }

    pub fn with_model_get_last_error(mut self, f: FnGna2ModelGetLastError) -> Self {
        self.model_get_last_error = Some(f);
        self
    }

    pub fn with_model_error_get_message(mut self, f: FnGna2ModelErrorGetMessage) -> Self {
        self.model_error_get_message = Some(f);
        self
    }

    pub fn with_model_error_get_max_message_length(
        mut self,
        f: FnGna2ModelErrorGetMaxMessageLength,
    ) -> Self {
        self.model_error_get_max_message_length = Some(f);
        self
    }

    pub fn with_operation_init_fully_connected_affine(
        mut self,
        f: FnGna2OperationInitFullyConnectedAffine,
    ) -> Self {
        self.operation_init_fully_connected_affine = Some(f);
        self
    }

    pub fn with_request_config_create(mut self, f: FnGna2RequestConfigCreate) -> Self {
        self.request_config_create = Some(f);
        self
    }

    pub fn with_request_config_set_operand_buffer(
        mut self,
        f: FnGna2RequestConfigSetOperandBuffer,
    ) -> Self {
        self.request_config_set_operand_buffer = Some(f);
        self
    }

    pub fn with_request_config_enable_active_list(
        mut self,
        f: FnGna2RequestConfigEnableActiveList,
    ) -> Self {
        self.request_config_enable_active_list = Some(f);
        self
    }

    pub fn with_request_config_enable_hardware_consistency(
        mut self,
        f: FnGna2RequestConfigEnableHardwareConsistency,
    ) -> Self {
        self.request_config_enable_hardware_consistency = Some(f);
        self
    }

    pub fn with_request_config_set_acceleration_mode(
        mut self,
        f: FnGna2RequestConfigSetAccelerationMode,
    ) -> Self {
        self.request_config_set_acceleration_mode = Some(f);
        self
    }

    pub fn with_request_config_release(mut self, f: FnGna2RequestConfigRelease) -> Self {
        self.request_config_release = Some(f);
        self
    }

    pub fn with_request_enqueue(mut self, f: FnGna2RequestEnqueue) -> Self {
        self.request_enqueue = Some(f);
        self
    }

    pub fn with_request_wait(mut self, f: FnGna2RequestWait) -> Self {
        self.request_wait = Some(f);
        self
    }

    pub fn with_instrumentation_config_create(
        mut self,
        f: FnGna2InstrumentationConfigCreate,
    ) -> Self {
        self.instrumentation_config_create = Some(f);
        self
    }

    pub fn with_instrumentation_config_assign_to_request_config(
        mut self,
        f: FnGna2InstrumentationConfigAssignToRequestConfig,
    ) -> Self {
        self.instrumentation_config_assign_to_request_config = Some(f);
        self
    }

    pub fn with_instrumentation_config_set_unit(
        mut self,
        f: FnGna2InstrumentationConfigSetUnit,
    ) -> Self {
        self.instrumentation_config_set_unit = Some(f);
        self
    }

    pub fn with_instrumentation_config_set_mode(
        mut self,
        f: FnGna2InstrumentationConfigSetMode,
    ) -> Self {
        self.instrumentation_config_set_mode = Some(f);
        self
    }

    pub fn with_instrumentation_config_release(
        mut self,
        f: FnGna2InstrumentationConfigRelease,
    ) -> Self {
        self.instrumentation_config_release = Some(f);
        self
    }

    pub fn with_model_export_config_create(mut self, f: FnGna2ModelExportConfigCreate) -> Self {
        self.model_export_config_create = Some(f);
        self
    }

    pub fn with_model_export_config_release(mut self, f: FnGna2ModelExportConfigRelease) -> Self {
        self.model_export_config_release = Some(f);
        self
    }

    pub fn with_model_export_config_set_source(
        mut self,
        f: FnGna2ModelExportConfigSetSource,
    ) -> Self {
        self.model_export_config_set_source = Some(f);
        self
    }

    pub fn with_model_export_config_set_target(
        mut self,
        f: FnGna2ModelExportConfigSetTarget,
    ) -> Self {
        self.model_export_config_set_target = Some(f);
        self
    }

    pub fn with_model_export(mut self, f: FnGna2ModelExport) -> Self {
        self.model_export = Some(f);
        self
    }

    pub fn with_model_override_alignment(mut self, f: FnGna2ModelOverrideAlignment) -> Self {
        self.model_override_alignment = Some(f);
        self
    }

    pub fn with_status_get_message(mut self, f: FnGna2StatusGetMessage) -> Self {
        self.status_get_message = Some(f);
        self
    }

    pub fn with_status_get_max_message_length(
        mut self,
        f: FnGna2StatusGetMaxMessageLength,
    ) -> Self {
        self.status_get_max_message_length = Some(f);
        self
    }
}

use super::GnaSymbolTable;
use super::fn_types::*;

impl GnaSymbolTable {
    pub fn device_get_count(&self) -> FnGna2DeviceGetCount {
        self.device_get_count
    }

    pub fn device_get_version(&self) -> FnGna2DeviceGetVersion {
        self.device_get_version
    }

    pub fn device_open(&self) -> FnGna2DeviceOpen {
        self.device_open
    }

    pub fn device_create_for_export(&self) -> Option<FnGna2DeviceCreateForExport> {
        self.device_create_for_export
    }

    pub fn device_close(&self) -> FnGna2DeviceClose {
        self.device_close
    }

    pub fn device_set_number_of_threads(&self) -> Option<FnGna2DeviceSetNumberOfThreads> {
        self.device_set_number_of_threads
    }

    pub fn get_library_version(&self) -> Option<FnGna2GetLibraryVersion> {
        self.get_library_version
    }

    pub fn memory_alloc(&self) -> FnGna2MemoryAlloc {
        self.memory_alloc
    }

    pub fn memory_alloc_for_device(&self) -> Option<FnGna2MemoryAllocForDevice> {
        self.memory_alloc_for_device
    }

    pub fn memory_free(&self) -> FnGna2MemoryFree {
        self.memory_free
    }

    pub fn memory_set_tag(&self) -> Option<FnGna2MemorySetTag> {
        self.memory_set_tag
    }

    pub fn model_create(&self) -> Option<FnGna2ModelCreate> {
        self.model_create
    }

    pub fn model_release(&self) -> Option<FnGna2ModelRelease> {
        self.model_release
    }

    pub fn model_get_last_error(&self) -> Option<FnGna2ModelGetLastError> {
        self.model_get_last_error
    }

    pub fn model_error_get_message(&self) -> Option<FnGna2ModelErrorGetMessage> {
        self.model_error_get_message
    }

    pub fn model_error_get_max_message_length(
        &self,
    ) -> Option<FnGna2ModelErrorGetMaxMessageLength> {
        self.model_error_get_max_message_length
    }

    pub fn operation_init_fully_connected_affine(
        &self,
    ) -> Option<FnGna2OperationInitFullyConnectedAffine> {
        self.operation_init_fully_connected_affine
    }

    pub fn request_config_create(&self) -> Option<FnGna2RequestConfigCreate> {
        self.request_config_create
    }

    pub fn request_config_set_operand_buffer(&self) -> Option<FnGna2RequestConfigSetOperandBuffer> {
        self.request_config_set_operand_buffer
    }

    pub fn request_config_enable_active_list(&self) -> Option<FnGna2RequestConfigEnableActiveList> {
        self.request_config_enable_active_list
    }

    pub fn request_config_enable_hardware_consistency(
        &self,
    ) -> Option<FnGna2RequestConfigEnableHardwareConsistency> {
        self.request_config_enable_hardware_consistency
    }

    pub fn request_config_set_acceleration_mode(
        &self,
    ) -> Option<FnGna2RequestConfigSetAccelerationMode> {
        self.request_config_set_acceleration_mode
    }

    pub fn request_config_release(&self) -> Option<FnGna2RequestConfigRelease> {
        self.request_config_release
    }

    pub fn request_enqueue(&self) -> Option<FnGna2RequestEnqueue> {
        self.request_enqueue
    }

    pub fn request_wait(&self) -> Option<FnGna2RequestWait> {
        self.request_wait
    }

    pub fn instrumentation_config_create(&self) -> Option<FnGna2InstrumentationConfigCreate> {
        self.instrumentation_config_create
    }

    pub fn instrumentation_config_assign_to_request_config(
        &self,
    ) -> Option<FnGna2InstrumentationConfigAssignToRequestConfig> {
        self.instrumentation_config_assign_to_request_config
    }

    pub fn instrumentation_config_set_unit(&self) -> Option<FnGna2InstrumentationConfigSetUnit> {
        self.instrumentation_config_set_unit
    }

    pub fn instrumentation_config_set_mode(&self) -> Option<FnGna2InstrumentationConfigSetMode> {
        self.instrumentation_config_set_mode
    }

    pub fn instrumentation_config_release(&self) -> Option<FnGna2InstrumentationConfigRelease> {
        self.instrumentation_config_release
    }

    pub fn model_export_config_create(&self) -> Option<FnGna2ModelExportConfigCreate> {
        self.model_export_config_create
    }

    pub fn model_export_config_release(&self) -> Option<FnGna2ModelExportConfigRelease> {
        self.model_export_config_release
    }

    pub fn model_export_config_set_source(&self) -> Option<FnGna2ModelExportConfigSetSource> {
        self.model_export_config_set_source
    }

    pub fn model_export_config_set_target(&self) -> Option<FnGna2ModelExportConfigSetTarget> {
        self.model_export_config_set_target
    }

    pub fn model_export(&self) -> Option<FnGna2ModelExport> {
        self.model_export
    }

    pub fn model_override_alignment(&self) -> Option<FnGna2ModelOverrideAlignment> {
        self.model_override_alignment
    }

    pub fn status_get_message(&self) -> Option<FnGna2StatusGetMessage> {
        self.status_get_message
    }

    pub fn status_get_max_message_length(&self) -> Option<FnGna2StatusGetMaxMessageLength> {
        self.status_get_max_message_length
    }
}

impl GnaSymbolTable {
    pub fn clear_get_library_version(&mut self) {
        self.get_library_version = None;
    }

    pub fn clear_model_create(&mut self) {
        self.model_create = None;
    }

    pub fn clear_model_release(&mut self) {
        self.model_release = None;
    }

    pub fn clear_model_get_last_error(&mut self) {
        self.model_get_last_error = None;
    }

    pub fn clear_model_error_get_message(&mut self) {
        self.model_error_get_message = None;
    }

    pub fn clear_model_error_get_max_message_length(&mut self) {
        self.model_error_get_max_message_length = None;
    }

    pub fn clear_operation_init_fully_connected_affine(&mut self) {
        self.operation_init_fully_connected_affine = None;
    }

    pub fn clear_instrumentation_config_create(&mut self) {
        self.instrumentation_config_create = None;
    }

    pub fn clear_instrumentation_config_assign_to_request_config(&mut self) {
        self.instrumentation_config_assign_to_request_config = None;
    }

    pub fn clear_instrumentation_config_set_unit(&mut self) {
        self.instrumentation_config_set_unit = None;
    }

    pub fn clear_model_export_config_create(&mut self) {
        self.model_export_config_create = None;
    }

    pub fn clear_model_export_config_release(&mut self) {
        self.model_export_config_release = None;
    }

    pub fn clear_model_export_config_set_source(&mut self) {
        self.model_export_config_set_source = None;
    }

    pub fn clear_model_export_config_set_target(&mut self) {
        self.model_export_config_set_target = None;
    }

    pub fn clear_model_export(&mut self) {
        self.model_export = None;
    }

    pub fn clear_model_override_alignment(&mut self) {
        self.model_override_alignment = None;
    }

    pub fn clear_status_get_message(&mut self) {
        self.status_get_message = None;
    }
}

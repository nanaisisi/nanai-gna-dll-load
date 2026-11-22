use tempfile::tempdir;

use nanai_gna_dll_load::{loader::GnaLibrary, symbol_table::GnaSymbolTable};

use super::all_dll_src::*;
use super::helpers::*;

#[test]
fn test_symbol_table_builder_and_getters_coverage() {
    let dir = tempdir().unwrap();
    let default_name = GnaLibrary::default_dll_name();
    let dll_path = compile_mock_dll(
        dir.path(),
        "all_syms_cov",
        default_name,
        ALL_SYMBOLS_DLL_SRC,
    );
    let raw_lib = unsafe { libloading::Library::new(&dll_path) }.unwrap();
    let loaded = GnaSymbolTable::load(&raw_lib).unwrap();

    let mut built = GnaSymbolTable::from_required(
        loaded.device_get_count(),
        loaded.device_get_version(),
        loaded.device_open(),
        loaded.device_close(),
        loaded.memory_alloc(),
        loaded.memory_free(),
    )
    .with_device_create_for_export(loaded.device_create_for_export().unwrap())
    .with_device_set_number_of_threads(loaded.device_set_number_of_threads().unwrap())
    .with_get_library_version(loaded.get_library_version().unwrap())
    .with_memory_alloc_for_device(loaded.memory_alloc_for_device().unwrap())
    .with_memory_set_tag(loaded.memory_set_tag().unwrap())
    .with_model_create(loaded.model_create().unwrap())
    .with_model_release(loaded.model_release().unwrap())
    .with_model_get_last_error(loaded.model_get_last_error().unwrap())
    .with_model_error_get_message(loaded.model_error_get_message().unwrap())
    .with_model_error_get_max_message_length(loaded.model_error_get_max_message_length().unwrap())
    .with_operation_init_fully_connected_affine(
        loaded.operation_init_fully_connected_affine().unwrap(),
    )
    .with_request_config_create(loaded.request_config_create().unwrap())
    .with_request_config_set_operand_buffer(loaded.request_config_set_operand_buffer().unwrap())
    .with_request_config_enable_active_list(loaded.request_config_enable_active_list().unwrap())
    .with_request_config_enable_hardware_consistency(
        loaded.request_config_enable_hardware_consistency().unwrap(),
    )
    .with_request_config_set_acceleration_mode(
        loaded.request_config_set_acceleration_mode().unwrap(),
    )
    .with_request_config_release(loaded.request_config_release().unwrap())
    .with_request_enqueue(loaded.request_enqueue().unwrap())
    .with_request_wait(loaded.request_wait().unwrap())
    .with_instrumentation_config_create(loaded.instrumentation_config_create().unwrap())
    .with_instrumentation_config_assign_to_request_config(
        loaded
            .instrumentation_config_assign_to_request_config()
            .unwrap(),
    )
    .with_instrumentation_config_set_unit(loaded.instrumentation_config_set_unit().unwrap())
    .with_instrumentation_config_set_mode(loaded.instrumentation_config_set_mode().unwrap())
    .with_instrumentation_config_release(loaded.instrumentation_config_release().unwrap())
    .with_model_export_config_create(loaded.model_export_config_create().unwrap())
    .with_model_export_config_release(loaded.model_export_config_release().unwrap())
    .with_model_export_config_set_source(loaded.model_export_config_set_source().unwrap())
    .with_model_export_config_set_target(loaded.model_export_config_set_target().unwrap())
    .with_model_export(loaded.model_export().unwrap())
    .with_model_override_alignment(loaded.model_override_alignment().unwrap())
    .with_status_get_message(loaded.status_get_message().unwrap())
    .with_status_get_max_message_length(loaded.status_get_max_message_length().unwrap());

    assert!(built.device_create_for_export().is_some());
    assert!(built.status_get_max_message_length().is_some());

    built.clear_get_library_version();
    built.clear_model_override_alignment();
    assert!(built.get_library_version().is_none());
    assert!(built.model_override_alignment().is_none());
}

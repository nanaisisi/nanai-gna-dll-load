use tempfile::tempdir;

use nanai_gna_dll_load::{
    error::GnaError,
    loader::{GnaLibrary, GnaLibraryBuilder},
    symbol_table::GnaSymbolTable,
};

use super::all_dll_src::*;
use super::guards::*;
use super::helpers::*;

#[test]
fn test_symbol_table_load_all_symbols_present_and_loader_success() {
    let dir = tempdir().unwrap();
    let default_name = GnaLibrary::default_dll_name();
    let dll_path = compile_mock_dll(dir.path(), "all_syms", default_name, ALL_SYMBOLS_DLL_SRC);

    // 1. Direct GnaSymbolTable::load verification
    {
        let raw_lib = unsafe { libloading::Library::new(&dll_path) }.unwrap();
        let table = GnaSymbolTable::load(&raw_lib).unwrap();

        assert!(table.device_create_for_export().is_some());
        assert!(table.device_set_number_of_threads().is_some());
        assert!(table.get_library_version().is_some());
        assert!(table.memory_alloc_for_device().is_some());
        assert!(table.memory_set_tag().is_some());
        assert!(table.model_create().is_some());
        assert!(table.model_release().is_some());
        assert!(table.model_get_last_error().is_some());
        assert!(table.model_error_get_message().is_some());
        assert!(table.model_error_get_max_message_length().is_some());
        assert!(table.operation_init_fully_connected_affine().is_some());
        assert!(table.request_config_create().is_some());
        assert!(table.request_config_set_operand_buffer().is_some());
        assert!(table.request_config_enable_active_list().is_some());
        assert!(table.request_config_enable_hardware_consistency().is_some());
        assert!(table.request_config_set_acceleration_mode().is_some());
        assert!(table.request_config_release().is_some());
        assert!(table.request_enqueue().is_some());
        assert!(table.request_wait().is_some());
        assert!(table.instrumentation_config_create().is_some());
        assert!(
            table
                .instrumentation_config_assign_to_request_config()
                .is_some()
        );
        assert!(table.instrumentation_config_set_unit().is_some());
        assert!(table.instrumentation_config_set_mode().is_some());
        assert!(table.instrumentation_config_release().is_some());
        assert!(table.model_export_config_create().is_some());
        assert!(table.model_export_config_release().is_some());
        assert!(table.model_export_config_set_source().is_some());
        assert!(table.model_export_config_set_target().is_some());
        assert!(table.model_export().is_some());
        assert!(table.model_override_alignment().is_some());
        assert!(table.status_get_message().is_some());
        assert!(table.status_get_max_message_length().is_some());

        let mut count = 0u32;
        let status = unsafe { (table.device_get_count())(&mut count) };
        assert_eq!(status, 0);
        assert_eq!(count, 2);
        assert_eq!(
            unsafe { (table.model_error_get_max_message_length().unwrap())() },
            512
        );
        assert_eq!(
            unsafe { (table.status_get_max_message_length().unwrap())() },
            256
        );

        drop(raw_lib);
    }

    // 2. GnaLibrary::load_from_path
    {
        let lib = GnaLibrary::load_from_path(&dll_path).unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        assert_eq!(lib.get_library_version().unwrap(), "3.5.0-mock");
        lib.override_alignment(64).unwrap();
        assert!(lib.symbols().model_create().is_some());
        drop(lib);
    }

    // 3. load_from_env, load_from_env_or_path, and GnaLibraryBuilder::load
    {
        let _lock = ENV_LOCK.lock().unwrap();
        let mut guard = EnvGuard::new();
        guard.set("TEST_SYM_TABLE_FILE", &dll_path);
        guard.set("TEST_SYM_TABLE_DIR", dir.path());

        let lib = GnaLibrary::load_from_env("TEST_SYM_TABLE_FILE").unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        drop(lib);

        let lib = GnaLibrary::load_from_env("TEST_SYM_TABLE_DIR").unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        drop(lib);

        let lib = GnaLibrary::load_from_env_or_path("TEST_SYM_TABLE_FILE", "missing_fallback.dll")
            .unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        drop(lib);

        let lib =
            GnaLibrary::load_from_env_or_path("TEST_SYM_TABLE_MISSING_VAR", &dll_path).unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        drop(lib);

        let lib = GnaLibraryBuilder::new().path(&dll_path).load().unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        drop(lib);

        let lib = GnaLibraryBuilder::new()
            .env("TEST_SYM_TABLE_DIR")
            .load()
            .unwrap();
        assert_eq!(lib.path(), dll_path.as_path());
        drop(lib);
    }
}

#[test]
fn test_symbol_table_load_required_only_symbols() {
    let dir = tempdir().unwrap();
    let source = build_required_source(None);
    let dll_path = compile_mock_dll(dir.path(), "req_only", "req_only_gna.dll", &source);

    {
        let raw_lib = unsafe { libloading::Library::new(&dll_path) }.unwrap();
        let table = GnaSymbolTable::load(&raw_lib).unwrap();

        assert!(table.device_create_for_export().is_none());
        assert!(table.device_set_number_of_threads().is_none());
        assert!(table.get_library_version().is_none());
        assert!(table.memory_alloc_for_device().is_none());
        assert!(table.memory_set_tag().is_none());
        assert!(table.model_create().is_none());
        assert!(table.model_release().is_none());
        assert!(table.model_get_last_error().is_none());
        assert!(table.model_error_get_message().is_none());
        assert!(table.model_error_get_max_message_length().is_none());
        assert!(table.operation_init_fully_connected_affine().is_none());
        assert!(table.request_config_create().is_none());
        assert!(table.request_config_set_operand_buffer().is_none());
        assert!(table.request_config_enable_active_list().is_none());
        assert!(table.request_config_enable_hardware_consistency().is_none());
        assert!(table.request_config_set_acceleration_mode().is_none());
        assert!(table.request_config_release().is_none());
        assert!(table.request_enqueue().is_none());
        assert!(table.request_wait().is_none());
        assert!(table.instrumentation_config_create().is_none());
        assert!(
            table
                .instrumentation_config_assign_to_request_config()
                .is_none()
        );
        assert!(table.instrumentation_config_set_unit().is_none());
        assert!(table.instrumentation_config_set_mode().is_none());
        assert!(table.instrumentation_config_release().is_none());
        assert!(table.model_export_config_create().is_none());
        assert!(table.model_export_config_release().is_none());
        assert!(table.model_export_config_set_source().is_none());
        assert!(table.model_export_config_set_target().is_none());
        assert!(table.model_export().is_none());
        assert!(table.model_override_alignment().is_none());
        assert!(table.status_get_message().is_none());
        assert!(table.status_get_max_message_length().is_none());

        let mut count = 0u32;
        assert_eq!(unsafe { (table.device_get_count())(&mut count) }, 0);
        assert_eq!(count, 1);

        drop(raw_lib);
    }

    let lib = GnaLibrary::load_from_path(&dll_path).unwrap();
    assert!(matches!(lib.get_library_version(), Err(GnaError::Other(_))));
    assert!(matches!(
        lib.override_alignment(64),
        Err(GnaError::Other(_))
    ));
    drop(lib);
}

#[test]
fn test_symbol_table_missing_device_get_count() {
    assert_missing_required_symbol("Gna2DeviceGetCount");
}

#[test]
fn test_symbol_table_missing_device_get_version() {
    assert_missing_required_symbol("Gna2DeviceGetVersion");
}

#[test]
fn test_symbol_table_missing_device_open() {
    assert_missing_required_symbol("Gna2DeviceOpen");
}

#[test]
fn test_symbol_table_missing_device_close() {
    assert_missing_required_symbol("Gna2DeviceClose");
}

#[test]
fn test_symbol_table_missing_memory_alloc() {
    assert_missing_required_symbol("Gna2MemoryAlloc");
}

#[test]
fn test_symbol_table_missing_memory_free() {
    assert_missing_required_symbol("Gna2MemoryFree");
}

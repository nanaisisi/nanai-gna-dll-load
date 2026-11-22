use nanai_gna_dll_load::{
    error::GnaError,
    loader::GnaLibrary,
    model_export::GnaModelExportConfig,
    types::{GNA2_STATUS_SUCCESS, Gna2DeviceVersion, Gna2ModelExportComponent},
};

use super::mock::{
    MOCK_STATE, free_user_allocation, full_export_symbol_table, remaining_user_allocations,
    reset_mock_state, test_user_allocator,
};

#[test]
fn test_model_export_full_lifecycle_and_allocator() {
    reset_mock_state();
    let library = GnaLibrary::from_symbol_table(full_export_symbol_table());

    let mut config =
        GnaModelExportConfig::create(&library, test_user_allocator).expect("create export config");
    assert_eq!(config.id(), 42);

    MOCK_STATE.with(|s| {
        let state = s.borrow();
        assert!(state.allocator_zero_was_null);
        assert!(state.allocator_scratch_succeeded);
    });

    config.set_source(1, 99).expect("set source");
    config
        .set_target(Gna2DeviceVersion::V3_0)
        .expect("set target");

    let (buf, size) = config
        .export(Gna2ModelExportComponent::LayerDescriptors)
        .expect("export component");
    assert!(!buf.is_null());
    assert_eq!(size, 20);

    let exported_bytes = unsafe { std::slice::from_raw_parts(buf as *const u8, size as usize) };
    assert_eq!(exported_bytes, b"GNA_EXPORT_BINARY_V3");
    free_user_allocation(buf);
    assert_eq!(remaining_user_allocations(), 0);

    MOCK_STATE.with(|s| {
        let state = s.borrow();
        assert_eq!(state.recorded_source, Some((42, 1, 99)));
        assert_eq!(state.recorded_target, Some((42, Gna2DeviceVersion::V3_0)));
        assert_eq!(
            state.recorded_export_call,
            Some((42, Gna2ModelExportComponent::LayerDescriptors))
        );
        assert_eq!(state.release_calls, 0);
    });

    drop(config);

    MOCK_STATE.with(|s| {
        let state = s.borrow();
        assert_eq!(state.release_calls, 1);
        assert_eq!(state.released_ids, vec![42]);
    });
    assert_eq!(remaining_user_allocations(), 0);
}

#[test]
fn test_model_export_drop_without_release_symbol() {
    reset_mock_state();
    let mut syms = full_export_symbol_table();
    syms.model_export_config_release = None;
    let library = GnaLibrary::from_symbol_table(syms);

    let config =
        GnaModelExportConfig::create(&library, test_user_allocator).expect("create export config");
    assert_eq!(config.id(), 42);
    drop(config);

    MOCK_STATE.with(|s| {
        assert_eq!(s.borrow().release_calls, 0);
    });
    assert_eq!(remaining_user_allocations(), 0);
}

#[test]
fn test_model_export_missing_symbols() {
    reset_mock_state();

    // 1. Missing model_export_config_create
    let mut syms = full_export_symbol_table();
    syms.model_export_config_create = None;
    let lib = GnaLibrary::from_symbol_table(syms);
    let err = GnaModelExportConfig::create(&lib, test_user_allocator)
        .err()
        .expect("expected missing create symbol error");
    assert!(
        matches!(err, GnaError::Other(ref msg) if msg.contains("Gna2ModelExportConfigCreate is not supported"))
    );

    // 2. Missing model_export_config_set_source
    let mut syms = full_export_symbol_table();
    syms.model_export_config_set_source = None;
    let lib = GnaLibrary::from_symbol_table(syms);
    let mut cfg = GnaModelExportConfig::create(&lib, test_user_allocator).expect("create");
    let err = cfg
        .set_source(0, 1)
        .expect_err("expected missing set_source symbol error");
    assert!(
        matches!(err, GnaError::Other(ref msg) if msg.contains("Gna2ModelExportConfigSetSource is not supported"))
    );

    // 3. Missing model_export_config_set_target
    let mut syms = full_export_symbol_table();
    syms.model_export_config_set_target = None;
    let lib = GnaLibrary::from_symbol_table(syms);
    let mut cfg = GnaModelExportConfig::create(&lib, test_user_allocator).expect("create");
    let err = cfg
        .set_target(Gna2DeviceVersion::V3_5)
        .expect_err("expected missing set_target symbol error");
    assert!(
        matches!(err, GnaError::Other(ref msg) if msg.contains("Gna2ModelExportConfigSetTarget is not supported"))
    );

    // 4. Missing model_export
    let mut syms = full_export_symbol_table();
    syms.model_export = None;
    let lib = GnaLibrary::from_symbol_table(syms);
    let cfg = GnaModelExportConfig::create(&lib, test_user_allocator).expect("create");
    let err = cfg
        .export(Gna2ModelExportComponent::ReadOnlyDump)
        .expect_err("expected missing export symbol error");
    assert!(
        matches!(err, GnaError::Other(ref msg) if msg.contains("Gna2ModelExport is not supported"))
    );
    assert_eq!(remaining_user_allocations(), 0);
}

#[test]
fn test_model_export_error_statuses() {
    reset_mock_state();
    let library = GnaLibrary::from_symbol_table(full_export_symbol_table());

    // 1. create returns error status
    MOCK_STATE.with(|s| s.borrow_mut().create_status = -8);
    let err = GnaModelExportConfig::create(&library, test_user_allocator)
        .err()
        .expect("expected create status error");
    assert!(matches!(err, GnaError::StatusError { status: -8, .. }));

    // 2. set_source, set_target, export return error statuses
    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.create_status = GNA2_STATUS_SUCCESS;
        state.set_source_status = -5;
        state.set_target_status = -11;
        state.export_status = -4;
    });

    let mut config = GnaModelExportConfig::create(&library, test_user_allocator).expect("create");

    let err_src = config
        .set_source(0, 10)
        .expect_err("expected set_source status error");
    assert!(matches!(err_src, GnaError::StatusError { status: -5, .. }));

    let err_tgt = config
        .set_target(Gna2DeviceVersion::EMBEDDED_3_1)
        .expect_err("expected set_target status error");
    assert!(matches!(err_tgt, GnaError::StatusError { status: -11, .. }));

    let err_exp = config
        .export(Gna2ModelExportComponent::ScratchDump)
        .expect_err("expected export status error");
    assert!(matches!(err_exp, GnaError::StatusError { status: -4, .. }));
    assert_eq!(remaining_user_allocations(), 0);
}

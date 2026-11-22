use nanai_gna_dll_load::error::GnaError;
use nanai_gna_dll_load::inference::GnaRequestConfig;
use nanai_gna_dll_load::instrumentation::GnaInstrumentationConfig;
use nanai_gna_dll_load::loader::GnaLibrary;
use nanai_gna_dll_load::types::{
    Gna2AccelerationMode, Gna2DeviceVersion, Gna2InstrumentationPoint,
};

use super::mock::{MOCK_STATE, mock_request_config_create, reset_mock_state};
use super::symbols::{empty_symbols, full_symbols};

#[test]
fn test_request_config_enable_performance_counter_branches() {
    reset_mock_state();

    // Case 1: set_unit missing or failing is ignored by enable_performance_counter
    let mut syms_no_unit = full_symbols();
    syms_no_unit.clear_instrumentation_config_set_unit();
    let lib_no_unit = GnaLibrary::from_symbol_table(syms_no_unit);
    let mut cfg = GnaRequestConfig::create(&lib_no_unit, 1).unwrap();
    cfg.enable_performance_counter()
        .expect("missing set_unit should be tolerated");

    // Case 2: assign_to_request_config missing -> Err
    let mut syms_no_assign = full_symbols();
    syms_no_assign.clear_instrumentation_config_assign_to_request_config();
    let lib_no_assign = GnaLibrary::from_symbol_table(syms_no_assign);
    let mut cfg_no_assign = GnaRequestConfig::create(&lib_no_assign, 1).unwrap();
    assert!(matches!(
        cfg_no_assign.enable_performance_counter(),
        Err(GnaError::Other(_))
    ));

    // Case 3: instrumentation_config_create missing -> Err
    let mut syms_no_inst_create = full_symbols();
    syms_no_inst_create.clear_instrumentation_config_create();
    let lib_no_inst = GnaLibrary::from_symbol_table(syms_no_inst_create);
    let mut cfg_no_inst = GnaRequestConfig::create(&lib_no_inst, 1).unwrap();
    assert!(matches!(
        cfg_no_inst.enable_performance_counter(),
        Err(GnaError::Other(_))
    ));

    // Case 4: assign returns error status -> Err
    let lib_full = GnaLibrary::from_symbol_table(full_symbols());
    let mut cfg_err = GnaRequestConfig::create(&lib_full, 1).unwrap();
    MOCK_STATE.with(|s| s.borrow_mut().inst_assign_status = -8);
    assert!(matches!(
        cfg_err.enable_performance_counter(),
        Err(GnaError::StatusError { status: -8, .. })
    ));
}

#[test]
fn test_request_config_missing_symbols() {
    reset_mock_state();

    // Missing create
    let lib_empty = GnaLibrary::from_symbol_table(empty_symbols());
    assert!(matches!(
        GnaRequestConfig::create(&lib_empty, 0),
        Err(GnaError::Other(_))
    ));

    // Has create and release only -> all other methods report missing symbol
    let syms_create_only = empty_symbols().with_request_config_create(mock_request_config_create);
    let lib = GnaLibrary::from_symbol_table(syms_create_only);
    let mut cfg = GnaRequestConfig::create(&lib, 0).unwrap();

    let mut buf = [0u8; 4];
    assert!(matches!(
        unsafe { cfg.set_operand_buffer(0, 0, buf.as_mut_ptr().cast()) },
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        cfg.enable_active_list(0, &[1]),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        cfg.enable_hardware_consistency(Gna2DeviceVersion::V3_0),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        cfg.set_acceleration_mode(Gna2AccelerationMode::Software),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(cfg.enqueue(), Err(GnaError::Other(_))));
    assert!(matches!(cfg.wait_request(1, 10), Err(GnaError::Other(_))));
    assert!(matches!(cfg.wait(1, 10), Err(GnaError::Other(_))));

    // Drop without request_config_release should be a safe no-op
    drop(cfg);
    MOCK_STATE.with(|s| {
        assert!(s.borrow().released_config_ids.is_empty());
    });
}

#[test]
fn test_request_config_error_statuses_and_inst_errors() {
    reset_mock_state();
    let lib = GnaLibrary::from_symbol_table(full_symbols());

    MOCK_STATE.with(|s| s.borrow_mut().create_status = -1);
    assert!(matches!(
        GnaRequestConfig::create(&lib, 2),
        Err(GnaError::StatusError { status: -1, .. })
    ));

    reset_mock_state();
    let mut cfg = GnaRequestConfig::create(&lib, 2).unwrap();

    MOCK_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.set_operand_buffer_status = -6;
        st.enable_active_list_status = -5;
        st.enable_hardware_consistency_status = -11;
        st.set_acceleration_mode_status = -4;
        st.enqueue_status = -12;
        st.wait_status = -13;
    });

    let mut buf = [0u8; 4];
    assert!(matches!(
        unsafe { cfg.set_operand_buffer(0, 0, buf.as_mut_ptr().cast()) },
        Err(GnaError::StatusError { status: -6, .. })
    ));
    assert!(matches!(
        cfg.enable_active_list(0, &[0]),
        Err(GnaError::StatusError { status: -5, .. })
    ));
    assert!(matches!(
        cfg.enable_hardware_consistency(Gna2DeviceVersion::V2_0),
        Err(GnaError::StatusError { status: -11, .. })
    ));
    assert!(matches!(
        cfg.set_acceleration_mode(Gna2AccelerationMode::Hardware),
        Err(GnaError::StatusError { status: -4, .. })
    ));
    assert!(matches!(
        cfg.enqueue(),
        Err(GnaError::StatusError { status: -12, .. })
    ));
    assert!(matches!(
        cfg.wait_request(1, 10),
        Err(GnaError::StatusError { status: -13, .. })
    ));
    assert!(matches!(
        cfg.wait(1, 10),
        Err(GnaError::StatusError { status: -13, .. })
    ));

    // Propagate instrumentation errors through get_hw_usage / get_performance_stats
    let inst_missing_points =
        GnaInstrumentationConfig::create(&lib, &[Gna2InstrumentationPoint::LibExecution]).unwrap();
    cfg.attach_instrumentation(inst_missing_points).unwrap();
    assert!(matches!(cfg.get_hw_usage(), Err(GnaError::Other(_))));
    assert!(matches!(
        cfg.get_performance_stats(),
        Err(GnaError::Other(_))
    ));
}

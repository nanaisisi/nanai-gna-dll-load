use nanai_gna_dll_load::error::GnaError;
use nanai_gna_dll_load::instrumentation::GnaInstrumentationConfig;
use nanai_gna_dll_load::loader::GnaLibrary;
use nanai_gna_dll_load::types::{
    Gna2InstrumentationMode, Gna2InstrumentationPoint, Gna2InstrumentationUnit,
};

use super::mock::{
    MOCK_INST_STATE, empty_symbols, full_inst_symbols, mock_inst_create, reset_mock_inst_state,
};

#[test]
fn test_instrumentation_create_validation_and_missing_symbol() {
    reset_mock_inst_state();
    let lib = GnaLibrary::from_symbol_table(full_inst_symbols());

    // Empty points slice
    assert!(matches!(
        GnaInstrumentationConfig::create(&lib, &[]),
        Err(GnaError::Other(_))
    ));

    // Too many points (> 15)
    let too_many = [Gna2InstrumentationPoint::HwTotalCycles; 16];
    assert!(matches!(
        GnaInstrumentationConfig::create(&lib, &too_many),
        Err(GnaError::Other(_))
    ));

    // Max valid count (15)
    let max_points = [Gna2InstrumentationPoint::HwTotalCycles; 15];
    let inst_15 = GnaInstrumentationConfig::create(&lib, &max_points)
        .expect("15 instrumentation points should be valid");
    assert_eq!(inst_15.points().len(), 15);
    assert_eq!(inst_15.results().len(), 15);
    drop(inst_15);

    // Missing create symbol
    let empty_lib = GnaLibrary::from_symbol_table(empty_symbols());
    assert!(matches!(
        GnaInstrumentationConfig::create(&empty_lib, &[Gna2InstrumentationPoint::HwTotalCycles]),
        Err(GnaError::Other(_))
    ));

    // Non-success status from create
    MOCK_INST_STATE.with(|s| s.borrow_mut().create_status = -8);
    assert!(matches!(
        GnaInstrumentationConfig::create(&lib, &[Gna2InstrumentationPoint::HwTotalCycles]),
        Err(GnaError::StatusError { status: -8, .. })
    ));
}

#[test]
fn test_instrumentation_config_methods_success_and_drop() {
    reset_mock_inst_state();
    let lib = GnaLibrary::from_symbol_table(full_inst_symbols());

    let points = [
        Gna2InstrumentationPoint::LibPreprocessing,
        Gna2InstrumentationPoint::HwTotalCycles,
        Gna2InstrumentationPoint::HwStallCycles,
        Gna2InstrumentationPoint::LibExecution,
    ];
    let mut inst = GnaInstrumentationConfig::create(&lib, &points).expect("create should succeed");
    let inst_id = inst.id();
    assert_eq!(inst_id, 1);
    assert_eq!(inst.points(), &points);
    assert_eq!(inst.results(), &[7, 1000, 250, 42]);

    inst.assign_to_request_config(99)
        .expect("assign_to_request_config should succeed");
    inst.set_unit(Gna2InstrumentationUnit::Cycles)
        .expect("set_unit should succeed");
    inst.set_mode(Gna2InstrumentationMode::WaitForDmaCompletion)
        .expect("set_mode should succeed");

    let hw_usage = inst.compute_hw_usage().expect("hw usage should succeed");
    assert!((hw_usage - 0.75).abs() < 1e-6);

    let stats = inst
        .compute_performance_stats()
        .expect("compute_performance_stats should succeed");
    assert_eq!(stats.total_cycles(), 1000);
    assert_eq!(stats.stall_cycles(), 250);
    assert_eq!(stats.active_cycles(), 750);
    assert!((stats.hw_usage_ratio() - 0.75).abs() < 1e-6);
    assert_eq!(stats.execution_time(), Some(42));

    MOCK_INST_STATE.with(|s| {
        let st = s.borrow();
        assert_eq!(st.last_assign, Some((inst_id, 99)));
        assert_eq!(
            st.last_unit,
            Some((inst_id, Gna2InstrumentationUnit::Cycles))
        );
        assert_eq!(
            st.last_mode,
            Some((inst_id, Gna2InstrumentationMode::WaitForDmaCompletion))
        );
    });

    drop(inst);

    MOCK_INST_STATE.with(|s| {
        assert_eq!(s.borrow().released_ids, vec![inst_id]);
    });
}

#[test]
fn test_instrumentation_config_missing_symbols_and_errors() {
    reset_mock_inst_state();

    // Create-only symbol table: assign, set_unit, set_mode, release are all None
    let create_only = empty_symbols().with_instrumentation_config_create(mock_inst_create);
    let lib_create_only = GnaLibrary::from_symbol_table(create_only);
    let mut inst_missing = GnaInstrumentationConfig::create(
        &lib_create_only,
        &[Gna2InstrumentationPoint::HwTotalCycles],
    )
    .unwrap();

    assert!(matches!(
        inst_missing.assign_to_request_config(1),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        inst_missing.set_unit(Gna2InstrumentationUnit::Microseconds),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        inst_missing.set_mode(Gna2InstrumentationMode::TotalStall),
        Err(GnaError::Other(_))
    ));

    // Drop without release symbol should safely do nothing
    drop(inst_missing);
    MOCK_INST_STATE.with(|s| assert!(s.borrow().released_ids.is_empty()));

    // Error status paths for assign, set_unit, set_mode
    let lib_full = GnaLibrary::from_symbol_table(full_inst_symbols());
    let mut inst_err =
        GnaInstrumentationConfig::create(&lib_full, &[Gna2InstrumentationPoint::HwTotalCycles])
            .unwrap();

    MOCK_INST_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.assign_status = -5;
        st.set_unit_status = -5;
        st.set_mode_status = -5;
    });

    assert!(matches!(
        inst_err.assign_to_request_config(1),
        Err(GnaError::StatusError { status: -5, .. })
    ));
    assert!(matches!(
        inst_err.set_unit(Gna2InstrumentationUnit::Milliseconds),
        Err(GnaError::StatusError { status: -5, .. })
    ));
    assert!(matches!(
        inst_err.set_mode(Gna2InstrumentationMode::Disabled),
        Err(GnaError::StatusError { status: -5, .. })
    ));
}

#[test]
fn test_instrumentation_edge_cases_zero_cycles_stall_gt_total_and_missing_points() {
    reset_mock_inst_state();
    let lib = GnaLibrary::from_symbol_table(full_inst_symbols());

    // Missing HwTotalCycles
    let inst_no_total =
        GnaInstrumentationConfig::create(&lib, &[Gna2InstrumentationPoint::HwStallCycles]).unwrap();
    assert!(matches!(
        inst_no_total.compute_hw_usage(),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        inst_no_total.compute_performance_stats(),
        Err(GnaError::Other(_))
    ));

    // Missing HwStallCycles
    let inst_no_stall =
        GnaInstrumentationConfig::create(&lib, &[Gna2InstrumentationPoint::HwTotalCycles]).unwrap();
    assert!(matches!(
        inst_no_stall.compute_hw_usage(),
        Err(GnaError::Other(_))
    ));
    assert!(matches!(
        inst_no_stall.compute_performance_stats(),
        Err(GnaError::Other(_))
    ));

    // Zero total cycles and no LibExecution point
    MOCK_INST_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.total_cycles = 0;
        st.stall_cycles = 0;
    });
    let inst_zero = GnaInstrumentationConfig::create(
        &lib,
        &[
            Gna2InstrumentationPoint::HwTotalCycles,
            Gna2InstrumentationPoint::HwStallCycles,
        ],
    )
    .unwrap();
    assert!(matches!(
        inst_zero.compute_hw_usage(),
        Err(GnaError::Other(_))
    ));
    let zero_stats = inst_zero
        .compute_performance_stats()
        .expect("compute_performance_stats should succeed on zero cycles");
    assert_eq!(zero_stats.total_cycles(), 0);
    assert_eq!(zero_stats.stall_cycles(), 0);
    assert_eq!(zero_stats.active_cycles(), 0);
    assert_eq!(zero_stats.hw_usage_ratio(), 0.0);
    assert_eq!(zero_stats.execution_time(), None);

    // Stall cycles greater than total cycles -> saturating_sub clamps to 0 active cycles
    MOCK_INST_STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.total_cycles = 100;
        st.stall_cycles = 250;
    });
    let inst_stall_gt = GnaInstrumentationConfig::create(
        &lib,
        &[
            Gna2InstrumentationPoint::HwTotalCycles,
            Gna2InstrumentationPoint::HwStallCycles,
        ],
    )
    .unwrap();
    assert_eq!(inst_stall_gt.compute_hw_usage().unwrap(), 0.0);
    let stall_stats = inst_stall_gt.compute_performance_stats().unwrap();
    assert_eq!(stall_stats.total_cycles(), 100);
    assert_eq!(stall_stats.stall_cycles(), 250);
    assert_eq!(stall_stats.active_cycles(), 0);
    assert_eq!(stall_stats.hw_usage_ratio(), 0.0);
    assert_eq!(stall_stats.execution_time(), None);
}

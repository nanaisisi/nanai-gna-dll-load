use std::ffi::c_void;

use nanai_gna_dll_load::error::GnaError;
use nanai_gna_dll_load::inference::GnaRequestConfig;
use nanai_gna_dll_load::instrumentation::GnaInstrumentationConfig;
use nanai_gna_dll_load::loader::GnaLibrary;
use nanai_gna_dll_load::types::{
    Gna2AccelerationMode, Gna2DeviceVersion, Gna2InstrumentationPoint,
};

use super::mock::{MOCK_STATE, reset_mock_state};
use super::symbols::full_symbols;

#[test]
fn test_request_config_full_lifecycle_and_methods() {
    reset_mock_state();
    let lib = GnaLibrary::from_symbol_table(full_symbols());

    let mut cfg = GnaRequestConfig::create(&lib, 7).expect("create should succeed");
    assert_eq!(cfg.id(), 10);
    assert!(cfg.instrumentation().is_none());

    // Calling hw usage / stats before instrumentation is configured must error
    assert!(matches!(cfg.get_hw_usage(), Err(GnaError::Other(_))));
    assert!(matches!(
        cfg.get_performance_stats(),
        Err(GnaError::Other(_))
    ));

    let mut buffer = [1u8; 16];
    let buf_ptr = buffer.as_mut_ptr().cast::<c_void>();
    unsafe { cfg.set_operand_buffer(1, 2, buf_ptr) }.expect("set_operand_buffer should succeed");

    cfg.enable_active_list(3, &[0, 2, 5])
        .expect("enable_active_list should succeed");
    cfg.enable_hardware_consistency(Gna2DeviceVersion::V3_0)
        .expect("enable_hardware_consistency should succeed");
    cfg.set_acceleration_mode(Gna2AccelerationMode::Hardware)
        .expect("set_acceleration_mode should succeed");
    cfg.enable_performance_counter()
        .expect("enable_performance_counter should succeed");

    assert!(cfg.instrumentation().is_some());

    let req_id = cfg.enqueue().expect("enqueue should succeed");
    assert_eq!(req_id, 100);

    cfg.wait_request(req_id, 500)
        .expect("wait_request should succeed");
    MOCK_STATE.with(|s| {
        assert_eq!(s.borrow().last_wait_args, Some((100, 500)));
    });

    cfg.wait(req_id, 250).expect("wait should succeed");

    let hw_usage = cfg.get_hw_usage().expect("get_hw_usage should succeed");
    assert!((hw_usage - 0.75).abs() < 1e-6);

    let stats = cfg
        .get_performance_stats()
        .expect("get_performance_stats should succeed");
    assert_eq!(stats.total_cycles, 2000);
    assert_eq!(stats.stall_cycles, 500);
    assert_eq!(stats.active_cycles, 1500);
    assert!((stats.hw_usage_ratio - 0.75).abs() < 1e-6);
    assert_eq!(stats.execution_time, Some(80));

    MOCK_STATE.with(|s| {
        let st = s.borrow();
        assert_eq!(st.last_model_id, Some(7));
        assert_eq!(st.last_operand_buf, Some((10, 1, 2, buf_ptr as usize)));
        assert_eq!(st.last_active_list, Some((10, 3, vec![0, 2, 5])));
        assert_eq!(st.last_hw_consistency, Some((10, Gna2DeviceVersion::V3_0)));
        assert_eq!(
            st.last_accel_mode,
            Some((10, Gna2AccelerationMode::Hardware))
        );
        assert_eq!(st.last_enqueue_config_id, Some(10));
        assert_eq!(st.last_wait_args, Some((100, 250)));
    });

    drop(cfg);

    MOCK_STATE.with(|s| {
        let st = s.borrow();
        assert_eq!(st.released_config_ids, vec![10]);
        assert_eq!(st.released_inst_ids, vec![50]);
    });
}

#[test]
fn test_request_config_attach_instrumentation() {
    reset_mock_state();
    let lib = GnaLibrary::from_symbol_table(full_symbols());
    let mut cfg = GnaRequestConfig::create(&lib, 1).unwrap();

    let inst = GnaInstrumentationConfig::create(
        &lib,
        &[
            Gna2InstrumentationPoint::HwTotalCycles,
            Gna2InstrumentationPoint::HwStallCycles,
        ],
    )
    .unwrap();

    cfg.attach_instrumentation(inst)
        .expect("attach_instrumentation should succeed");
    assert!(cfg.instrumentation().is_some());
    assert!((cfg.get_hw_usage().unwrap() - 0.75).abs() < 1e-6);

    // Attach failure should propagate error and leave existing instrumentation untouched
    MOCK_STATE.with(|s| s.borrow_mut().inst_assign_status = -5);
    let inst_fail =
        GnaInstrumentationConfig::create(&lib, &[Gna2InstrumentationPoint::HwTotalCycles]).unwrap();
    let err = cfg.attach_instrumentation(inst_fail).unwrap_err();
    assert!(matches!(err, GnaError::StatusError { status: -5, .. }));
}

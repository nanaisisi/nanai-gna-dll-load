use nanai_gna_dll_load::{
    device::GnaDevice,
    error::GnaError,
    loader::GnaLibrary,
    model::{GnaModel, GnaModelBuilder},
    types::{GNA2_STATUS_MODEL_ERROR_UNAVAILABLE, GNA2_STATUS_SUCCESS, Gna2Operation},
};

use super::mock::{MOCK_STATE, free_tracked_heap_allocations, reset_mock_state};
use super::symbols::{full_mock_symbol_table, sample_tensors};

#[test]
fn test_model_build_missing_model_create_and_empty_layers() {
    reset_mock_state();
    let mut no_create_syms = full_mock_symbol_table();
    no_create_syms.model_create = None;
    let no_create_lib = GnaLibrary::from_symbol_table(no_create_syms);
    let device_no_create = GnaDevice::open(&no_create_lib, 0).expect("open device");

    let err = GnaModelBuilder::new()
        .add_operation(Gna2Operation::default())
        .build(&device_no_create)
        .err()
        .expect("expected missing symbol error");
    assert!(
        matches!(err, GnaError::Other(ref msg) if msg.contains("Gna2ModelCreate is not supported")),
        "unexpected error: {err:?}"
    );

    let full_lib = GnaLibrary::from_symbol_table(full_mock_symbol_table());
    let device = GnaDevice::open(&full_lib, 0).expect("open device");
    let empty_err = GnaModelBuilder::default()
        .build(&device)
        .err()
        .expect("expected empty layers error");
    assert!(
        matches!(empty_err, GnaError::Other(ref msg) if msg.contains("at least one layer")),
        "unexpected error: {empty_err:?}"
    );
}

#[test]
fn test_model_build_fca_init_failure() {
    reset_mock_state();
    let library = GnaLibrary::from_symbol_table(full_mock_symbol_table());
    let device = GnaDevice::open(&library, 0).expect("open device");

    let mut in_buf = vec![0i16; 64];
    let mut out_buf = vec![0i32; 32];
    let mut w_buf = vec![0i16; 128];
    let mut b_buf = vec![0i32; 8];
    let (inp1, out1, w1, b1) = sample_tensors(&mut in_buf, &mut out_buf, &mut w_buf, &mut b_buf);
    let (inp2, out2, w2, b2) = sample_tensors(&mut in_buf, &mut out_buf, &mut w_buf, &mut b_buf);

    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.fca_init_status = -6;
        state.fca_init_fail_on_call = Some(2);
    });

    let err = GnaModelBuilder::new()
        .add_fully_connected_affine(inp1, out1, w1, b1, None)
        .add_fully_connected_affine(inp2, out2, w2, b2, None)
        .build(&device)
        .err()
        .expect("expected fca_init failure");

    assert!(matches!(err, GnaError::StatusError { status: -6, .. }));
    MOCK_STATE.with(|s| {
        let state = s.borrow();
        assert_eq!(state.fca_init_calls, 2);
        assert_eq!(state.model_create_calls, 0);
    });
    free_tracked_heap_allocations();
}

#[test]
fn test_model_build_create_error_with_status_and_last_error_detail() {
    reset_mock_state();
    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.model_create_status = -5;
        state.status_get_message_status = GNA2_STATUS_SUCCESS;
        state.status_message = String::from("Invalid item identifier");
        state.get_last_error_status = GNA2_STATUS_SUCCESS;
        state.max_error_message_len = 128; // Clamped to 256 by .max(256)
        state.error_get_message_status = GNA2_STATUS_SUCCESS;
        state.error_message = String::from("Operand shape mismatch");
    });

    let library = GnaLibrary::from_symbol_table(full_mock_symbol_table());
    let device = GnaDevice::open(&library, 0).expect("open device");

    let err = GnaModelBuilder::new()
        .add_operation(Gna2Operation::default())
        .build(&device)
        .err()
        .expect("expected ModelCreationError");

    match err {
        GnaError::ModelCreationError { status, detail } => {
            assert_eq!(status, -5);
            assert_eq!(detail, "Operand shape mismatch");
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    MOCK_STATE.with(|s| {
        assert_eq!(s.borrow().status_get_message_calls, 1);
    });
}

#[test]
fn test_model_build_create_error_fallbacks() {
    // Subcase A: status_get_message returns failure, model_get_last_error is None
    reset_mock_state();
    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.model_create_status = -9;
        state.status_get_message_status = -1;
    });
    let mut syms_a = full_mock_symbol_table();
    syms_a.model_get_last_error = None;
    let lib_a = GnaLibrary::from_symbol_table(syms_a);
    let dev_a = GnaDevice::open(&lib_a, 0).expect("open device");

    let err_a = GnaModelBuilder::new()
        .add_operation(Gna2Operation::default())
        .build(&dev_a)
        .err()
        .expect("expected ModelCreationError");
    match err_a {
        GnaError::ModelCreationError { status, detail } => {
            assert_eq!(status, -9);
            assert_eq!(detail, "Failed to compile model on device 0. Status: -9");
        }
        other => panic!("unexpected error: {other:?}"),
    }
    MOCK_STATE.with(|s| {
        assert_eq!(s.borrow().status_get_message_calls, 1);
    });

    // Subcase B: status_get_message is None, model_get_last_error returns unavailable
    reset_mock_state();
    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.model_create_status = -11;
        state.get_last_error_status = GNA2_STATUS_MODEL_ERROR_UNAVAILABLE;
    });
    let mut syms_b = full_mock_symbol_table();
    syms_b.status_get_message = None;
    let lib_b = GnaLibrary::from_symbol_table(syms_b);
    let dev_b = GnaDevice::open(&lib_b, 0).expect("open device");

    let err_b = GnaModelBuilder::new()
        .add_operation(Gna2Operation::default())
        .build(&dev_b)
        .err()
        .expect("expected ModelCreationError");
    match err_b {
        GnaError::ModelCreationError { status, detail } => {
            assert_eq!(status, -11);
            assert_eq!(detail, "Failed to compile model on device 0. Status: -11");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn test_get_last_error_message_all_branches() {
    reset_mock_state();

    // 1. model_get_last_error missing -> None
    let mut syms = full_mock_symbol_table();
    syms.model_get_last_error = None;
    let lib = GnaLibrary::from_symbol_table(syms);
    assert_eq!(GnaModel::get_last_error_message(&lib), None);

    // 2. model_error_get_message missing -> None
    let mut syms = full_mock_symbol_table();
    syms.model_error_get_message = None;
    let lib = GnaLibrary::from_symbol_table(syms);
    assert_eq!(GnaModel::get_last_error_message(&lib), None);

    // 3. get_err returns GNA2_STATUS_MODEL_ERROR_UNAVAILABLE -> None
    let lib = GnaLibrary::from_symbol_table(full_mock_symbol_table());
    MOCK_STATE.with(|s| {
        s.borrow_mut().get_last_error_status = GNA2_STATUS_MODEL_ERROR_UNAVAILABLE;
    });
    assert_eq!(GnaModel::get_last_error_message(&lib), None);

    // 4. get_err returns negative error status -> None
    MOCK_STATE.with(|s| {
        s.borrow_mut().get_last_error_status = -1;
    });
    assert_eq!(GnaModel::get_last_error_message(&lib), None);

    // 5. model_error_get_max_message_length missing (defaults to 1024), get_msg succeeds
    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.get_last_error_status = GNA2_STATUS_SUCCESS;
        state.error_get_message_status = GNA2_STATUS_SUCCESS;
        state.error_message = String::from("Dimension out of bounds");
    });
    let mut syms = full_mock_symbol_table();
    syms.model_error_get_max_message_length = None;
    let lib_no_maxlen = GnaLibrary::from_symbol_table(syms);
    assert_eq!(
        GnaModel::get_last_error_message(&lib_no_maxlen),
        Some(String::from("Dimension out of bounds"))
    );

    // 6. get_msg returns non-success -> formatted fallback string
    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.get_last_error_status = GNA2_STATUS_SUCCESS;
        state.max_error_message_len = 512;
        state.error_get_message_status = -5;
    });
    let fallback = GnaModel::get_last_error_message(&lib).expect("expected fallback string");
    assert!(fallback.contains("Model item error: type="));
    assert!(fallback.contains("val=123"));
}

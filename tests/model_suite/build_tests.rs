use std::ffi::c_void;
use std::path::Path;
use std::ptr;

use nanai_gna_dll_load::{
    device::GnaDevice,
    loader::GnaLibrary,
    model::GnaModelBuilder,
    types::{Gna2DataType, Gna2Operation, Gna2OperationType, Gna2Tensor, Gna2TensorMode},
};

use super::mock::{MOCK_STATE, reset_mock_state};
use super::symbols::{full_mock_symbol_table, sample_tensors};

#[test]
fn test_model_build_with_fca_init_and_activation() {
    reset_mock_state();
    let library = GnaLibrary::from_symbol_table(full_mock_symbol_table());
    let device = GnaDevice::open(&library, 0).expect("open device");

    let mut in_buf = vec![0i16; 64];
    let mut out_buf = vec![0i32; 32];
    let mut w_buf = vec![0i16; 128];
    let mut b_buf = vec![0i32; 8];
    let mut act_buf = vec![0u8; 32];

    let (inp1, out1, w1, b1) = sample_tensors(&mut in_buf, &mut out_buf, &mut w_buf, &mut b_buf);
    let (inp2, out2, w2, b2) = sample_tensors(&mut in_buf, &mut out_buf, &mut w_buf, &mut b_buf);
    let act2 = Gna2Tensor::d1(
        8,
        Gna2DataType::PwlSegment,
        act_buf.as_mut_ptr() as *mut c_void,
    );

    let model = GnaModelBuilder::new()
        .add_fully_connected_affine(inp1, out1, w1, b1, None)
        .add_fully_connected_affine(inp2, out2, w2, b2, Some(act2))
        .build(&device)
        .expect("build model");

    assert_eq!(model.id(), 77);
    assert_eq!(model.device_index(), 0);
    assert_eq!(model.library().path(), Path::new("mock_gna.dll"));

    MOCK_STATE.with(|s| {
        let state = s.borrow();
        assert_eq!(state.fca_init_calls, 2);
        assert!(state.fca_allocator_zero_was_null);
        assert!(state.fca_allocator_nonzero_was_nonnull);
        assert_eq!(state.model_create_calls, 1);
        assert_eq!(state.last_created_device_index, Some(0));
        assert_eq!(state.last_num_operations, 2);
        assert_eq!(
            state.observed_op_types,
            vec![
                Gna2OperationType::FullyConnectedAffine,
                Gna2OperationType::FullyConnectedAffine,
            ]
        );
        assert_eq!(state.observed_operand_counts, vec![5, 5]);
        assert_eq!(state.observed_first_layer_tensor_modes.len(), 5);
        assert_eq!(
            state.observed_first_layer_tensor_modes[4],
            Gna2TensorMode::Disabled
        );
        assert!(state.heap_allocated_ptrs.is_empty());
        assert_eq!(state.model_release_calls, 0);
    });

    drop(model);

    MOCK_STATE.with(|s| {
        let state = s.borrow();
        assert_eq!(state.model_release_calls, 1);
        assert_eq!(state.released_model_ids, vec![77]);
    });
}

#[test]
fn test_model_build_manual_fca_fallback_and_raw_operations() {
    reset_mock_state();
    let mut symbols = full_mock_symbol_table();
    symbols.operation_init_fully_connected_affine = None;
    let library = GnaLibrary::from_symbol_table(symbols);
    let device = GnaDevice::open(&library, 0).expect("open device");

    let mut in_buf = vec![0i16; 64];
    let mut out_buf = vec![0i32; 32];
    let mut w_buf = vec![0i16; 128];
    let mut b_buf = vec![0i32; 8];
    let (inp, out, w, b) = sample_tensors(&mut in_buf, &mut out_buf, &mut w_buf, &mut b_buf);

    let raw_tensor = Gna2Tensor::d1(4, Gna2DataType::Int8, in_buf.as_mut_ptr() as *mut c_void);
    let raw_operands: [*const Gna2Tensor; 2] = [&raw_tensor as *const Gna2Tensor, ptr::null()];

    let raw_copy_op = Gna2Operation {
        operation_type: Gna2OperationType::Copy,
        operands: raw_operands.as_ptr(),
        number_of_operands: 2,
        parameters: ptr::null_mut(),
        number_of_parameters: 0,
    };

    let raw_empty_op = Gna2Operation {
        operation_type: Gna2OperationType::Convolution,
        operands: ptr::null(),
        number_of_operands: 0,
        parameters: ptr::null_mut(),
        number_of_parameters: 0,
    };

    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.free_fallback_fca_operands = true;
        state.raw_operand_ptrs.push(raw_operands.as_ptr() as usize);
    });

    let model = GnaModelBuilder::default()
        .add_fully_connected_affine(inp, out, w, b, None)
        .add_operation(raw_copy_op)
        .add_operation(raw_empty_op)
        .build(&device)
        .expect("build model via fallback and raw ops");

    assert_eq!(model.id(), 77);

    MOCK_STATE.with(|s| {
        let state = s.borrow();
        assert_eq!(state.fca_init_calls, 0);
        assert_eq!(state.last_num_operations, 3);
        assert_eq!(
            state.observed_op_types,
            vec![
                Gna2OperationType::FullyConnectedAffine,
                Gna2OperationType::Copy,
                Gna2OperationType::Convolution,
            ]
        );
        assert_eq!(state.observed_operand_counts, vec![5, 2, 0]);
        assert_eq!(
            state.observed_first_layer_tensor_modes,
            vec![
                Gna2TensorMode::Default,
                Gna2TensorMode::Default,
                Gna2TensorMode::Default,
                Gna2TensorMode::Default,
                Gna2TensorMode::Disabled,
            ]
        );
        assert_eq!(state.freed_fallback_fca_operands_calls, 1);
    });
}

#[test]
fn test_model_drop_without_release_symbol() {
    reset_mock_state();
    let mut symbols = full_mock_symbol_table();
    symbols.model_release = None;
    let library = GnaLibrary::from_symbol_table(symbols);
    let device = GnaDevice::open(&library, 0).expect("open device");

    let op = Gna2Operation {
        operation_type: Gna2OperationType::Transposition,
        operands: ptr::null(),
        number_of_operands: 0,
        parameters: ptr::null_mut(),
        number_of_parameters: 0,
    };

    let model = GnaModelBuilder::default()
        .add_operation(op)
        .build(&device)
        .expect("build model without release symbol");
    assert_eq!(model.id(), 77);
    drop(model);

    MOCK_STATE.with(|s| {
        assert_eq!(s.borrow().model_release_calls, 0);
    });
}

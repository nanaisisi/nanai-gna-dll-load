use std::cell::RefCell;
use std::ffi::{c_char, c_void};
use std::path::Path;
use std::ptr;

use nanai_gna_dll_load::{
    device::GnaDevice,
    error::GnaError,
    loader::GnaLibrary,
    model::{GnaModel, GnaModelBuilder},
    symbol_table::GnaSymbolTable,
    types::{
        GNA2_MODEL_ITEM_NUMBER_OF_PROPERTIES, GNA2_STATUS_MODEL_ERROR_UNAVAILABLE,
        GNA2_STATUS_SUCCESS, Gna2DataType, Gna2DeviceVersion, Gna2ErrorType, Gna2ItemType,
        Gna2Model, Gna2ModelError, Gna2ModelItem, Gna2Operation, Gna2OperationType, Gna2Shape,
        Gna2Status, Gna2Tensor, Gna2TensorMode, Gna2UserAllocator,
    },
};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetProcessHeap() -> *mut c_void;
    fn HeapFree(hHeap: *mut c_void, dwFlags: u32, lpMem: *mut c_void) -> i32;
}

#[test]
fn test_shape_dimensions() {
    let s0 = Gna2Shape::scalar();
    assert_eq!(s0.number_of_dimensions, 0);
    assert_eq!(s0.total_elements(), 1);

    let s1 = Gna2Shape::d1(64);
    assert_eq!(s1.number_of_dimensions, 1);
    assert_eq!(s1.dimensions[0], 64);
    assert_eq!(s1.total_elements(), 64);

    let s2 = Gna2Shape::d2(16, 8);
    assert_eq!(s2.number_of_dimensions, 2);
    assert_eq!(s2.dimensions[0], 16);
    assert_eq!(s2.dimensions[1], 8);
    assert_eq!(s2.total_elements(), 128);
}

#[test]
fn test_tensor_constructors() {
    let dummy_ptr = 0x1000 as *mut c_void;
    let t = Gna2Tensor::d2(16, 4, Gna2DataType::Int16, dummy_ptr);
    assert_eq!(t.shape.number_of_dimensions, 2);
    assert_eq!(t.shape.dimensions[0], 16);
    assert_eq!(t.shape.dimensions[1], 4);
    assert_eq!(t.mode, Gna2TensorMode::Default);
    assert_eq!(t.data_type, Gna2DataType::Int16);
    assert_eq!(t.data, dummy_ptr);

    let disabled = Gna2Tensor::disabled();
    assert_eq!(disabled.mode, Gna2TensorMode::Disabled);
}

#[test]
fn test_data_type_sizes() {
    assert_eq!(Gna2DataType::Int8.size_in_bytes(), 1);
    assert_eq!(Gna2DataType::Int16.size_in_bytes(), 2);
    assert_eq!(Gna2DataType::Int32.size_in_bytes(), 4);
    assert_eq!(Gna2DataType::Int64.size_in_bytes(), 8);
}

#[test]
fn test_model_builder_construction() {
    let dummy_ptr = 0x1000 as *mut c_void;
    let inp = Gna2Tensor::d2(16, 4, Gna2DataType::Int16, dummy_ptr);
    let out = Gna2Tensor::d2(8, 4, Gna2DataType::Int32, dummy_ptr);
    let w = Gna2Tensor::d2(8, 16, Gna2DataType::Int16, dummy_ptr);
    let b = Gna2Tensor::d1(8, Gna2DataType::Int32, dummy_ptr);

    let _builder = GnaModelBuilder::new().add_fully_connected_affine(inp, out, w, b, None);

    // Verify builder builds safely (requires device to compile)
    assert_eq!(Gna2OperationType::FullyConnectedAffine as u32, 3);
}

struct MockModelState {
    model_create_status: Gna2Status,
    next_model_id: u32,
    model_create_calls: u32,
    last_created_device_index: Option<u32>,
    last_num_operations: u32,
    observed_op_types: Vec<Gna2OperationType>,
    observed_operand_counts: Vec<u32>,
    observed_first_layer_tensor_modes: Vec<Gna2TensorMode>,
    free_fallback_fca_operands: bool,
    freed_fallback_fca_operands_calls: u32,
    raw_operand_ptrs: Vec<usize>,
    heap_allocated_ptrs: Vec<usize>,

    model_release_calls: u32,
    released_model_ids: Vec<u32>,

    fca_init_status: Gna2Status,
    fca_init_fail_on_call: Option<u32>,
    fca_init_calls: u32,
    fca_allocator_zero_was_null: bool,
    fca_allocator_nonzero_was_nonnull: bool,

    get_last_error_status: Gna2Status,
    last_error_desc: Gna2ModelError,
    error_get_message_status: Gna2Status,
    error_message: String,
    max_error_message_len: u32,

    status_get_message_status: Gna2Status,
    status_message: String,
    status_get_message_calls: u32,
}

impl Default for MockModelState {
    fn default() -> Self {
        Self {
            model_create_status: GNA2_STATUS_SUCCESS,
            next_model_id: 77,
            model_create_calls: 0,
            last_created_device_index: None,
            last_num_operations: 0,
            observed_op_types: Vec::new(),
            observed_operand_counts: Vec::new(),
            observed_first_layer_tensor_modes: Vec::new(),
            free_fallback_fca_operands: false,
            freed_fallback_fca_operands_calls: 0,
            raw_operand_ptrs: Vec::new(),
            heap_allocated_ptrs: Vec::new(),
            model_release_calls: 0,
            released_model_ids: Vec::new(),
            fca_init_status: GNA2_STATUS_SUCCESS,
            fca_init_fail_on_call: None,
            fca_init_calls: 0,
            fca_allocator_zero_was_null: false,
            fca_allocator_nonzero_was_nonnull: false,
            get_last_error_status: GNA2_STATUS_SUCCESS,
            last_error_desc: Gna2ModelError {
                source: Gna2ModelItem {
                    item_type: Gna2ItemType::SHAPE_DIMENSIONS,
                    operation_index: 0,
                    operand_index: 2,
                    parameter_index: -1,
                    shape_dimension_index: 1,
                    properties: [0; GNA2_MODEL_ITEM_NUMBER_OF_PROPERTIES],
                },
                reason: Gna2ErrorType::BELOW_RANGE,
                value: 123,
            },
            error_get_message_status: GNA2_STATUS_SUCCESS,
            error_message: String::from("Mock model error"),
            max_error_message_len: 512,
            status_get_message_status: GNA2_STATUS_SUCCESS,
            status_message: String::from("Mock status message"),
            status_get_message_calls: 0,
        }
    }
}

thread_local! {
    static MOCK_STATE: RefCell<MockModelState> = RefCell::new(MockModelState::default());
}

fn free_tracked_heap_allocations() {
    let ptrs = MOCK_STATE.with(|s| std::mem::take(&mut s.borrow_mut().heap_allocated_ptrs));
    if !ptrs.is_empty() {
        let heap = unsafe { GetProcessHeap() };
        for addr in ptrs {
            unsafe {
                HeapFree(heap, 0, addr as *mut c_void);
            }
        }
    }
}

fn reset_mock_state() {
    free_tracked_heap_allocations();
    MOCK_STATE.with(|s| {
        *s.borrow_mut() = MockModelState::default();
    });
}

unsafe extern "C" fn mock_device_get_count(device_count: *mut u32) -> Gna2Status {
    if !device_count.is_null() {
        unsafe { *device_count = 1 };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_get_version(
    device_index: u32,
    device_version: *mut Gna2DeviceVersion,
) -> Gna2Status {
    if device_index != 0 {
        return -5;
    }
    if !device_version.is_null() {
        unsafe { *device_version = Gna2DeviceVersion::SOFTWARE_EMULATION };
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_open(_device_index: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_device_close(_device_index: u32) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_memory_alloc(
    _size_requested: u32,
    _size_granted: *mut u32,
    _memory_address: *mut *mut c_void,
) -> Gna2Status {
    -4
}

unsafe extern "C" fn mock_memory_free(_memory: *mut c_void) -> Gna2Status {
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_operation_init_fully_connected_affine(
    operation: *mut Gna2Operation,
    user_allocator: Gna2UserAllocator,
    inputs: *mut Gna2Tensor,
    outputs: *mut Gna2Tensor,
    weights: *mut Gna2Tensor,
    biases: *mut Gna2Tensor,
    activation: *mut Gna2Tensor,
) -> Gna2Status {
    let zero_ptr = unsafe { user_allocator(0) };
    let (call_num, configured_status, fail_on_call) = MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.fca_init_calls += 1;
        state.fca_allocator_zero_was_null = zero_ptr.is_null();
        (
            state.fca_init_calls,
            state.fca_init_status,
            state.fca_init_fail_on_call,
        )
    });

    if let Some(target_call) = fail_on_call {
        if call_num == target_call {
            return configured_status;
        }
    } else if configured_status != GNA2_STATUS_SUCCESS {
        return configured_status;
    }

    let alloc_bytes = (5 * std::mem::size_of::<*const Gna2Tensor>()) as u32;
    let mem = unsafe { user_allocator(alloc_bytes) } as *mut *const Gna2Tensor;
    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.fca_allocator_nonzero_was_nonnull = !mem.is_null();
        if !mem.is_null() {
            state.heap_allocated_ptrs.push(mem as usize);
        }
    });
    if mem.is_null() {
        return -8;
    }

    unsafe {
        *mem.add(0) = inputs as *const Gna2Tensor;
        *mem.add(1) = outputs as *const Gna2Tensor;
        *mem.add(2) = weights as *const Gna2Tensor;
        *mem.add(3) = biases as *const Gna2Tensor;
        *mem.add(4) = activation as *const Gna2Tensor;

        if !operation.is_null() {
            (*operation).operation_type = Gna2OperationType::FullyConnectedAffine;
            (*operation).operands = mem;
            (*operation).number_of_operands = 5;
            (*operation).parameters = ptr::null_mut();
            (*operation).number_of_parameters = 0;
        }
    }

    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_model_create(
    device_index: u32,
    model: *const Gna2Model,
    model_id: *mut u32,
) -> Gna2Status {
    let mut ptrs_to_free: Vec<usize> = Vec::new();

    let status = MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.model_create_calls += 1;
        state.last_created_device_index = Some(device_index);

        if !model.is_null() {
            let m = unsafe { &*model };
            state.last_num_operations = m.number_of_operations;
            for i in 0..m.number_of_operations as usize {
                let op = unsafe { &*m.operations.add(i) };
                state.observed_op_types.push(op.operation_type);
                state.observed_operand_counts.push(op.number_of_operands);
                if i == 0 && !op.operands.is_null() {
                    for j in 0..op.number_of_operands as usize {
                        let t_ptr = unsafe { *op.operands.add(j) };
                        if !t_ptr.is_null() {
                            let mode = unsafe { (*t_ptr).mode };
                            state.observed_first_layer_tensor_modes.push(mode);
                        }
                    }
                }
                if !op.operands.is_null() {
                    let addr = op.operands as usize;
                    if let Some(pos) = state.heap_allocated_ptrs.iter().position(|&p| p == addr) {
                        state.heap_allocated_ptrs.remove(pos);
                        ptrs_to_free.push(addr);
                    } else if state.free_fallback_fca_operands
                        && op.operation_type == Gna2OperationType::FullyConnectedAffine
                        && !state.raw_operand_ptrs.contains(&addr)
                    {
                        state.freed_fallback_fca_operands_calls += 1;
                        ptrs_to_free.push(addr);
                    }
                }
            }
        }

        if state.model_create_status == GNA2_STATUS_SUCCESS && !model_id.is_null() {
            unsafe { *model_id = state.next_model_id };
            state.next_model_id += 1;
        }
        state.model_create_status
    });

    if !ptrs_to_free.is_empty() {
        let heap = unsafe { GetProcessHeap() };
        for addr in ptrs_to_free {
            unsafe {
                HeapFree(heap, 0, addr as *mut c_void);
            }
        }
    }

    status
}

unsafe extern "C" fn mock_model_release(model_id: u32) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.model_release_calls += 1;
        state.released_model_ids.push(model_id);
    });
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_model_get_last_error(error: *mut Gna2ModelError) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let state = s.borrow();
        if state.get_last_error_status == GNA2_STATUS_SUCCESS && !error.is_null() {
            unsafe { *error = state.last_error_desc };
        }
        state.get_last_error_status
    })
}

unsafe extern "C" fn mock_model_error_get_message(
    _error: *const Gna2ModelError,
    message_buffer: *mut c_char,
    message_buffer_size: u32,
) -> Gna2Status {
    let (status, msg) = MOCK_STATE.with(|s| {
        let state = s.borrow();
        (state.error_get_message_status, state.error_message.clone())
    });
    if status != GNA2_STATUS_SUCCESS {
        return status;
    }
    if !message_buffer.is_null() && message_buffer_size > 0 {
        let bytes = msg.as_bytes();
        let copy_len = bytes.len().min((message_buffer_size - 1) as usize);
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), message_buffer as *mut u8, copy_len);
            *message_buffer.add(copy_len) = 0;
        }
    }
    GNA2_STATUS_SUCCESS
}

unsafe extern "C" fn mock_model_error_get_max_message_length() -> u32 {
    MOCK_STATE.with(|s| s.borrow().max_error_message_len)
}

unsafe extern "C" fn mock_status_get_message(
    _status: Gna2Status,
    buffer: *mut c_char,
    buffer_size: u32,
) -> Gna2Status {
    let (status, msg) = MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.status_get_message_calls += 1;
        (
            state.status_get_message_status,
            state.status_message.clone(),
        )
    });
    if status != GNA2_STATUS_SUCCESS {
        return status;
    }
    if !buffer.is_null() && buffer_size > 0 {
        let bytes = msg.as_bytes();
        let copy_len = bytes.len().min((buffer_size - 1) as usize);
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, copy_len);
            *buffer.add(copy_len) = 0;
        }
    }
    GNA2_STATUS_SUCCESS
}

fn full_mock_symbol_table() -> GnaSymbolTable {
    GnaSymbolTable {
        device_get_count: mock_device_get_count,
        device_get_version: mock_device_get_version,
        device_open: mock_device_open,
        device_create_for_export: None,
        device_close: mock_device_close,
        device_set_number_of_threads: None,
        get_library_version: None,
        memory_alloc: mock_memory_alloc,
        memory_alloc_for_device: None,
        memory_free: mock_memory_free,
        memory_set_tag: None,
        model_create: Some(mock_model_create),
        model_release: Some(mock_model_release),
        model_get_last_error: Some(mock_model_get_last_error),
        model_error_get_message: Some(mock_model_error_get_message),
        model_error_get_max_message_length: Some(mock_model_error_get_max_message_length),
        operation_init_fully_connected_affine: Some(mock_operation_init_fully_connected_affine),
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
        status_get_message: Some(mock_status_get_message),
        status_get_max_message_length: None,
    }
}

fn sample_tensors(
    in_buf: &mut [i16],
    out_buf: &mut [i32],
    w_buf: &mut [i16],
    b_buf: &mut [i32],
) -> (Gna2Tensor, Gna2Tensor, Gna2Tensor, Gna2Tensor) {
    (
        Gna2Tensor::d2(
            16,
            4,
            Gna2DataType::Int16,
            in_buf.as_mut_ptr() as *mut c_void,
        ),
        Gna2Tensor::d2(
            8,
            4,
            Gna2DataType::Int32,
            out_buf.as_mut_ptr() as *mut c_void,
        ),
        Gna2Tensor::d2(
            8,
            16,
            Gna2DataType::Int16,
            w_buf.as_mut_ptr() as *mut c_void,
        ),
        Gna2Tensor::d1(8, Gna2DataType::Int32, b_buf.as_mut_ptr() as *mut c_void),
    )
}

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

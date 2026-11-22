use std::cell::RefCell;
use std::ffi::c_void;

use nanai_gna_dll_load::types::{
    GNA2_STATUS_SUCCESS, Gna2ErrorType, Gna2Model, Gna2ModelError, Gna2ModelItem, Gna2Operation,
    Gna2OperationType, Gna2Status, Gna2Tensor, Gna2TensorMode, Gna2UserAllocator,
};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetProcessHeap() -> *mut c_void;
    fn HeapFree(hHeap: *mut c_void, dwFlags: u32, lpMem: *mut c_void) -> i32;
}

pub(super) struct MockModelState {
    pub model_create_status: Gna2Status,
    pub next_model_id: u32,
    pub model_create_calls: u32,
    pub last_created_device_index: Option<u32>,
    pub last_num_operations: u32,
    pub observed_op_types: Vec<Gna2OperationType>,
    pub observed_operand_counts: Vec<u32>,
    pub observed_first_layer_tensor_modes: Vec<Gna2TensorMode>,
    pub free_fallback_fca_operands: bool,
    pub freed_fallback_fca_operands_calls: u32,
    pub raw_operand_ptrs: Vec<usize>,
    pub heap_allocated_ptrs: Vec<usize>,

    pub model_release_calls: u32,
    pub released_model_ids: Vec<u32>,

    pub fca_init_status: Gna2Status,
    pub fca_init_fail_on_call: Option<u32>,
    pub fca_init_calls: u32,
    pub fca_allocator_zero_was_null: bool,
    pub fca_allocator_nonzero_was_nonnull: bool,

    pub get_last_error_status: Gna2Status,
    pub last_error_desc: Gna2ModelError,
    pub error_get_message_status: Gna2Status,
    pub error_message: String,
    pub max_error_message_len: u32,

    pub status_get_message_status: Gna2Status,
    pub status_message: String,
    pub status_get_message_calls: u32,
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
            last_error_desc: Gna2ModelError::new(
                Gna2ModelItem::new(),
                Gna2ErrorType::BELOW_RANGE,
                123,
            ),
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
    pub(super) static MOCK_STATE: RefCell<MockModelState> = RefCell::new(MockModelState::default());
}

pub(super) fn free_tracked_heap_allocations() {
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

pub(super) fn reset_mock_state() {
    free_tracked_heap_allocations();
    MOCK_STATE.with(|s| {
        *s.borrow_mut() = MockModelState::default();
    });
}

pub(super) unsafe extern "C" fn mock_operation_init_fully_connected_affine(
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
            *operation =
                Gna2Operation::new(Gna2OperationType::FullyConnectedAffine).with_operands(mem, 5);
        }
    }

    GNA2_STATUS_SUCCESS
}

pub(super) unsafe extern "C" fn mock_model_create(
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
            state.last_num_operations = m.number_of_operations();
            for i in 0..m.number_of_operations() as usize {
                let op = unsafe { &*m.operations().add(i) };
                state.observed_op_types.push(op.operation_type());
                state.observed_operand_counts.push(op.number_of_operands());
                if i == 0 && !op.operands().is_null() {
                    for j in 0..op.number_of_operands() as usize {
                        let t_ptr = unsafe { *op.operands().add(j) };
                        if !t_ptr.is_null() {
                            let mode = unsafe { (*t_ptr).mode() };
                            state.observed_first_layer_tensor_modes.push(mode);
                        }
                    }
                }
                if !op.operands().is_null() {
                    let addr = op.operands() as usize;
                    if let Some(pos) = state.heap_allocated_ptrs.iter().position(|&p| p == addr) {
                        state.heap_allocated_ptrs.remove(pos);
                        ptrs_to_free.push(addr);
                    } else if state.free_fallback_fca_operands
                        && op.operation_type() == Gna2OperationType::FullyConnectedAffine
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

pub(super) unsafe extern "C" fn mock_model_release(model_id: u32) -> Gna2Status {
    MOCK_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.model_release_calls += 1;
        state.released_model_ids.push(model_id);
    });
    GNA2_STATUS_SUCCESS
}

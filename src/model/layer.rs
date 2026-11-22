use std::ffi::c_void;
use std::ptr;

use crate::error::{GnaError, Result};
use crate::symbol_table::FnGna2OperationInitFullyConnectedAffine;
use crate::types::device::{GNA2_STATUS_SUCCESS, Gna2UserAllocator};
use crate::types::model::{Gna2Operation, Gna2OperationType, Gna2Tensor};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetProcessHeap() -> *mut c_void;
    fn HeapAlloc(hHeap: *mut c_void, dwFlags: u32, dwBytes: usize) -> *mut c_void;
}

/// Custom allocator matching C ABI `void* (*Gna2UserAllocator)(uint32_t size)`
pub(super) unsafe extern "C" fn rust_gna2_user_allocator(size: u32) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }
    unsafe {
        let heap = GetProcessHeap();
        // HEAP_ZERO_MEMORY = 0x00000008
        HeapAlloc(heap, 0x00000008, size as usize)
    }
}

pub(super) struct FullyConnectedAffineLayer {
    pub(super) input: Gna2Tensor,
    pub(super) output: Gna2Tensor,
    pub(super) weights: Gna2Tensor,
    pub(super) biases: Gna2Tensor,
    pub(super) activation: Gna2Tensor,
}

/// Layer operation definition for constructing a GNA data-flow model.
pub(super) enum LayerDefinition {
    FullyConnectedAffine(Box<FullyConnectedAffineLayer>),
    Generic {
        op_type: Gna2OperationType,
        tensors: Box<[Gna2Tensor]>,
    },
    Raw(Gna2Operation),
}

/// Allocate and populate the 5-operand pointer table for a fully connected affine layer.
pub(super) unsafe fn allocate_fca_operands(
    allocator: Gna2UserAllocator,
    fca: &FullyConnectedAffineLayer,
) -> Result<*mut *const Gna2Tensor> {
    let size = (5 * std::mem::size_of::<*const Gna2Tensor>()) as u32;
    let mem = unsafe { allocator(size) } as *mut *const Gna2Tensor;
    if mem.is_null() {
        return Err(GnaError::NullPointer);
    }
    unsafe {
        *mem.add(0) = &fca.input as *const Gna2Tensor;
        *mem.add(1) = &fca.output as *const Gna2Tensor;
        *mem.add(2) = &fca.weights as *const Gna2Tensor;
        *mem.add(3) = &fca.biases as *const Gna2Tensor;
        *mem.add(4) = &fca.activation as *const Gna2Tensor;
    }
    Ok(mem)
}

/// Initialize a fully connected affine operation using the library's helper symbol.
pub(super) fn init_fca_via_symbol(
    fca_init: FnGna2OperationInitFullyConnectedAffine,
    allocator: Gna2UserAllocator,
    fca: &mut FullyConnectedAffineLayer,
) -> Result<Gna2Operation> {
    let mut op = Gna2Operation::default();
    let status = unsafe {
        fca_init(
            &mut op,
            allocator,
            &mut fca.input as *mut Gna2Tensor,
            &mut fca.output as *mut Gna2Tensor,
            &mut fca.weights as *mut Gna2Tensor,
            &mut fca.biases as *mut Gna2Tensor,
            &mut fca.activation as *mut Gna2Tensor,
        )
    };
    if status != GNA2_STATUS_SUCCESS {
        return Err(GnaError::from_status(status));
    }
    Ok(op)
}

/// Manually initialize a fully connected affine operation when the helper symbol is absent.
pub(super) fn init_fca_manual(
    allocator: Gna2UserAllocator,
    fca: &FullyConnectedAffineLayer,
) -> Result<Gna2Operation> {
    // Allocate operands pointer table (5 operands: input, output, weights, biases, activation)
    let buf = unsafe { allocate_fca_operands(allocator, fca) }?;
    Ok(Gna2Operation::new(Gna2OperationType::FullyConnectedAffine).with_operands(buf, 5))
}

/// Initialize a generic GNA operation from a slice of stable-address tensors.
pub(super) fn init_generic_operation(
    allocator: Gna2UserAllocator,
    op_type: Gna2OperationType,
    tensors: &[Gna2Tensor],
) -> Result<Gna2Operation> {
    let count = tensors.len();
    let size = (count * std::mem::size_of::<*const Gna2Tensor>()) as u32;
    let mem = unsafe { allocator(size) } as *mut *const Gna2Tensor;
    if mem.is_null() {
        return Err(GnaError::NullPointer);
    }
    for (i, tensor) in tensors.iter().enumerate() {
        unsafe {
            *mem.add(i) = tensor as *const Gna2Tensor;
        }
    }
    Ok(Gna2Operation::new(op_type).with_operands(mem, count as u32))
}

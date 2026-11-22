mod builder;

pub use builder::*;

#[cfg(test)]
mod tests;

use std::ffi::{c_char, c_void};
use std::ptr;

use crate::error::{GnaError, Result};
use crate::loader::GnaLibrary;
use crate::symbol_table::FnGna2OperationInitFullyConnectedAffine;
use crate::types::{
    GNA2_STATUS_SUCCESS, Gna2ModelError, Gna2Operation, Gna2OperationType, Gna2Tensor,
    Gna2UserAllocator,
};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetProcessHeap() -> *mut c_void;
    fn HeapAlloc(hHeap: *mut c_void, dwFlags: u32, dwBytes: usize) -> *mut c_void;
}

/// Custom allocator matching C ABI `void* (*Gna2UserAllocator)(uint32_t size)`
unsafe extern "C" fn rust_gna2_user_allocator(size: u32) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }
    unsafe {
        let heap = GetProcessHeap();
        // HEAP_ZERO_MEMORY = 0x00000008
        HeapAlloc(heap, 0x00000008, size as usize)
    }
}

/// RAII wrapper for a compiled GNA Model handle (`model_id`).
///
/// Automatically invokes `Gna2ModelRelease` when dropped.
pub struct GnaModel {
    library: GnaLibrary,
    model_id: u32,
    device_index: u32,
}

impl GnaModel {
    /// Get the underlying `model_id`.
    pub fn id(&self) -> u32 {
        self.model_id
    }

    /// Get the device index this model is compiled for.
    pub fn device_index(&self) -> u32 {
        self.device_index
    }

    /// Access reference to parent GnaLibrary.
    pub fn library(&self) -> &GnaLibrary {
        &self.library
    }

    /// Query the last model error from the library if model creation failed.
    pub fn get_last_error_message(library: &GnaLibrary) -> Option<String> {
        let get_err = library.symbols().model_get_last_error?;
        let get_msg = library.symbols().model_error_get_message?;
        let max_len = library
            .symbols()
            .model_error_get_max_message_length
            .map(|f| unsafe { f() })
            .unwrap_or(1024)
            .max(256);

        let mut error_desc = unsafe { std::mem::zeroed::<Gna2ModelError>() };
        let status = unsafe { get_err(&mut error_desc) };
        if status != GNA2_STATUS_SUCCESS {
            return None;
        }

        println!(
            "    [DEBUG get_last_error_message] status={}, error_desc = {:#?}",
            status, error_desc
        );

        let mut buf = vec![0 as c_char; max_len as usize];
        let status = unsafe { get_msg(&error_desc, buf.as_mut_ptr(), max_len) };
        if status == GNA2_STATUS_SUCCESS {
            let c_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
            Some(c_str.to_string_lossy().into_owned())
        } else {
            Some(format!(
                "Model item error: type={:?}, reason={:?}, val={}",
                error_desc.source.item_type, error_desc.reason, error_desc.value
            ))
        }
    }
}

impl Drop for GnaModel {
    fn drop(&mut self) {
        if let Some(release_fn) = self.library.symbols().model_release {
            unsafe {
                let _ = release_fn(self.model_id);
            }
        }
    }
}

struct FullyConnectedAffineLayer {
    input: Gna2Tensor,
    output: Gna2Tensor,
    weights: Gna2Tensor,
    biases: Gna2Tensor,
    activation: Gna2Tensor,
}

/// Layer operation definition for constructing a GNA data-flow model.
enum LayerDefinition {
    FullyConnectedAffine(Box<FullyConnectedAffineLayer>),
    Raw(Gna2Operation),
}

/// Allocate and populate the 5-operand pointer table for a fully connected affine layer.
unsafe fn allocate_fca_operands(
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
fn init_fca_via_symbol(
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
fn init_fca_manual(
    allocator: Gna2UserAllocator,
    fca: &FullyConnectedAffineLayer,
) -> Result<Gna2Operation> {
    // Allocate operands pointer table (5 operands: input, output, weights, biases, activation)
    let buf = unsafe { allocate_fca_operands(allocator, fca) }?;
    Ok(Gna2Operation {
        operation_type: Gna2OperationType::FullyConnectedAffine,
        operands: buf,
        number_of_operands: 5,
        parameters: ptr::null_mut(),
        number_of_parameters: 0,
    })
}

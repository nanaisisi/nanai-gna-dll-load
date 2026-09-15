use std::ffi::{c_char, c_void};
use std::ptr;

use crate::device::GnaDevice;
use crate::error::{GnaError, Result};
use crate::loader::GnaLibrary;
use crate::types::{
    Gna2Model, Gna2ModelError, Gna2Operation, Gna2OperationType, Gna2Tensor,
    GNA2_STATUS_MODEL_ERROR_UNAVAILABLE, GNA2_STATUS_SUCCESS,
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
        if status != GNA2_STATUS_SUCCESS || status == GNA2_STATUS_MODEL_ERROR_UNAVAILABLE {
            return None;
        }

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

/// Layer operation definition for constructing a GNA data-flow model.
enum LayerDefinition {
    FullyConnectedAffine {
        input: Gna2Tensor,
        output: Gna2Tensor,
        weights: Gna2Tensor,
        biases: Gna2Tensor,
        activation: Gna2Tensor,
    },
    Raw(Gna2Operation),
}

/// Fluent builder for constructing and compiling a GNA Model on a given device.
pub struct GnaModelBuilder {
    layers: Vec<LayerDefinition>,
}

impl GnaModelBuilder {
    /// Create an empty model builder.
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Add a Fully Connected Affine layer (Dense / Matrix Multiplication + Bias + Activation).
    ///
    /// # Arguments
    /// - `inputs`: Input tensor (`[W x B]` or `[N x W]`)
    /// - `outputs`: Output tensor (`[H x B]`)
    /// - `weights`: Weights matrix (`[H x W]`)
    /// - `biases`: Biases vector (`[H]` or `[H x B]`)
    /// - `activation`: Optional activation function PWL tensor (pass `None` for disabled/identity)
    pub fn add_fully_connected_affine(
        mut self,
        inputs: Gna2Tensor,
        outputs: Gna2Tensor,
        weights: Gna2Tensor,
        biases: Gna2Tensor,
        activation: Option<Gna2Tensor>,
    ) -> Self {
        let act = activation.unwrap_or_else(Gna2Tensor::disabled);
        self.layers.push(LayerDefinition::FullyConnectedAffine {
            input: inputs,
            output: outputs,
            weights,
            biases,
            activation: act,
        });
        self
    }

    /// Add a custom pre-initialized Gna2Operation to the network.
    pub fn add_operation(mut self, operation: Gna2Operation) -> Self {
        self.layers.push(LayerDefinition::Raw(operation));
        self
    }

    /// Build, validate, and compile the model on the specified GNA device.
    pub fn build(mut self, device: &GnaDevice) -> Result<GnaModel> {
        let library = device.library();
        let model_create = library
            .symbols()
            .model_create
            .ok_or_else(|| GnaError::Other("Gna2ModelCreate is not supported by DLL".into()))?;

        if self.layers.is_empty() {
            return Err(GnaError::Other("Model must contain at least one layer".into()));
        }

        let mut operations: Vec<Gna2Operation> = Vec::with_capacity(self.layers.len());
        // Pointers passed into Gna2OperationInitFullyConnectedAffine are stored into op.operands!
        // We must ensure the Gna2Tensor structs themselves remain at stable memory addresses!
        for mut layer in self.layers.drain(..) {
            match &mut layer {
                LayerDefinition::FullyConnectedAffine {
                    input,
                    output,
                    weights,
                    biases,
                    activation,
                } => {
                    let mut op = Gna2Operation::default();
                    if let Some(fca_init) = library.symbols().operation_init_fully_connected_affine {
                        let status = unsafe {
                            fca_init(
                                &mut op,
                                rust_gna2_user_allocator,
                                &mut **input,
                                &mut **output,
                                &mut **weights,
                                &mut **biases,
                                &mut **activation,
                            )
                        };
                        if status != GNA2_STATUS_SUCCESS {
                            return Err(GnaError::from_status(status));
                        }
                    } else {
                        // Manual operation initialization if helper symbol not present
                        op.operation_type = Gna2OperationType::FullyConnectedAffine;
                        // Allocate operands pointer table (5 operands: input, output, weights, biases, activation)
                        let operands_buf = unsafe {
                            let mem = rust_gna2_user_allocator(
                                (5 * std::mem::size_of::<*const Gna2Tensor>()) as u32,
                            ) as *mut *const Gna2Tensor;
                            if mem.is_null() {
                                return Err(GnaError::NullPointer);
                            }
                            *mem.add(0) = input.as_ref() as *const Gna2Tensor;
                            *mem.add(1) = output.as_ref() as *const Gna2Tensor;
                            *mem.add(2) = weights.as_ref() as *const Gna2Tensor;
                            *mem.add(3) = biases.as_ref() as *const Gna2Tensor;
                            *mem.add(4) = activation.as_ref() as *const Gna2Tensor;
                            mem
                        };
                        op.operands = operands_buf;
                        op.number_of_operands = 5;
                        op.parameters = ptr::null_mut();
                        op.number_of_parameters = 0;
                    }
                    operations.push(op);
                }
                LayerDefinition::Raw(op) => {
                    operations.push(op);
                }
            }
        }

        let model_raw = Gna2Model {
            number_of_operations: operations.len() as u32,
            operations: operations.as_mut_ptr(),
        };

        println!("    [DEBUG] Preparing to call Gna2ModelCreate with {} operations", operations.len());
        println!("    [DEBUG] Operation[0] type: {:?}, operands count: {}, operands ptr: {:p}", 
            operations[0].operation_type, operations[0].number_of_operands, operations[0].operands);
        let mut model_id: u32 = u32::MAX;
        println!("    [DEBUG] Invoking model_create on device {}", device.index());
        let status = unsafe { model_create(device.index(), &model_raw, &mut model_id) };
        println!("    [DEBUG] model_create returned status {}", status);

        if status != GNA2_STATUS_SUCCESS {
            let detail = GnaModel::get_last_error_message(library).unwrap_or_else(|| {
                format!("Failed to compile model on device {}. Status: {}", device.index(), status)
            });
            return Err(GnaError::ModelCreationError { status, detail });
        }

        Ok(GnaModel {
            library: library.clone(),
            model_id,
            device_index: device.index(),
        })
    }
}

impl Default for GnaModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

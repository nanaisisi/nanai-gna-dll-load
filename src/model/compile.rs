use std::ffi::c_char;

use crate::device::GnaDevice;
use crate::error::{GnaError, Result};
use crate::types::{GNA2_STATUS_SUCCESS, Gna2Model, Gna2Operation, Gna2UserAllocator};

use super::{
    GnaModel, GnaModelBuilder, LayerDefinition, init_fca_manual, init_fca_via_symbol,
    rust_gna2_user_allocator,
};

impl GnaModelBuilder {
    /// Build, validate, and compile the model on the specified GNA device.
    pub fn build(self, device: &GnaDevice) -> Result<GnaModel> {
        self.build_with_allocator(device, rust_gna2_user_allocator)
    }

    pub(super) fn build_with_allocator(
        mut self,
        device: &GnaDevice,
        allocator: Gna2UserAllocator,
    ) -> Result<GnaModel> {
        let library = device.library();
        let model_create = library
            .symbols()
            .model_create
            .ok_or_else(|| GnaError::Other("Gna2ModelCreate is not supported by DLL".into()))?;

        if self.layers.is_empty() {
            return Err(GnaError::Other(
                "Model must contain at least one layer".into(),
            ));
        }

        let mut operations: Vec<Gna2Operation> = Vec::with_capacity(self.layers.len());
        // Pointers passed into Gna2OperationInitFullyConnectedAffine are stored into op.operands!
        // We must ensure the Gna2Tensor structs themselves remain at stable memory addresses!
        for layer in &mut self.layers {
            match layer {
                LayerDefinition::FullyConnectedAffine(fca) => {
                    let op_res = match library.symbols().operation_init_fully_connected_affine {
                        Some(fca_init) => init_fca_via_symbol(fca_init, allocator, fca),
                        None => init_fca_manual(allocator, fca),
                    };
                    operations.push(op_res?);
                }
                LayerDefinition::Raw(op) => {
                    operations.push(*op);
                }
            }
        }

        let model_raw = Gna2Model {
            number_of_operations: operations.len() as u32,
            operations: operations.as_mut_ptr(),
        };

        println!(
            "    [DEBUG] Preparing to call Gna2ModelCreate with {} operations",
            operations.len()
        );
        for (op_idx, op) in operations.iter().enumerate() {
            println!(
                "    [DEBUG] Op[{}] type: {:?}, operands count: {}, operands ptr: {:p}",
                op_idx, op.operation_type, op.number_of_operands, op.operands
            );
            for i in 0..op.number_of_operands as usize {
                let tensor_ptr = unsafe { *op.operands.add(i) };
                println!("      [DEBUG] operand[{}] ptr: {:p}", i, tensor_ptr);
                if !tensor_ptr.is_null() {
                    let t = unsafe { &*tensor_ptr };
                    println!(
                        "        dims: {}, [{}, {}, {}, ...], mode: {:?}, data_type: {:?}, data: {:p}",
                        t.shape.number_of_dimensions,
                        t.shape.dimensions[0],
                        t.shape.dimensions[1],
                        t.shape.dimensions[2],
                        t.mode,
                        t.data_type,
                        t.data
                    );
                }
            }
        }

        let mut model_id: u32 = u32::MAX;
        println!(
            "    [DEBUG] Invoking model_create on device {}",
            device.index()
        );
        let status = unsafe { model_create(device.index(), &model_raw, &mut model_id) };
        println!("    [DEBUG] model_create returned status {}", status);

        if status != GNA2_STATUS_SUCCESS {
            if let Some(status_msg_fn) = library.symbols().status_get_message {
                let mut sbuf = vec![0 as c_char; 256];
                let s_res = unsafe { status_msg_fn(status, sbuf.as_mut_ptr(), 256) };
                if s_res == GNA2_STATUS_SUCCESS {
                    let c_str = unsafe { std::ffi::CStr::from_ptr(sbuf.as_ptr()) };
                    println!(
                        "    [DEBUG] status_get_message: {}",
                        c_str.to_string_lossy()
                    );
                }
            }
            let detail = GnaModel::get_last_error_message(library).unwrap_or_else(|| {
                format!(
                    "Failed to compile model on device {}. Status: {}",
                    device.index(),
                    status
                )
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

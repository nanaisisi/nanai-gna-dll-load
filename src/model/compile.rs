use std::ffi::c_char;

use crate::device::GnaDevice;
use crate::error::{GnaError, Result};
use crate::types::device::{GNA2_STATUS_SUCCESS, Gna2UserAllocator};
use crate::types::model::{Gna2Model, Gna2Operation};

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
            .model_create()
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
                    let op_res = match library.symbols().operation_init_fully_connected_affine() {
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

        let model_raw = Gna2Model::new(operations.len() as u32, operations.as_mut_ptr());

        let mut model_id: u32 = u32::MAX;
        let status = unsafe { model_create(device.index(), &model_raw, &mut model_id) };

        if status != GNA2_STATUS_SUCCESS {
            let mut status_msg = None;
            if let Some(status_msg_fn) = library.symbols().status_get_message() {
                let max_len = library
                    .symbols()
                    .status_get_max_message_length()
                    .map(|f| unsafe { f() })
                    .unwrap_or(256)
                    .max(256);
                let mut sbuf = vec![0 as c_char; max_len as usize];
                let s_res = unsafe { status_msg_fn(status, sbuf.as_mut_ptr(), max_len) };
                if s_res == GNA2_STATUS_SUCCESS {
                    let last = sbuf.len() - 1;
                    sbuf[last] = 0;
                    let c_str = unsafe { std::ffi::CStr::from_ptr(sbuf.as_ptr()) };
                    status_msg = Some(c_str.to_string_lossy().into_owned());
                }
            }
            let detail =
                GnaModel::get_last_error_message(library).unwrap_or_else(|| match status_msg {
                    Some(msg) => format!(
                        "Failed to compile model on device {}. Status: {} ({})",
                        device.index(),
                        status,
                        msg
                    ),
                    None => format!(
                        "Failed to compile model on device {}. Status: {}",
                        device.index(),
                        status
                    ),
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

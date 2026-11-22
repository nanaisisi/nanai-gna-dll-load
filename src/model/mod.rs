mod builder;
mod compile;
mod layer;

pub use builder::*;
use layer::*;

#[cfg(test)]
mod tests;

use std::ffi::c_char;

use crate::loader::GnaLibrary;
use crate::types::device::GNA2_STATUS_SUCCESS;
use crate::types::error_types::Gna2ModelError;

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

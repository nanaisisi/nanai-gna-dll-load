use std::ffi::{c_char, c_void};
use std::ptr;

use crate::device::GnaDevice;
use crate::error::{GnaError, Result};
use crate::loader::GnaLibrary;
use crate::symbol_table::FnGna2OperationInitFullyConnectedAffine;
use crate::types::{
    GNA2_STATUS_SUCCESS, Gna2Model, Gna2ModelError, Gna2Operation, Gna2OperationType, Gna2Tensor,
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
        self.layers
            .push(LayerDefinition::FullyConnectedAffine(Box::new(
                FullyConnectedAffineLayer {
                    input: inputs,
                    output: outputs,
                    weights,
                    biases,
                    activation: act,
                },
            )));
        self
    }

    /// Add a custom pre-initialized Gna2Operation to the network.
    pub fn add_operation(mut self, operation: Gna2Operation) -> Self {
        self.layers.push(LayerDefinition::Raw(operation));
        self
    }

    /// Build, validate, and compile the model on the specified GNA device.
    pub fn build(self, device: &GnaDevice) -> Result<GnaModel> {
        self.build_with_allocator(device, rust_gna2_user_allocator)
    }

    fn build_with_allocator(
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

impl Default for GnaModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Gna2DeviceVersion, Gna2Status};

    unsafe extern "C" fn null_allocator(_size: u32) -> *mut c_void {
        ptr::null_mut()
    }

    unsafe extern "C" fn heap_fca_allocator(_size: u32) -> *mut c_void {
        unsafe {
            std::alloc::alloc(std::alloc::Layout::new::<[*const Gna2Tensor; 5]>()) as *mut c_void
        }
    }

    unsafe extern "C" fn mock_device_get_count(device_count: *mut u32) -> Gna2Status {
        unsafe { *device_count = 1 };
        GNA2_STATUS_SUCCESS
    }

    unsafe extern "C" fn mock_device_get_version(
        _device_index: u32,
        device_version: *mut Gna2DeviceVersion,
    ) -> Gna2Status {
        unsafe { *device_version = Gna2DeviceVersion::SOFTWARE_EMULATION };
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

    unsafe extern "C" fn mock_model_create(
        _device_index: u32,
        model: *const Gna2Model,
        model_id: *mut u32,
    ) -> Gna2Status {
        unsafe {
            let op = &*(*model).operations;
            std::alloc::dealloc(
                op.operands as *mut u8,
                std::alloc::Layout::new::<[*const Gna2Tensor; 5]>(),
            );
            *model_id = 1;
        }
        GNA2_STATUS_SUCCESS
    }

    fn mock_symbol_table() -> crate::symbol_table::GnaSymbolTable {
        crate::symbol_table::GnaSymbolTable {
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
            model_release: None,
            model_get_last_error: None,
            model_error_get_message: None,
            model_error_get_max_message_length: None,
            operation_init_fully_connected_affine: None,
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
            status_get_message: None,
            status_get_max_message_length: None,
        }
    }

    #[test]
    fn test_allocate_fca_operands_null_allocator_returns_err() {
        let fca = FullyConnectedAffineLayer {
            input: Gna2Tensor::disabled(),
            output: Gna2Tensor::disabled(),
            weights: Gna2Tensor::disabled(),
            biases: Gna2Tensor::disabled(),
            activation: Gna2Tensor::disabled(),
        };
        let err = unsafe { allocate_fca_operands(null_allocator, &fca) }.unwrap_err();
        assert_eq!(err.to_string(), GnaError::NullPointer.to_string());
    }

    #[test]
    fn test_build_with_null_allocator_returns_err() {
        let library = GnaLibrary::from_symbol_table(mock_symbol_table());
        let device = GnaDevice::open(&library, 0).expect("open device");
        let err = GnaModelBuilder::new()
            .add_fully_connected_affine(
                Gna2Tensor::disabled(),
                Gna2Tensor::disabled(),
                Gna2Tensor::disabled(),
                Gna2Tensor::disabled(),
                None,
            )
            .build_with_allocator(&device, null_allocator)
            .err()
            .expect("expected NullPointer error");
        assert_eq!(err.to_string(), GnaError::NullPointer.to_string());
        assert_eq!(
            unsafe { (library.symbols().memory_alloc)(0, ptr::null_mut(), ptr::null_mut()) },
            -4
        );
        assert_eq!(
            unsafe { (library.symbols().memory_free)(ptr::null_mut()) },
            GNA2_STATUS_SUCCESS
        );
    }

    #[test]
    fn test_build_with_non_null_allocator_succeeds() {
        let library = GnaLibrary::from_symbol_table(mock_symbol_table());
        let device = GnaDevice::open(&library, 0).expect("open device");
        let model = GnaModelBuilder::new()
            .add_fully_connected_affine(
                Gna2Tensor::disabled(),
                Gna2Tensor::disabled(),
                Gna2Tensor::disabled(),
                Gna2Tensor::disabled(),
                None,
            )
            .build_with_allocator(&device, heap_fca_allocator)
            .expect("build model");
        assert_eq!(model.id(), 1);
    }
}

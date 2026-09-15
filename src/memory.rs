use std::ffi::c_void;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::slice;

use crate::error::{GnaError, Result};
use crate::loader::GnaLibrary;
use crate::types::GNA2_STATUS_SUCCESS;

/// A safe, RAII-managed memory buffer allocated via GNA's memory allocator.
///
/// When dropped, `Gna2MemoryFree` is automatically invoked.
pub struct GnaBuffer {
    library: GnaLibrary,
    ptr: NonNull<u8>,
    size_granted: usize,
}

// Safety: GNA allocated memory buffer can be transferred across threads
unsafe impl Send for GnaBuffer {}
unsafe impl Sync for GnaBuffer {}

impl GnaBuffer {
    /// Allocate a new buffer of at least `size` bytes using the library's allocator.
    pub fn new(library: &GnaLibrary, size: usize) -> Result<Self> {
        if size == 0 {
            return Err(GnaError::InvalidBufferSize(0));
        }

        let mut granted: u32 = 0;
        let mut raw_ptr: *mut c_void = std::ptr::null_mut();

        let status = unsafe {
            (library.symbols().memory_alloc)(size as u32, &mut granted, &mut raw_ptr)
        };

        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }

        let ptr = NonNull::new(raw_ptr as *mut u8).ok_or(GnaError::NullPointer)?;

        Ok(Self {
            library: library.clone(),
            ptr,
            size_granted: granted as usize,
        })
    }

    /// Allocate a new buffer bound to a specific GNA device.
    pub fn new_for_device(library: &GnaLibrary, device_index: u32, size: usize) -> Result<Self> {
        if size == 0 {
            return Err(GnaError::InvalidBufferSize(0));
        }

        let alloc_for_device = library
            .symbols()
            .memory_alloc_for_device
            .ok_or_else(|| GnaError::Other("Gna2MemoryAllocForDevice not supported by DLL".into()))?;

        let mut granted: u32 = 0;
        let mut raw_ptr: *mut c_void = std::ptr::null_mut();

        println!("[DEBUG new_for_device] device_index={}, size={}", device_index, size);
        let status =
            unsafe { alloc_for_device(device_index, size as u32, &mut granted, &mut raw_ptr) };
        println!("[DEBUG new_for_device] status={}, granted={}, raw_ptr={:p}", status, granted, raw_ptr);

        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }

        let ptr = NonNull::new(raw_ptr as *mut u8).ok_or(GnaError::NullPointer)?;

        Ok(Self {
            library: library.clone(),
            ptr,
            size_granted: granted as usize,
        })
    }

    /// Set special designation tag for the buffer (e.g. for embedded model export).
    pub fn set_tag(&mut self, tag: u32) -> Result<()> {
        let tag_fn = library_symbol!(self.library, memory_set_tag)?;
        let status = unsafe { tag_fn(self.as_raw_ptr(), tag) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Get granted buffer size in bytes.
    pub fn len(&self) -> usize {
        self.size_granted
    }

    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.size_granted == 0
    }

    /// Get raw pointer as void ptr.
    pub fn as_raw_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr() as *mut c_void
    }

    /// Get slice view of the buffer.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.size_granted) }
    }

    /// Get mutable slice view of the buffer.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size_granted) }
    }
}

macro_rules! library_symbol {
    ($lib:expr, $sym:ident) => {
        $lib.symbols()
            .$sym
            .ok_or_else(|| GnaError::Other(concat!(stringify!($sym), " not supported").into()))
    };
}
use library_symbol;

impl Deref for GnaBuffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for GnaBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl Drop for GnaBuffer {
    fn drop(&mut self) {
        println!("    [DEBUG GnaBuffer::drop] Calling memory_free({:p})", self.as_raw_ptr());
        unsafe {
            let status = (self.library.symbols().memory_free)(self.as_raw_ptr());
            println!("    [DEBUG GnaBuffer::drop] memory_free returned status {}", status);
        }
    }
}

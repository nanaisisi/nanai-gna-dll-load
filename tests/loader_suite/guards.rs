use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub(super) static ENV_LOCK: Mutex<()> = Mutex::new(());

pub(super) struct EnvGuard {
    saved: Vec<(OsString, Option<OsString>)>,
}

impl EnvGuard {
    pub(super) fn new() -> Self {
        Self { saved: Vec::new() }
    }

    fn save_key(&mut self, key: &OsStr) {
        if !self.saved.iter().any(|(k, _)| k.as_os_str() == key) {
            self.saved.push((key.to_os_string(), env::var_os(key)));
        }
    }

    pub(super) fn set(&mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) {
        let key_ref = key.as_ref();
        self.save_key(key_ref);
        unsafe {
            env::set_var(key_ref, val);
        }
    }

    pub(super) fn remove(&mut self, key: impl AsRef<OsStr>) {
        let key_ref = key.as_ref();
        self.save_key(key_ref);
        unsafe {
            env::remove_var(key_ref);
        }
    }

    pub(super) fn strip_dll_from_path(&mut self, dll_name: &str) {
        self.save_key(OsStr::new("PATH"));
        if let Some(current_path) = env::var_os("PATH") {
            let filtered: Vec<PathBuf> = env::split_paths(&current_path)
                .filter(|dir| !dir.join(dll_name).exists())
                .collect();
            let joined = env::join_paths(filtered).expect("failed to join filtered PATH");
            unsafe {
                env::set_var("PATH", joined);
            }
        }
    }

    pub(super) fn prepend_to_path(&mut self, dir: &Path) {
        self.save_key(OsStr::new("PATH"));
        let mut dirs = vec![dir.to_path_buf()];
        if let Some(current_path) = env::var_os("PATH") {
            dirs.extend(env::split_paths(&current_path));
        }
        let joined = env::join_paths(dirs).expect("failed to join prepended PATH");
        unsafe {
            env::set_var("PATH", joined);
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, orig) in self.saved.iter().rev() {
            unsafe {
                match orig {
                    Some(v) => env::set_var(key, v),
                    None => env::remove_var(key),
                }
            }
        }
    }
}

pub(super) struct CwdGuard {
    original: PathBuf,
}

impl CwdGuard {
    pub(super) fn enter(dir: &Path) -> Self {
        let original = env::current_dir().expect("failed to capture current_dir");
        env::set_current_dir(dir).expect("failed to switch current_dir");
        Self { original }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

pub(super) const MINIMAL_MOCK_DLL_SRC: &str = r#"
use std::ffi::{c_char, c_void};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceGetCount(device_count: *mut u32) -> i32 {
    if !device_count.is_null() {
        unsafe { *device_count = 1 };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceGetVersion(_device_index: u32, device_version: *mut u32) -> i32 {
    if !device_version.is_null() {
        unsafe { *device_version = 0x30 };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceOpen(_device_index: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceClose(_device_index: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2MemoryAlloc(
    size_requested: u32,
    size_granted: *mut u32,
    memory_address: *mut *mut c_void,
) -> i32 {
    if !size_granted.is_null() {
        unsafe { *size_granted = size_requested };
    }
    if !memory_address.is_null() {
        unsafe { *memory_address = std::ptr::null_mut() };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2MemoryFree(_memory: *mut c_void) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2GetLibraryVersion(
    version_buffer: *mut c_char,
    version_buffer_size: u32,
) -> i32 {
    let ver = b"1.0.0-test\0";
    if version_buffer.is_null() || (version_buffer_size as usize) < ver.len() {
        return -6;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(ver.as_ptr(), version_buffer as *mut u8, ver.len());
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2ModelOverrideAlignment(_new_alignment: u32) -> i32 {
    0
}
"#;

pub(super) const MISSING_SYMBOL_DLL_SRC: &str = r#"
use std::ffi::c_void;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceGetCount(_device_count: *mut u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceGetVersion(_device_index: u32, _device_version: *mut u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceOpen(_device_index: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2DeviceClose(_device_index: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Gna2MemoryAlloc(
    _size_requested: u32,
    _size_granted: *mut u32,
    _memory_address: *mut *mut c_void,
) -> i32 {
    0
}
// Intentionally omit Gna2MemoryFree
"#;

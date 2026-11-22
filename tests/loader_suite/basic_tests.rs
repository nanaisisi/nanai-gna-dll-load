use std::ffi::c_char;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use tempfile::tempdir;

use nanai_gna_dll_load::{
    error::GnaError,
    loader::{GnaLibrary, GnaLibraryBuilder},
    types::{GNA2_STATUS_SUCCESS, Gna2Status},
};

use super::guards::*;
use super::helpers::*;

#[test]
fn test_default_dll_name() {
    let name = GnaLibrary::default_dll_name();
    #[cfg(target_os = "windows")]
    assert_eq!(name, "gna.dll");
    #[cfg(target_os = "macos")]
    assert_eq!(name, "libgna.dylib");
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    assert_eq!(name, "libgna.so");
}

#[test]
fn test_load_from_nonexistent_path() {
    let bad_path = PathBuf::from("nonexistent_path_to_gna.dll");
    let res = GnaLibrary::load_from_path(&bad_path);
    assert!(res.is_err());
    match res.unwrap_err() {
        GnaError::LibraryLoadError(_) => {}
        other => panic!("Expected LibraryLoadError, got: {:?}", other),
    }
}

#[test]
fn test_load_from_env_var_not_set() {
    let res = GnaLibrary::load_from_env("TOTALLY_NONEXISTENT_GNA_ENV_12345");
    assert!(res.is_err());
    match res.unwrap_err() {
        GnaError::EnvVarNotSet(var) => {
            assert_eq!(var, "TOTALLY_NONEXISTENT_GNA_ENV_12345");
        }
        other => panic!("Expected EnvVarNotSet, got: {:?}", other),
    }
}

#[test]
fn test_load_from_env_file_not_found() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut guard = EnvGuard::new();
    guard.set("TEST_GNA_NOT_FOUND_ENV", "C:\\some_invalid_path\\gna.dll");

    let res = GnaLibrary::load_from_env("TEST_GNA_NOT_FOUND_ENV");
    assert!(res.is_err());
    match res.unwrap_err() {
        GnaError::LibraryPathNotFound(p) => {
            assert!(p.to_string_lossy().contains("some_invalid_path"));
        }
        other => panic!("Expected LibraryPathNotFound, got: {:?}", other),
    }
}

#[test]
fn test_load_from_env_dir_appends_default_dll() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();

    let mut guard = EnvGuard::new();
    guard.set("TEST_GNA_DIR_ENV", &dir_path);

    let res = GnaLibrary::load_from_env("TEST_GNA_DIR_ENV");
    assert!(res.is_err());
    match res.unwrap_err() {
        GnaError::LibraryPathNotFound(target) => {
            assert_eq!(target, dir_path.join(GnaLibrary::default_dll_name()));
        }
        other => panic!("Expected LibraryPathNotFound, got: {:?}", other),
    }
}

#[test]
fn test_load_from_env_or_path_fallback() {
    let bad_fallback = PathBuf::from("fallback_invalid.dll");
    let res = GnaLibrary::load_from_env_or_path("TOTALLY_NONEXISTENT_GNA_ENV_9999", &bad_fallback);
    assert!(res.is_err());
    match res.unwrap_err() {
        GnaError::LibraryLoadError(_) => {}
        other => panic!("Expected LibraryLoadError, got: {:?}", other),
    }
}

#[test]
fn test_builder_search_failed_reporting() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut guard = EnvGuard::new();
    guard.remove("CUSTOM_ENV_FOR_TEST");

    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();

    // Create a dummy file that is not a valid DLL/PE image
    let dummy_file = dir_path.join("dummy_gna.dll");
    fs::write(&dummy_file, b"not a dll").unwrap();

    let res = GnaLibraryBuilder::new()
        .path(&dummy_file)
        .env("CUSTOM_ENV_FOR_TEST")
        .load();

    assert!(res.is_err());
    match res.unwrap_err() {
        GnaError::LibrarySearchFailed { tried, last_error } => {
            assert!(tried.contains(&dummy_file));
            assert!(last_error.is_some());
        }
        other => panic!("Expected LibrarySearchFailed, got: {:?}", other),
    }
}

#[test]
fn test_device_version_and_generations() {
    use nanai_gna_dll_load::types::{
        Gna2DeviceGeneration, Gna2DeviceVersion, Gna2InstrumentationMode, Gna2InstrumentationUnit,
        Gna2ModelExportComponent,
    };

    assert_eq!(Gna2DeviceVersion::SOFTWARE_EMULATION.raw(), 0);
    assert_eq!(Gna2DeviceVersion::EMBEDDED_1_0.raw(), 0x10E);
    assert_eq!(Gna2DeviceVersion::EMBEDDED_3_1.raw(), 0x310E);
    assert_eq!(Gna2DeviceVersion::EMBEDDED_3_1.as_str(), "GNA Embedded 3.1");

    assert_eq!(Gna2DeviceGeneration::Gen3_1 as u32, 0x310);
    assert_eq!(Gna2InstrumentationUnit::Cycles as u32, 2);
    assert_eq!(Gna2InstrumentationMode::Disabled as i32, -1);
    assert_eq!(
        Gna2ModelExportComponent::ExternalBufferOutputDump as u32,
        22
    );
}

#[test]
fn test_debug_clone_and_path_from_symbol_table() {
    let lib = GnaLibrary::from_symbol_table(base_mock_symbol_table());
    assert_eq!(lib.path(), Path::new("mock_gna.dll"));

    let dbg = format!("{:?}", lib);
    assert!(dbg.contains("GnaLibrary"));
    assert!(dbg.contains("mock_gna.dll"));

    let cloned = lib.clone();
    assert_eq!(cloned.path(), lib.path());
}

#[test]
fn test_get_library_version_none_err_and_ok() {
    // None branch
    let lib_none = GnaLibrary::from_symbol_table(base_mock_symbol_table());
    match lib_none.get_library_version().unwrap_err() {
        GnaError::Other(msg) => assert!(msg.contains("Gna2GetLibraryVersion")),
        other => panic!("Expected Other error, got: {:?}", other),
    }

    // Error status branch
    unsafe extern "C" fn version_err(_buf: *mut c_char, _len: u32) -> Gna2Status {
        -1
    }
    let table_err = base_mock_symbol_table().with_get_library_version(version_err);
    let lib_err = GnaLibrary::from_symbol_table(table_err);
    match lib_err.get_library_version().unwrap_err() {
        GnaError::StatusError { status, .. } => assert_eq!(status, -1),
        other => panic!("Expected StatusError, got: {:?}", other),
    }

    // Success branch
    unsafe extern "C" fn version_ok(buf: *mut c_char, len: u32) -> Gna2Status {
        let ver = b"2.1.3\0";
        if buf.is_null() || (len as usize) < ver.len() {
            return -6;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(ver.as_ptr(), buf as *mut u8, ver.len());
        }
        GNA2_STATUS_SUCCESS
    }
    let table_ok = base_mock_symbol_table().with_get_library_version(version_ok);
    let lib_ok = GnaLibrary::from_symbol_table(table_ok);
    assert_eq!(lib_ok.get_library_version().unwrap(), "2.1.3");
}

#[test]
fn test_override_alignment_none_err_and_ok() {
    // None branch
    let lib_none = GnaLibrary::from_symbol_table(base_mock_symbol_table());
    match lib_none.override_alignment(64).unwrap_err() {
        GnaError::Other(msg) => assert!(msg.contains("Gna2ModelOverrideAlignment")),
        other => panic!("Expected Other error, got: {:?}", other),
    }

    // Error status branch
    unsafe extern "C" fn align_err(_alignment: u32) -> Gna2Status {
        -5
    }
    let table_err = base_mock_symbol_table().with_model_override_alignment(align_err);
    let lib_err = GnaLibrary::from_symbol_table(table_err);
    match lib_err.override_alignment(64).unwrap_err() {
        GnaError::StatusError { status, .. } => assert_eq!(status, -5),
        other => panic!("Expected StatusError, got: {:?}", other),
    }

    // Success branch
    static LAST_ALIGNMENT: AtomicU32 = AtomicU32::new(0);
    unsafe extern "C" fn align_ok(alignment: u32) -> Gna2Status {
        LAST_ALIGNMENT.store(alignment, Ordering::SeqCst);
        GNA2_STATUS_SUCCESS
    }
    let table_ok = base_mock_symbol_table().with_model_override_alignment(align_ok);
    let lib_ok = GnaLibrary::from_symbol_table(table_ok);
    lib_ok.override_alignment(128).unwrap();
    assert_eq!(LAST_ALIGNMENT.load(Ordering::SeqCst), 128);
}

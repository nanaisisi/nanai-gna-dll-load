use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use nanai_gna_dll_load::{GnaError, GnaLibrary, GnaLibraryBuilder};

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
    unsafe {
        env::set_var("TEST_GNA_NOT_FOUND_ENV", "C:\\some_invalid_path\\gna.dll");
    }
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
    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();

    unsafe {
        env::set_var("TEST_GNA_DIR_ENV", &dir_path);
    }

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
    use nanai_gna_dll_load::{
        Gna2DeviceGeneration, Gna2DeviceVersion, Gna2InstrumentationMode, Gna2InstrumentationUnit,
        Gna2ModelExportComponent,
    };

    assert_eq!(Gna2DeviceVersion::SOFTWARE_EMULATION.0, 0);
    assert_eq!(Gna2DeviceVersion::EMBEDDED_1_0.0, 0x10E);
    assert_eq!(Gna2DeviceVersion::EMBEDDED_3_1.0, 0x310E);
    assert_eq!(Gna2DeviceVersion::EMBEDDED_3_1.as_str(), "GNA Embedded 3.1");

    assert_eq!(Gna2DeviceGeneration::Gen3_1 as u32, 0x310);
    assert_eq!(Gna2InstrumentationUnit::Cycles as u32, 2);
    assert_eq!(Gna2InstrumentationMode::Disabled as i32, -1);
    assert_eq!(
        Gna2ModelExportComponent::ExternalBufferOutputDump as u32,
        22
    );
}

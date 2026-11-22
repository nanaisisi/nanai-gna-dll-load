use std::fs;
use tempfile::tempdir;

use nanai_gna_dll_load::{error::GnaError, loader::GnaLibrary};

use super::guards::*;
use super::helpers::*;

#[test]
fn test_load_from_path_success_and_missing_symbol() {
    let dir = tempdir().unwrap();
    let incomplete_dll = compile_cdylib(
        dir.path(),
        "incomplete",
        "incomplete.dll",
        MISSING_SYMBOL_DLL_SRC,
    );
    let valid_dll = compile_cdylib(dir.path(), "valid", "valid_gna.dll", MINIMAL_MOCK_DLL_SRC);

    match GnaLibrary::load_from_path(&incomplete_dll).unwrap_err() {
        GnaError::SymbolNotFound { symbol, .. } => assert_eq!(symbol, "Gna2MemoryFree"),
        other => panic!("Expected SymbolNotFound, got: {:?}", other),
    }

    let lib = GnaLibrary::load_from_path(&valid_dll).unwrap();
    assert_eq!(lib.path(), valid_dll.as_path());
    assert_eq!(lib.get_library_version().unwrap(), "1.0.0-test");
    lib.override_alignment(64).unwrap();
    let dbg = format!("{:?}", lib);
    assert!(dbg.contains("valid_gna.dll"));

    // Drop loaded library before TempDir drops so Windows unlocks the DLL file.
    drop(lib);
}

#[test]
fn test_load_from_env_and_env_or_path_success() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let compiled_dll = compile_cdylib(dir.path(), "mock", "direct_gna.dll", MINIMAL_MOCK_DLL_SRC);
    let incomplete_dll = compile_cdylib(
        dir.path(),
        "env_incomplete",
        "env_incomplete.dll",
        MISSING_SYMBOL_DLL_SRC,
    );

    let sub_dir = dir.path().join("sub_dir");
    fs::create_dir(&sub_dir).unwrap();
    let dir_dll = sub_dir.join(GnaLibrary::default_dll_name());
    fs::copy(&compiled_dll, &dir_dll).unwrap();

    let bad_dir = dir.path().join("bad_dir");
    fs::create_dir(&bad_dir).unwrap();
    let bad_dll = bad_dir.join(GnaLibrary::default_dll_name());
    fs::write(&bad_dll, b"not a dll").unwrap();

    let mut guard = EnvGuard::new();

    // Direct DLL path in env var
    guard.set("TEST_LOAD_ENV_FILE", &compiled_dll);
    let lib = GnaLibrary::load_from_env("TEST_LOAD_ENV_FILE").unwrap();
    assert_eq!(lib.path(), compiled_dll.as_path());
    drop(lib);

    // Directory path in env var (appends gna.dll)
    guard.set("TEST_LOAD_ENV_DIR", &sub_dir);
    let lib = GnaLibrary::load_from_env("TEST_LOAD_ENV_DIR").unwrap();
    assert_eq!(lib.path(), dir_dll.as_path());
    drop(lib);

    // Existing invalid DLL in env var hits load_from_path error
    guard.set("TEST_LOAD_ENV_BAD", &bad_dll);
    match GnaLibrary::load_from_env("TEST_LOAD_ENV_BAD").unwrap_err() {
        GnaError::LibraryLoadError(_) => {}
        other => panic!("Expected LibraryLoadError, got: {:?}", other),
    }

    // Existing DLL missing required symbol in env var hits symbol table error via PathBuf instantiation
    guard.set("TEST_LOAD_ENV_INCOMPLETE", &incomplete_dll);
    match GnaLibrary::load_from_env("TEST_LOAD_ENV_INCOMPLETE").unwrap_err() {
        GnaError::SymbolNotFound { symbol, .. } => assert_eq!(symbol, "Gna2MemoryFree"),
        other => panic!("Expected SymbolNotFound, got: {:?}", other),
    }

    // load_from_env_or_path succeeds via env var for both (&str, &str) and (&str, &PathBuf)
    let lib = GnaLibrary::load_from_env_or_path("TEST_LOAD_ENV_FILE", "nonexistent_fallback.dll")
        .unwrap();
    assert_eq!(lib.path(), compiled_dll.as_path());
    drop(lib);

    let lib = GnaLibrary::load_from_env_or_path("TEST_LOAD_ENV_FILE", &compiled_dll).unwrap();
    assert_eq!(lib.path(), compiled_dll.as_path());
    drop(lib);

    // load_from_env_or_path falls back to valid fallback path when env is unset (&PathBuf and &str)
    let lib =
        GnaLibrary::load_from_env_or_path("TOTALLY_UNSET_ENV_FOR_OR_PATH", &compiled_dll).unwrap();
    assert_eq!(lib.path(), compiled_dll.as_path());
    drop(lib);

    let compiled_str = compiled_dll.to_str().unwrap();
    let lib =
        GnaLibrary::load_from_env_or_path("TOTALLY_UNSET_ENV_FOR_OR_PATH", compiled_str).unwrap();
    assert_eq!(lib.path(), compiled_dll.as_path());
    drop(lib);

    // load_from_env_or_path (&str, &str) exercises load_from_path::<&str> error branches
    let incomplete_str = incomplete_dll.to_str().unwrap();
    match GnaLibrary::load_from_env_or_path("TOTALLY_UNSET_ENV_FOR_OR_PATH", incomplete_str)
        .unwrap_err()
    {
        GnaError::SymbolNotFound { symbol, .. } => assert_eq!(symbol, "Gna2MemoryFree"),
        other => panic!("Expected SymbolNotFound, got: {:?}", other),
    }

    match GnaLibrary::load_from_env_or_path(
        "TOTALLY_UNSET_ENV_FOR_OR_PATH",
        "nonexistent_fallback.dll",
    )
    .unwrap_err()
    {
        GnaError::LibraryLoadError(_) => {}
        other => panic!("Expected LibraryLoadError, got: {:?}", other),
    }
}

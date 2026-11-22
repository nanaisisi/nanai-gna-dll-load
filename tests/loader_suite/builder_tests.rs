use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

use nanai_gna_dll_load::{
    error::GnaError,
    loader::{GnaLibrary, GnaLibraryBuilder},
};

use super::guards::*;
use super::helpers::*;

#[test]
fn test_builder_directory_and_env_candidates() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let compiled_dll = compile_cdylib(dir.path(), "mock", "mock.dll", MINIMAL_MOCK_DLL_SRC);

    let dir_empty = dir.path().join("dir_empty");
    let dir_bad = dir.path().join("dir_bad");
    let dir_ok = dir.path().join("dir_ok");
    let env_dir_empty = dir.path().join("env_dir_empty");
    let env_dir_bad = dir.path().join("env_dir_bad");
    let env_dir_ok = dir.path().join("env_dir_ok");
    for d in [
        &dir_empty,
        &dir_bad,
        &dir_ok,
        &env_dir_empty,
        &env_dir_bad,
        &env_dir_ok,
    ] {
        fs::create_dir(d).unwrap();
    }

    let default_name = GnaLibrary::default_dll_name();
    fs::write(dir_bad.join(default_name), b"corrupt dll").unwrap();
    fs::copy(&compiled_dll, dir_ok.join(default_name)).unwrap();
    fs::write(env_dir_bad.join(default_name), b"corrupt env dll").unwrap();
    fs::copy(&compiled_dll, env_dir_ok.join(default_name)).unwrap();

    let env_bad_file = dir.path().join("env_bad_file.dll");
    fs::write(&env_bad_file, b"corrupt file").unwrap();
    let env_missing_file = dir.path().join("does_not_exist.dll");

    let mut guard = EnvGuard::new();
    guard.remove("TEST_BUILDER_UNSET");
    guard.set("TEST_BUILDER_MISSING_FILE", &env_missing_file);
    guard.set("TEST_BUILDER_DIR_EMPTY", &env_dir_empty);
    guard.set("TEST_BUILDER_DIR_BAD", &env_dir_bad);
    guard.set("TEST_BUILDER_BAD_FILE", &env_bad_file);
    guard.set("TEST_BUILDER_OK_FILE", &compiled_dll);
    guard.set("TEST_BUILDER_OK_DIR", &env_dir_ok);

    // Builder defaults/debug/clone smoke check (empty and populated with DirectPath + EnvVar)
    let default_builder = GnaLibraryBuilder::default();
    let _cloned_builder = default_builder.clone();
    assert!(format!("{:?}", default_builder).contains("GnaLibraryBuilder"));

    let populated_builder = GnaLibraryBuilder::default()
        .path(&dir_empty)
        .env("TEST_BUILDER_OK_FILE")
        .system_fallback(true);
    let cloned_populated = populated_builder.clone();
    let dbg_populated = format!("{:?}", cloned_populated);
    assert!(dbg_populated.contains("DirectPath"));
    assert!(dbg_populated.contains("EnvVar"));
    assert!(dbg_populated.contains("system_fallback: true"));

    // All failing path & env candidates -> LibrarySearchFailed with tried paths and last_error
    let err = GnaLibrary::builder()
        .path(&dir_empty)
        .path(&dir_bad)
        .env("TEST_BUILDER_UNSET")
        .env("TEST_BUILDER_MISSING_FILE")
        .env("TEST_BUILDER_DIR_EMPTY")
        .env("TEST_BUILDER_DIR_BAD")
        .env("TEST_BUILDER_BAD_FILE")
        .load()
        .unwrap_err();
    match err {
        GnaError::LibrarySearchFailed { tried, last_error } => {
            assert!(tried.contains(&dir_empty.join(default_name)));
            assert!(tried.contains(&dir_bad.join(default_name)));
            assert!(tried.contains(&env_missing_file));
            assert!(tried.contains(&env_dir_empty.join(default_name)));
            assert!(tried.contains(&env_dir_bad.join(default_name)));
            assert!(tried.contains(&env_bad_file));
            assert!(last_error.is_some());
        }
        other => panic!("Expected LibrarySearchFailed, got: {:?}", other),
    }

    // Directory DirectPath candidate succeeds
    let lib = GnaLibrary::builder()
        .path(&dir_empty)
        .path(&dir_bad)
        .path(&dir_ok)
        .load()
        .unwrap();
    assert_eq!(lib.path(), dir_ok.join(default_name).as_path());
    drop(lib);

    // File EnvVar candidate succeeds
    let lib = GnaLibrary::builder()
        .env("TEST_BUILDER_UNSET")
        .env("TEST_BUILDER_BAD_FILE")
        .env("TEST_BUILDER_OK_FILE")
        .load()
        .unwrap();
    assert_eq!(lib.path(), compiled_dll.as_path());
    drop(lib);

    // Directory EnvVar candidate succeeds
    let lib = GnaLibrary::builder()
        .env("TEST_BUILDER_DIR_EMPTY")
        .env("TEST_BUILDER_OK_DIR")
        .load()
        .unwrap();
    assert_eq!(lib.path(), env_dir_ok.join(default_name).as_path());
    drop(lib);
}

#[test]
fn test_builder_current_dir_system_fallback_and_load_default() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let default_name = GnaLibrary::default_dll_name();

    // Compile mock DLL BEFORE altering PATH so rustc/linker can find toolchain tools.
    let compiled_dll = compile_cdylib(
        dir.path(),
        "mock",
        "compiled_mock.dll",
        MINIMAL_MOCK_DLL_SRC,
    );

    let empty_cwd = dir.path().join("empty_cwd");
    let cwd_with_dll = dir.path().join("cwd_with_dll");
    let path_with_dll = dir.path().join("path_with_dll");
    let env_with_dll = dir.path().join("env_with_dll");
    for d in [&empty_cwd, &cwd_with_dll, &path_with_dll, &env_with_dll] {
        fs::create_dir(d).unwrap();
    }
    fs::copy(&compiled_dll, cwd_with_dll.join(default_name)).unwrap();
    fs::copy(&compiled_dll, path_with_dll.join(default_name)).unwrap();
    fs::copy(&compiled_dll, env_with_dll.join(default_name)).unwrap();

    let mut guard = EnvGuard::new();
    guard.remove("GNA_LIB_PATH");
    guard.remove("GNA_LIB_DIR");
    guard.strip_dll_from_path(default_name);

    // 1. Clean environment + empty cwd -> default_envs + current_dir + system_fallback all fail
    {
        let _cwd = CwdGuard::enter(&empty_cwd);

        let err = GnaLibrary::builder()
            .default_envs()
            .current_dir()
            .system_fallback(true)
            .load()
            .unwrap_err();
        match err {
            GnaError::LibrarySearchFailed { tried, last_error } => {
                let expected = PathBuf::from(default_name);
                assert_eq!(tried, vec![expected.clone(), expected]);
                assert!(last_error.is_some());
            }
            other => panic!("Expected LibrarySearchFailed, got: {:?}", other),
        }

        let err_default = GnaLibrary::load_default().unwrap_err();
        assert!(matches!(err_default, GnaError::LibrarySearchFailed { .. }));
    }

    // 2. current_dir contains gna.dll -> builder().current_dir() and load_default() both succeed
    {
        let _cwd = CwdGuard::enter(&cwd_with_dll);

        let lib = GnaLibrary::builder().current_dir().load().unwrap();
        assert_eq!(lib.path(), Path::new(default_name));
        drop(lib);

        let lib = GnaLibrary::load_default().unwrap();
        assert_eq!(lib.path(), Path::new(default_name));
        drop(lib);
    }

    // 3. system_fallback(true) succeeds when gna.dll is discoverable via OS PATH
    {
        let _cwd = CwdGuard::enter(&empty_cwd);
        let mut path_guard = EnvGuard::new();
        path_guard.prepend_to_path(&path_with_dll);

        let lib = GnaLibrary::builder().system_fallback(true).load().unwrap();
        assert_eq!(lib.path(), Path::new(default_name));
        drop(lib);
    }

    // 4. load_default() succeeds via GNA_LIB_PATH and GNA_LIB_DIR
    {
        let _cwd = CwdGuard::enter(&empty_cwd);

        guard.set("GNA_LIB_PATH", &compiled_dll);
        let lib = GnaLibrary::load_default().unwrap();
        assert_eq!(lib.path(), compiled_dll.as_path());
        drop(lib);

        guard.remove("GNA_LIB_PATH");
        guard.set("GNA_LIB_DIR", &env_with_dll);
        let lib = GnaLibrary::load_default().unwrap();
        assert_eq!(lib.path(), env_with_dll.join(default_name).as_path());
        drop(lib);
    }
}

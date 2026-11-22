use std::env;
use std::ffi::{OsStr, OsString};
use std::sync::Mutex;

pub(super) static ENV_LOCK: Mutex<()> = Mutex::new(());

pub(super) struct EnvGuard {
    saved: Vec<(OsString, Option<OsString>)>,
}

impl EnvGuard {
    pub(super) fn new() -> Self {
        Self { saved: Vec::new() }
    }

    pub(super) fn set(&mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) {
        let key_ref = key.as_ref();
        if !self.saved.iter().any(|(k, _)| k.as_os_str() == key_ref) {
            self.saved
                .push((key_ref.to_os_string(), env::var_os(key_ref)));
        }
        unsafe {
            env::set_var(key_ref, val);
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

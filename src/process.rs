use crate::error::Result;

/// Information about a process using GNA (or a matching module).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GnaProcessInfo {
    /// Operating system process ID.
    pub pid: u32,
    /// Process executable name (e.g. `my_app.exe`).
    pub name: String,
    /// Full path or name of the matched GNA module(s) loaded in the process.
    pub matched_modules: Vec<String>,
}

/// Snapshot of system-wide GNA usage state, including external process contention
/// and optional OS performance counter readings.
#[derive(Debug, Clone, PartialEq)]
pub struct GnaSystemUtilization {
    /// Processes currently having a GNA runtime library loaded.
    pub active_gna_processes: Vec<GnaProcessInfo>,
    /// True if any process other than the current process has GNA loaded.
    pub has_external_processes: bool,
    /// Optional value read from an OS performance counter (e.g. Windows PDH).
    pub pdh_counter_value: Option<f64>,
}

/// Default module name substrings used to identify GNA runtime usage.
const DEFAULT_GNA_MODULE_PATTERNS: &[&str] =
    &["gna.dll", "gna_x64.dll", "libgna.so", "libgna.dylib"];

/// Detect running processes that have a GNA runtime library loaded.
///
/// On Windows, this enumerates running processes and their loaded modules via Toolhelp32 APIs,
/// skipping processes that cannot be inspected due to permissions.
/// On Linux, this inspects `/proc/<pid>/maps` and `/proc/<pid>/comm`.
pub fn detect_gna_processes() -> Result<Vec<GnaProcessInfo>> {
    detect_processes_by_module_patterns(DEFAULT_GNA_MODULE_PATTERNS)
}

/// Inspect system-wide GNA usage state: active GNA processes, external contention flag,
/// and an optional Windows PDH performance counter value.
pub fn inspect_system_gna_state(pdh_counter_path: Option<&str>) -> Result<GnaSystemUtilization> {
    let active_gna_processes = detect_gna_processes()?;
    let current_pid = std::process::id();
    let has_external_processes = active_gna_processes.iter().any(|p| p.pid != current_pid);
    let pdh_counter_value = match pdh_counter_path {
        Some(path) => query_pdh_counter_double(path)?,
        None => None,
    };
    Ok(GnaSystemUtilization {
        active_gna_processes,
        has_external_processes,
        pdh_counter_value,
    })
}

/// Returns `true` if no external process currently has a GNA runtime library loaded.
///
/// Intended as a lightweight check for upper-layer schedulers deciding whether to
/// dispatch work to GNA or route it to another backend (e.g. CPU).
pub fn is_gna_uncontended() -> Result<bool> {
    let current_pid = std::process::id();
    let active = detect_gna_processes()?;
    Ok(!active.iter().any(|p| p.pid != current_pid))
}

/// Query a Windows PDH performance counter as `f64` (returns `Ok(None)` if the counter
/// does not exist or on non-Windows platforms).
pub fn query_pdh_counter_double(counter_path: &str) -> Result<Option<f64>> {
    #[cfg(target_os = "windows")]
    {
        windows_pdh::query_double(counter_path)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = counter_path;
        Ok(None)
    }
}

/// Detect running processes that have any module whose name or path matches
/// one of the given case-insensitive patterns.
pub fn detect_processes_by_module_patterns(patterns: &[&str]) -> Result<Vec<GnaProcessInfo>> {
    let normalized: Vec<String> = patterns.iter().map(|p| p.to_ascii_lowercase()).collect();
    if normalized.is_empty() {
        return Ok(Vec::new());
    }

    #[cfg(target_os = "windows")]
    {
        windows_impl::detect_processes(&normalized)
    }

    #[cfg(target_os = "linux")]
    {
        return linux_impl::detect_processes(&normalized);
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = normalized;
        Ok(Vec::new())
    }
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::GnaProcessInfo;
    use crate::error::{GnaError, Result};
    use std::ffi::c_void;
    use std::mem::{size_of, zeroed};

    const TH32CS_SNAPPROCESS: u32 = 0x0000_0002;
    const TH32CS_SNAPMODULE: u32 = 0x0000_0008;
    const TH32CS_SNAPMODULE32: u32 = 0x0000_0010;
    const INVALID_HANDLE_VALUE: *mut c_void = -1isize as *mut c_void;

    #[repr(C)]
    struct ProcessEntry32W {
        dw_size: u32,
        cnt_usage: u32,
        th32_process_id: u32,
        th32_default_heap_id: usize,
        th32_module_id: u32,
        cnt_threads: u32,
        th32_parent_process_id: u32,
        pc_pri_class_base: i32,
        dw_flags: u32,
        sz_exe_file: [u16; 260],
    }

    #[repr(C)]
    struct ModuleEntry32W {
        dw_size: u32,
        th32_module_id: u32,
        th32_process_id: u32,
        glblcnt_usage: u32,
        proccnt_usage: u32,
        mod_base_addr: *mut u8,
        mod_base_size: u32,
        h_module: *mut c_void,
        sz_module: [u16; 256],
        sz_exe_path: [u16; 260],
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateToolhelp32Snapshot(dw_flags: u32, th32_process_id: u32) -> *mut c_void;
        fn Process32FirstW(h_snapshot: *mut c_void, lppe: *mut ProcessEntry32W) -> i32;
        fn Process32NextW(h_snapshot: *mut c_void, lppe: *mut ProcessEntry32W) -> i32;
        fn Module32FirstW(h_snapshot: *mut c_void, lpme: *mut ModuleEntry32W) -> i32;
        fn Module32NextW(h_snapshot: *mut c_void, lpme: *mut ModuleEntry32W) -> i32;
        fn CloseHandle(h_object: *mut c_void) -> i32;
    }

    struct HandleGuard(*mut c_void);

    impl Drop for HandleGuard {
        fn drop(&mut self) {
            if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
                unsafe {
                    CloseHandle(self.0);
                }
            }
        }
    }

    fn utf16_to_string(buf: &[u16]) -> String {
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..len])
    }

    pub fn detect_processes(patterns: &[String]) -> Result<Vec<GnaProcessInfo>> {
        let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snap == INVALID_HANDLE_VALUE || snap.is_null() {
            return Err(GnaError::Other(
                "Failed to create process snapshot via CreateToolhelp32Snapshot".into(),
            ));
        }
        let _snap_guard = HandleGuard(snap);

        let mut pe: ProcessEntry32W = unsafe { zeroed() };
        pe.dw_size = size_of::<ProcessEntry32W>() as u32;

        if unsafe { Process32FirstW(snap, &mut pe) } == 0 {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        loop {
            let pid = pe.th32_process_id;
            if pid != 0 {
                let matched = find_matching_modules(pid, patterns);
                if !matched.is_empty() {
                    results.push(GnaProcessInfo {
                        pid,
                        name: utf16_to_string(&pe.sz_exe_file),
                        matched_modules: matched,
                    });
                }
            }

            if unsafe { Process32NextW(snap, &mut pe) } == 0 {
                break;
            }
        }

        Ok(results)
    }

    fn find_matching_modules(pid: u32, patterns: &[String]) -> Vec<String> {
        let mod_snap =
            unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) };
        if mod_snap == INVALID_HANDLE_VALUE || mod_snap.is_null() {
            return Vec::new();
        }
        let _mod_guard = HandleGuard(mod_snap);

        let mut me: ModuleEntry32W = unsafe { zeroed() };
        me.dw_size = size_of::<ModuleEntry32W>() as u32;

        if unsafe { Module32FirstW(mod_snap, &mut me) } == 0 {
            return Vec::new();
        }

        let mut matched = Vec::new();
        loop {
            let mod_name = utf16_to_string(&me.sz_module);
            let mod_path = utf16_to_string(&me.sz_exe_path);
            let mod_name_lower = mod_name.to_ascii_lowercase();
            let mod_path_lower = mod_path.to_ascii_lowercase();

            if patterns
                .iter()
                .any(|p| mod_name_lower.contains(p) || mod_path_lower.contains(p))
            {
                let entry = if mod_path.is_empty() {
                    mod_name
                } else {
                    mod_path
                };
                if !matched.contains(&entry) {
                    matched.push(entry);
                }
            }

            if unsafe { Module32NextW(mod_snap, &mut me) } == 0 {
                break;
            }
        }

        matched
    }
}

#[cfg(target_os = "windows")]
mod windows_pdh {
    use crate::error::Result;
    use std::ffi::c_void;
    use std::ptr::null_mut;

    const PDH_FMT_DOUBLE: u32 = 0x0000_0200;

    #[repr(C)]
    struct PdhFmtCountervalueDouble {
        c_status: u32,
        double_value: f64,
    }

    #[link(name = "pdh")]
    unsafe extern "system" {
        fn PdhOpenQueryW(
            sz_data_source: *const u16,
            dw_user_data: usize,
            ph_query: *mut *mut c_void,
        ) -> i32;
        fn PdhAddEnglishCounterW(
            h_query: *mut c_void,
            sz_full_counter_path: *const u16,
            dw_user_data: usize,
            ph_counter: *mut *mut c_void,
        ) -> i32;
        fn PdhCollectQueryData(h_query: *mut c_void) -> i32;
        fn PdhGetFormattedCounterValue(
            h_counter: *mut c_void,
            dw_format: u32,
            lpdw_type: *mut u32,
            p_value: *mut PdhFmtCountervalueDouble,
        ) -> i32;
        fn PdhCloseQuery(h_query: *mut c_void) -> i32;
    }

    struct QueryGuard(*mut c_void);

    impl Drop for QueryGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    PdhCloseQuery(self.0);
                }
            }
        }
    }

    pub fn query_double(counter_path: &str) -> Result<Option<f64>> {
        let mut query: *mut c_void = null_mut();
        if unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } != 0 || query.is_null() {
            return Ok(None);
        }
        let _guard = QueryGuard(query);

        let wide_path: Vec<u16> = counter_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut counter: *mut c_void = null_mut();
        if unsafe { PdhAddEnglishCounterW(query, wide_path.as_ptr(), 0, &mut counter) } != 0
            || counter.is_null()
        {
            return Ok(None);
        }

        if unsafe { PdhCollectQueryData(query) } != 0 {
            return Ok(None);
        }

        let mut value = PdhFmtCountervalueDouble {
            c_status: 0,
            double_value: 0.0,
        };
        let mut counter_type: u32 = 0;
        if unsafe {
            PdhGetFormattedCounterValue(counter, PDH_FMT_DOUBLE, &mut counter_type, &mut value)
        } != 0
        {
            return Ok(None);
        }

        Ok(Some(value.double_value))
    }
}

#[cfg(target_os = "linux")]
mod linux_impl {
    use super::GnaProcessInfo;
    use crate::error::Result;
    use std::fs;

    pub fn detect_processes(patterns: &[String]) -> Result<Vec<GnaProcessInfo>> {
        let mut results = Vec::new();
        let entries = match fs::read_dir("/proc") {
            Ok(e) => e,
            Err(_) => return Ok(results),
        };

        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Ok(pid) = file_name.to_string_lossy().parse::<u32>() else {
                continue;
            };

            let maps_path = entry.path().join("maps");
            let Ok(maps) = fs::read_to_string(maps_path) else {
                continue;
            };

            let mut matched = Vec::new();
            for line in maps.lines() {
                if let Some(path) = line.split_whitespace().nth(5) {
                    let path_lower = path.to_ascii_lowercase();
                    if patterns.iter().any(|p| path_lower.contains(p))
                        && !matched.contains(&path.to_string())
                    {
                        matched.push(path.to_string());
                    }
                }
            }

            if !matched.is_empty() {
                let name = fs::read_to_string(entry.path().join("comm"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                results.push(GnaProcessInfo {
                    pid,
                    name,
                    matched_modules: matched,
                });
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_gna_processes_does_not_fail() {
        let res = detect_gna_processes();
        assert!(res.is_ok());
    }

    #[test]
    fn test_is_gna_uncontended_does_not_fail() {
        let res = is_gna_uncontended();
        assert!(res.is_ok());
    }

    #[test]
    fn test_detect_processes_empty_patterns() {
        let res = detect_processes_by_module_patterns(&[]).unwrap();
        assert!(res.is_empty());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_detect_current_process_with_kernel32() {
        let current_pid = std::process::id();
        let procs = detect_processes_by_module_patterns(&["kernel32.dll"]).unwrap();
        assert!(
            procs.iter().any(|p| p.pid == current_pid),
            "Current process ({current_pid}) should have kernel32.dll loaded"
        );
    }
}

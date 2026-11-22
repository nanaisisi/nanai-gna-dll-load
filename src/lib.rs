pub mod device;
pub mod error;
pub mod inference;
pub mod instrumentation;
pub mod loader;
pub mod memory;
pub mod metrics;
pub mod model;
pub mod model_export;
pub mod process;
pub mod stress;
pub mod symbol_table;
pub mod types;

// Re-export core items at root for easy access.
pub use device::GnaDevice;
pub use error::{GnaError, Result};
pub use inference::GnaRequestConfig;
pub use instrumentation::{
    GnaDeviceProfile, GnaInstrumentationConfig, GnaPerformanceStats, GnaUsageMonitor,
};
pub use loader::{GnaLibrary, GnaLibraryBuilder};
pub use memory::{GnaBuffer, gna_allocated_bytes};
pub use model::{GnaModel, GnaModelBuilder};
pub use model_export::GnaModelExportConfig;
pub use process::{
    GnaProcessInfo, GnaSystemUtilization, detect_gna_processes,
    detect_processes_by_module_patterns, inspect_system_gna_state, is_gna_uncontended,
    query_pdh_counter_double,
};
pub use stress::{GnaLoadTestConfig, GnaLoadTestReport, GnaLoadTester};
pub use types::*;

// Add new monitoring API components
pub use crate::metrics::metric_store::MetricStore;
pub use crate::metrics::monitoring_api::MonitoringApi;

pub mod device;
pub mod types;
pub mod stress;
pub mod error;
pub mod inference;
pub mod instrumentation;
pub mod loader;
pub mod memory;
pub mod model;
pub mod model_export;
pub mod metrics;
pub mod symbol_table;

// Re-export core items at root
pub use device::GnaDevice;
pub use error::{GnaError, Result};
pub use inference::GnaRequestConfig;
pub use instrumentation::{GnaInstrumentationConfig, GnaPerformanceStats, GnaUsageMonitor};
pub use loader::{GnaLibrary, GnaLibraryBuilder};
pub use memory::GnaBuffer;
pub use model::{GnaModel, GnaModelBuilder};
pub use model_export::GnaModelExportConfig;
pub use stress::{GnaLoadTestConfig, GnaLoadTestReport, GnaLoadTester};
pub use types::{
    Gna2AccelerationMode, Gna2BiasMode, Gna2DataType, Gna2DeviceGeneration, Gna2DeviceVersion,
    Gna2ErrorType, Gna2InstrumentationMode, Gna2InstrumentationPoint, Gna2InstrumentationUnit,
    Gna2ItemType, Gna2MemoryTag, Gna2Model, Gna2ModelError, Gna2ModelExportComponent,
    Gna2ModelItem, Gna2Operation, Gna2OperationType, Gna2Shape, Gna2Status, Gna2Tensor,
    Gna2TensorMode, Gna2UserAllocator, 
        GNA2_STATUS_MODEL_ERROR_UNAVAILABLE, GNA2_STATUS_SUCCESS,
        GNA2_STATUS_WARNING_ARITHMETIC_SATURATION, GNA2_STATUS_WARNING_DEVICE_BUSY,
    };

    // Add new monitoring API components
    pub use metrics::{MonitoringApi, MetricStore};

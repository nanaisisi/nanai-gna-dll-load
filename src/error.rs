use thiserror::Error;

pub type Result<T> = std::result::Result<T, GnaError>;

#[derive(Error, Debug)]
pub enum GnaError {
    #[error("Failed to load dynamic library: {0}")]
    LibraryLoadError(#[from] libloading::Error),

    #[error("Failed to load required symbol '{symbol}': {source}")]
    SymbolNotFound {
        symbol: &'static str,
        #[source]
        source: libloading::Error,
    },

    #[error("GNA operation failed with status code {status}: {message}")]
    StatusError { status: i32, message: String },

    #[error("Device index {0} out of range (total devices: {1})")]
    DeviceIndexOutOfRange(u32, u32),

    #[error("No GNA devices available")]
    NoDevicesAvailable,

    #[error("Null pointer encountered unexpectedly")]
    NullPointer,

    #[error("Invalid buffer size requested: {0}")]
    InvalidBufferSize(usize),

    #[error("Environment variable '{0}' is not set")]
    EnvVarNotSet(String),

    #[error("Library file not found at path: {0}")]
    LibraryPathNotFound(std::path::PathBuf),

    #[error(
        "Could not find or load GNA library. Locations tried: {tried:?}. Last error: {last_error:?}"
    )]
    LibrarySearchFailed {
        tried: Vec<std::path::PathBuf>,
        last_error: Option<String>,
    },

    #[error("GNA model creation failed (status: {status}): {detail}")]
    ModelCreationError { status: i32, detail: String },

    #[error("Operation error: {0}")]
    Other(String),
}

impl GnaError {
    pub fn from_status(status: i32) -> Self {
        let msg = match status {
            0 => "Success",
            1 => "Warning: Device busy",
            2 => "Warning: Arithmetic saturation",
            3 => "Warning: Model error unavailable",
            -1 => "General error",
            -3 => "Unknown error",
            -4 => "Functionality not implemented yet",
            -5 => "Item identifier is invalid",
            -6 => "NULL argument is not allowed",
            -7 => "NULL argument is required",
            -8 => "Unable to create new resources",
            -9 => "Device not available",
            -10 => "Device failed to open, thread count is invalid",
            -11 => "Device version is invalid",
            -12 => "Queue can not create or enqueue more requests",
            -13 => "Failed to receive communication from the device driver",
            -14 => "Failed to send communication to the device driver",
            -15 => "Hardware device parameter out of range",
            -16 => "Hardware device virtual address out of range",
            -17 => "Hardware device unexpected completion during PCIe operation",
            -18 => "Hardware device DMA error during PCIe operation",
            -19 => "Hardware device MMU error during PCIe operation",
            -20 => "Hardware device breakpoint hit",
            -21 => "Critical hardware device error occurred, device has been reset",
            _ => "Unrecognized GNA status code",
        };
        GnaError::StatusError {
            status,
            message: msg.to_string(),
        }
    }
}

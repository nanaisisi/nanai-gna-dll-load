#[path = "gna_demo_support/cli.rs"]
mod cli;
#[path = "gna_demo_support/runner.rs"]
mod runner;

use nanai_gna_dll_load::device::GnaDevice;

fn main() {
    println!("==================================================");
    println!("  nanai-gna-dll-rs: GNA Dynamic DLL Loader Demo   ");
    println!("==================================================");

    let Some(options) = cli::parse_args() else {
        return;
    };
    let Some(library) = cli::load_library(&options) else {
        return;
    };

    match GnaDevice::get_count(&library) {
        Ok(count) => {
            println!("Available GNA devices: {count}");
            for index in 0..count {
                match GnaDevice::get_version(&library, index) {
                    Ok(version) => println!(
                        "  Device #{index}: 0x{:02x} ({})",
                        version.0,
                        version.as_str()
                    ),
                    Err(error) => eprintln!("  Device #{index}: failed to query version: {error}"),
                }
            }
            if count > 0 {
                match GnaDevice::open(&library, 0) {
                    Ok(device) => {
                        println!("Opening device #0 ({})...", device.version().as_str());
                        runner::run_model_demo(&device, &options);
                    }
                    Err(error) => eprintln!("Failed to open device #0: {error}"),
                }
            }
        }
        Err(error) => eprintln!("Failed to query device count: {error}"),
    }
    println!("\nCompleted demo successfully.");
}

use std::env;
use std::path::PathBuf;
use std::time::Duration;

use nanai_gna_dll_load::{
    Gna2AccelerationMode, Gna2DataType, Gna2Tensor, GnaDevice, GnaLibrary, GnaLoadTestConfig,
    GnaLoadTester, GnaModelBuilder, GnaRequestConfig,
};

fn print_usage(program: &str) {
    println!("Usage: {program} [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --dll <PATH>               Load DLL directly from a file or directory");
    println!("  --env <VAR_NAME>           Load DLL from an environment variable");
    println!("  --stress [ITERATIONS]      Run stress test (default: 1000 iterations)");
    println!("  --duration <SECS>          Run stress test for a duration");
    println!("  --concurrency <N>          Stress-test queue depth (default: 1)");
    println!("  --help, -h                 Show this help message");
    println!();
    println!("Without options, GNA_LIB_PATH, GNA_LIB_DIR, the current directory, and");
    println!("the system library search path are checked in that order.");
}

fn main() {
    println!("==================================================");
    println!("  nanai-gna-dll-rs: GNA Dynamic DLL Loader Demo   ");
    println!("==================================================");

    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "gna_demo".to_owned());
    let mut dll_path = None;
    let mut env_var = None;
    let mut stress = false;
    let mut iterations = None;
    let mut duration = None;
    let mut concurrency = 1;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dll" => match args.next() {
                Some(value) => dll_path = Some(PathBuf::from(value)),
                None => return eprintln!("Error: --dll requires a path argument."),
            },
            "--env" => match args.next() {
                Some(value) => env_var = Some(value),
                None => return eprintln!("Error: --env requires an environment variable name."),
            },
            "--stress" => {
                stress = true;
                iterations = args
                    .next()
                    .and_then(|value| value.parse().ok())
                    .or(Some(1000));
            }
            "--duration" => {
                stress = true;
                match args.next().and_then(|value| value.parse().ok()) {
                    Some(value) => duration = Some(value),
                    None => return eprintln!("Error: --duration requires integer seconds."),
                }
            }
            "--concurrency" => match args.next().and_then(|value| value.parse::<usize>().ok()) {
                Some(value) => concurrency = value.max(1),
                None => return eprintln!("Error: --concurrency requires a positive integer."),
            },
            "--help" | "-h" => return print_usage(&program),
            option if option.starts_with('-') => {
                eprintln!("Unknown option: {option}");
                print_usage(&program);
                return;
            }
            value => dll_path = Some(PathBuf::from(value)),
        }
    }

    let library = match (dll_path, env_var) {
        (Some(path), _) => {
            println!("Attempting to load DLL from: {}", path.display());
            GnaLibrary::load_from_path(path)
        }
        (None, Some(variable)) => {
            println!("Attempting to load DLL from environment variable: {variable}");
            GnaLibrary::load_from_env(&variable)
        }
        (None, None) => {
            println!(
                "Attempting to load default DLL ({})...",
                GnaLibrary::default_dll_name()
            );
            GnaLibrary::load_default()
        }
    };

    let library = match library {
        Ok(library) => {
            println!("Successfully loaded DLL: {}", library.path().display());
            library
        }
        Err(error) => {
            eprintln!("\nDLL could not be loaded: {error}");
            eprintln!(
                "Hint: cargo run --example gna_demo -- --dll path/to/{}",
                GnaLibrary::default_dll_name()
            );
            return;
        }
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
                        run_model_demo(&device, stress, iterations, duration, concurrency);
                    }
                    Err(error) => eprintln!("Failed to open device #0: {error}"),
                }
            }
        }
        Err(error) => eprintln!("Failed to query device count: {error}"),
    }
    println!("\nCompleted demo successfully.");
}

fn run_model_demo(
    device: &GnaDevice,
    stress: bool,
    iterations: Option<usize>,
    duration: Option<u64>,
    concurrency: usize,
) {
    const W: usize = 64;
    const H: usize = 64;
    const B: usize = 1;

    let weights = vec![1_i16; W * H];
    let inputs = [1_i16; W * B];
    let biases = [0_i32; H];
    println!("[TRACE] 1. Allocating buffer...");
    let memory = match device.allocate_buffer(64 * 1024) {
        Ok(memory) => memory,
        Err(error) => return eprintln!("Failed to allocate model memory: {error}"),
    };
    println!("[TRACE] 2. Buffer allocated at {:p}, size {}", memory.as_raw_ptr(), memory.len());

    let base = memory.as_raw_ptr() as *mut u8;
    let inputs_ptr = base as *mut i16;
    let outputs_ptr = unsafe { base.add(4096) } as *mut i32;
    let weights_ptr = unsafe { base.add(8192) } as *mut i16;
    let biases_ptr = unsafe { base.add(16384) } as *mut i32;

    println!("[TRACE] 3. Copying inputs, weights, biases into buffer...");
    unsafe {
        std::ptr::copy_nonoverlapping(inputs.as_ptr(), inputs_ptr, inputs.len());
        std::ptr::copy_nonoverlapping(weights.as_ptr(), weights_ptr, weights.len());
        std::ptr::copy_nonoverlapping(biases.as_ptr(), biases_ptr, biases.len());
        std::ptr::write_bytes(outputs_ptr, 0, H * B);
    }
    println!("[TRACE] 4. Buffer filled. Building model...");

    let model = match GnaModelBuilder::new()
        .add_fully_connected_affine(
            Gna2Tensor::d2(W as u32, B as u32, Gna2DataType::Int16, inputs_ptr as _),
            Gna2Tensor::d2(H as u32, B as u32, Gna2DataType::Int32, outputs_ptr as _),
            Gna2Tensor::d2(H as u32, W as u32, Gna2DataType::Int16, weights_ptr as _),
            Gna2Tensor::d1(H as u32, Gna2DataType::Int32, biases_ptr as _),
            None,
        )
        .build(device)
    {
        Ok(model) => model,
        Err(error) => return eprintln!("Failed to compile model: {error}"),
    };
    println!("Model compiled successfully! Model ID: {}", model.id());

    let mut request = match GnaRequestConfig::create(device.library(), model.id()) {
        Ok(request) => request,
        Err(error) => return eprintln!("Failed to create request config: {error}"),
    };
    if let Err(error) = unsafe { request.set_operand_buffer(0, 0, inputs_ptr as _) } {
        return eprintln!("Failed to set input buffer: {error}");
    }
    if let Err(error) = unsafe { request.set_operand_buffer(0, 1, outputs_ptr as _) } {
        return eprintln!("Failed to set output buffer: {error}");
    }
    let _ = request.set_acceleration_mode(Gna2AccelerationMode::Auto);

    if stress {
        let mut config = GnaLoadTestConfig::new().with_concurrency(concurrency);
        if let Some(seconds) = duration {
            config = config.with_duration(Duration::from_secs(seconds));
            config.iterations = iterations;
        } else {
            config = config.with_iterations(iterations.unwrap_or(1000));
        }
        match GnaLoadTester::new(config).run(&mut request) {
            Ok(report) => report.print_summary(),
            Err(error) => eprintln!("Stress test failed: {error}"),
        }
        return;
    }

    if let Err(error) = request.enable_performance_counter() {
        println!("Performance counter unavailable: {error}");
    }
    let request_id = match request.enqueue() {
        Ok(request_id) => request_id,
        Err(error) => return eprintln!("Enqueue failed: {error}"),
    };
    match request.wait(request_id, 1000) {
        Ok(()) => println!("Inference request completed successfully!"),
        Err(error) => return eprintln!("Inference failed / timed out: {error}"),
    }
    if let Ok(stats) = request.get_performance_stats() {
        println!("Total cycles: {}", stats.total_cycles);
        println!("Stall cycles: {}", stats.stall_cycles);
        println!("HW utilization: {:.2}%", stats.hw_usage_percentage());
    }
    let results = unsafe { std::slice::from_raw_parts(outputs_ptr, H * B) };
    println!("Inference outputs: {results:?}");
}

use std::env;
use std::path::PathBuf;

use nanai_gna_dll_load::loader::GnaLibrary;

pub struct DemoOptions {
    pub dll_path: Option<PathBuf>,
    pub env_var: Option<String>,
    pub stress: bool,
    pub iterations: Option<usize>,
    pub duration: Option<u64>,
    pub concurrency: usize,
}

pub fn print_usage(program: &str) {
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

pub fn parse_args() -> Option<DemoOptions> {
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
                None => {
                    eprintln!("Error: --dll requires a path argument.");
                    return None;
                }
            },
            "--env" => match args.next() {
                Some(value) => env_var = Some(value),
                None => {
                    eprintln!("Error: --env requires an environment variable name.");
                    return None;
                }
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
                    None => {
                        eprintln!("Error: --duration requires integer seconds.");
                        return None;
                    }
                }
            }
            "--concurrency" => match args.next().and_then(|value| value.parse::<usize>().ok()) {
                Some(value) => concurrency = value.max(1),
                None => {
                    eprintln!("Error: --concurrency requires a positive integer.");
                    return None;
                }
            },
            "--help" | "-h" => {
                print_usage(&program);
                return None;
            }
            option if option.starts_with('-') => {
                eprintln!("Unknown option: {option}");
                print_usage(&program);
                return None;
            }
            value => dll_path = Some(PathBuf::from(value)),
        }
    }

    Some(DemoOptions {
        dll_path,
        env_var,
        stress,
        iterations,
        duration,
        concurrency,
    })
}

pub fn load_library(options: &DemoOptions) -> Option<GnaLibrary> {
    let library = match (&options.dll_path, &options.env_var) {
        (Some(path), _) => {
            println!("Attempting to load DLL from: {}", path.display());
            GnaLibrary::load_from_path(path)
        }
        (None, Some(variable)) => {
            println!("Attempting to load DLL from environment variable: {variable}");
            GnaLibrary::load_from_env(variable)
        }
        (None, None) => {
            println!(
                "Attempting to load default DLL ({})...",
                GnaLibrary::default_dll_name()
            );
            GnaLibrary::load_default()
        }
    };

    match library {
        Ok(library) => {
            println!("Successfully loaded DLL: {}", library.path().display());
            Some(library)
        }
        Err(error) => {
            eprintln!("\nDLL could not be loaded: {error}");
            eprintln!(
                "Hint: cargo run --example gna_demo -- --dll path/to/{}",
                GnaLibrary::default_dll_name()
            );
            None
        }
    }
}

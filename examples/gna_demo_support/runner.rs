use std::time::Duration;

use nanai_gna_dll_load::{
    device::GnaDevice,
    inference::GnaRequestConfig,
    model::GnaModelBuilder,
    stress::{GnaLoadTestConfig, GnaLoadTester},
    types::{Gna2AccelerationMode, Gna2DataType, Gna2Tensor},
};

use super::cli::DemoOptions;

pub fn run_model_demo(device: &GnaDevice, options: &DemoOptions) {
    const W: usize = 64;
    const H: usize = 64;
    const B: usize = 1;

    let weights = vec![1_i16; W * H];
    let inputs = [1_i16; W * B];
    let biases = [0_i32; H];
    let memory = match device.allocate_buffer(64 * 1024) {
        Ok(memory) => memory,
        Err(error) => return eprintln!("Failed to allocate model memory: {error}"),
    };

    let base = memory.as_raw_ptr() as *mut u8;
    let inputs_ptr = base as *mut i16;
    let outputs_ptr = unsafe { base.add(4096) } as *mut i32;
    let weights_ptr = unsafe { base.add(8192) } as *mut i16;
    let biases_ptr = unsafe { base.add(16384) } as *mut i32;

    unsafe {
        std::ptr::copy_nonoverlapping(inputs.as_ptr(), inputs_ptr, inputs.len());
        std::ptr::copy_nonoverlapping(weights.as_ptr(), weights_ptr, weights.len());
        std::ptr::copy_nonoverlapping(biases.as_ptr(), biases_ptr, biases.len());
        std::ptr::write_bytes(outputs_ptr, 0, H * B);
    }

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
    if let Err(error) = request.set_operand_gna_buffer(0, 0, &memory) {
        return eprintln!("Failed to set input buffer: {error}");
    }
    if let Err(error) = unsafe { request.set_operand_buffer(0, 1, outputs_ptr as _) } {
        return eprintln!("Failed to set output buffer: {error}");
    }
    let _ = request.set_acceleration_mode(Gna2AccelerationMode::Auto);

    if options.stress {
        let mut config = GnaLoadTestConfig::new().with_concurrency(options.concurrency);
        if let Some(seconds) = options.duration {
            config = config
                .with_duration(Duration::from_secs(seconds))
                .with_optional_iterations(options.iterations);
        } else {
            config = config.with_iterations(options.iterations.unwrap_or(1000));
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
        println!("Total cycles: {}", stats.total_cycles());
        println!("Stall cycles: {}", stats.stall_cycles());
        println!("Active cycles: {}", stats.active_cycles());
        println!("HW utilization: {:.2}%", stats.hw_usage_percentage());
        if let Some(exec_us) = stats.execution_time() {
            println!("Execution time: {exec_us} us");
        }
    }
    let results = unsafe { std::slice::from_raw_parts(outputs_ptr, H * B) };
    println!("Inference outputs: {results:?}");
}

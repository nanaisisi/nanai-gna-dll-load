use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::error::{GnaError, Result};
use crate::inference::GnaRequestConfig;
use crate::instrumentation::GnaUsageMonitor;

use super::{GnaLoadTestConfig, GnaLoadTestReport};

/// Load tester runner for GNA inference workloads.
pub struct GnaLoadTester {
    config: GnaLoadTestConfig,
}

struct RunStats {
    usage_monitor: GnaUsageMonitor,
    completed: usize,
    failed: usize,
    latency_min: Duration,
    latency_max: Duration,
    latency_sum: Duration,
}

impl RunStats {
    fn new() -> Self {
        Self {
            usage_monitor: GnaUsageMonitor::new(),
            completed: 0,
            failed: 0,
            latency_min: Duration::from_secs(u64::MAX),
            latency_max: Duration::ZERO,
            latency_sum: Duration::ZERO,
        }
    }

    fn wait_and_record(
        &mut self,
        request_config: &GnaRequestConfig,
        config: &GnaLoadTestConfig,
        in_flight_req: (u32, Instant),
    ) {
        let (req_id, req_start) = in_flight_req;
        match request_config.wait(req_id, config.timeout_per_request_ms) {
            Ok(()) => {
                let lat = req_start.elapsed();
                self.completed += 1;
                self.latency_sum += lat;
                if lat < self.latency_min {
                    self.latency_min = lat;
                }
                if lat > self.latency_max {
                    self.latency_max = lat;
                }

                if config.track_hw_usage
                    && let Ok(stats) = request_config.get_performance_stats()
                {
                    self.usage_monitor.record(&stats);
                }
            }
            Err(_) => {
                self.failed += 1;
            }
        }
    }
}

impl GnaLoadTester {
    /// Create a new load tester with the specified configuration.
    pub fn new(config: GnaLoadTestConfig) -> Self {
        Self { config }
    }

    /// Run the load test using the provided configured request.
    pub fn run(&self, request_config: &mut GnaRequestConfig) -> Result<GnaLoadTestReport> {
        let never = std::sync::atomic::AtomicBool::new(false);
        self.run_with_cancel(request_config, &never)
    }

    /// Run the load test, stopping early if `cancel` becomes `true`.
    pub fn run_with_cancel(
        &self,
        request_config: &mut GnaRequestConfig,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<GnaLoadTestReport> {
        use std::sync::atomic::Ordering;
        if self.config.track_hw_usage && request_config.instrumentation().is_none() {
            let _ = request_config.enable_performance_counter();
        }

        let concurrency = self.config.concurrency.max(1);
        let max_iterations = self.config.iterations.unwrap_or(usize::MAX);
        let max_duration = self.config.duration;

        let mut stats = RunStats::new();

        // Queue to track in-flight requests: (request_id, submit_instant)
        let mut in_flight: VecDeque<(u32, Instant)> = VecDeque::with_capacity(concurrency);

        let start_time = Instant::now();

        while !cancel.load(Ordering::Relaxed) && stats.completed + stats.failed < max_iterations {
            if let Some(dur) = max_duration
                && start_time.elapsed() >= dur
            {
                break;
            }

            // Enqueue up to concurrency limit
            while in_flight.len() < concurrency
                && (stats.completed + stats.failed + in_flight.len()) < max_iterations
            {
                if let Some(dur) = max_duration
                    && start_time.elapsed() >= dur
                {
                    break;
                }

                let req_start = Instant::now();
                match request_config.enqueue() {
                    Ok(req_id) => {
                        in_flight.push_back((req_id, req_start));
                    }
                    Err(err) => {
                        stats.failed += 1;
                        if in_flight.is_empty() {
                            return Err(GnaError::Other(format!(
                                "Failed to enqueue load test request: {}",
                                err
                            )));
                        }
                        break;
                    }
                }
            }

            if let Some(req) = in_flight.pop_front() {
                stats.wait_and_record(request_config, &self.config, req);
            } else {
                break;
            }
        }

        // Drain any remaining in-flight requests (drain in-place so deallocation stays outside elapsed timing)
        for req in in_flight.drain(..) {
            stats.wait_and_record(request_config, &self.config, req);
        }

        let elapsed = if max_iterations == 0 {
            Duration::ZERO
        } else {
            start_time.elapsed()
        };
        let ips = if elapsed.as_secs_f64() > 0.0 {
            (stats.completed as f64) / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let latency_avg = if stats.completed > 0 {
            stats.latency_sum / (stats.completed as u32)
        } else {
            Duration::ZERO
        };

        if stats.latency_min == Duration::from_secs(u64::MAX) {
            stats.latency_min = Duration::ZERO;
        }

        Ok(GnaLoadTestReport {
            completed_inferences: stats.completed,
            failed_inferences: stats.failed,
            elapsed_time: elapsed,
            inferences_per_second: ips,
            latency_min: stats.latency_min,
            latency_max: stats.latency_max,
            latency_avg,
            usage_monitor: stats.usage_monitor,
        })
    }

    /// Run a self-contained 64x64 dummy FCA workload on `device` with `config`.
    pub fn run_default_workload(
        device: &crate::device::GnaDevice,
        config: GnaLoadTestConfig,
    ) -> Result<GnaLoadTestReport> {
        let never = std::sync::atomic::AtomicBool::new(false);
        Self::run_default_workload_with_cancel(device, config, &never)
    }

    /// Run a self-contained 64x64 dummy FCA workload on `device` with `config`,
    /// stopping early if `cancel` becomes `true`.
    pub fn run_default_workload_with_cancel(
        device: &crate::device::GnaDevice,
        config: GnaLoadTestConfig,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<GnaLoadTestReport> {
        const W: usize = 64;
        const H: usize = 64;

        let weights_bytes = W * H * std::mem::size_of::<i16>();
        let inputs_bytes = W * std::mem::size_of::<i16>();
        let biases_bytes = H * std::mem::size_of::<i32>();
        let outputs_bytes = H * std::mem::size_of::<i32>();

        let mut buf =
            device.allocate_buffer(weights_bytes + inputs_bytes + biases_bytes + outputs_bytes)?;
        buf.as_mut_slice().fill(0);

        let base = buf.as_raw_ptr() as *mut u8;
        let weights_ptr = base as *mut std::ffi::c_void;
        let inputs_ptr = unsafe { base.add(weights_bytes) } as *mut std::ffi::c_void;
        let biases_ptr = unsafe { base.add(weights_bytes + inputs_bytes) } as *mut std::ffi::c_void;
        let outputs_ptr = unsafe { base.add(weights_bytes + inputs_bytes + biases_bytes) }
            as *mut std::ffi::c_void;

        let model = crate::model::GnaModelBuilder::new()
            .add_fully_connected_affine(
                crate::types::Gna2Tensor::d2(
                    W as u32,
                    1,
                    crate::types::Gna2DataType::Int16,
                    inputs_ptr,
                ),
                crate::types::Gna2Tensor::d2(
                    H as u32,
                    1,
                    crate::types::Gna2DataType::Int32,
                    outputs_ptr,
                ),
                crate::types::Gna2Tensor::d2(
                    H as u32,
                    W as u32,
                    crate::types::Gna2DataType::Int16,
                    weights_ptr,
                ),
                crate::types::Gna2Tensor::d1(
                    H as u32,
                    crate::types::Gna2DataType::Int32,
                    biases_ptr,
                ),
                None,
            )
            .build(device)?;

        let mut request = crate::inference::GnaRequestConfig::create(device.library(), model.id())?;
        unsafe {
            request.set_operand_buffer(0, 0, inputs_ptr)?;
            request.set_operand_buffer(0, 1, outputs_ptr)?;
        }
        let _ = request.set_acceleration_mode(crate::types::Gna2AccelerationMode::Auto);

        Self::new(config).run_with_cancel(&mut request, cancel)
    }
}

use std::time::{Duration, Instant};

use crate::error::{GnaError, Result};
use crate::inference::GnaRequestConfig;
use crate::instrumentation::GnaUsageMonitor;

use super::{GnaLoadTestConfig, GnaLoadTestReport};

/// Load tester runner for GNA inference workloads.
pub struct GnaLoadTester {
    config: GnaLoadTestConfig,
}

impl GnaLoadTester {
    /// Create a new load tester with the specified configuration.
    pub fn new(config: GnaLoadTestConfig) -> Self {
        Self { config }
    }

    /// Run the load test using the provided configured request.
    pub fn run(&self, request_config: &mut GnaRequestConfig) -> Result<GnaLoadTestReport> {
        if self.config.track_hw_usage && request_config.instrumentation().is_none() {
            let _ = request_config.enable_performance_counter();
        }

        let concurrency = self.config.concurrency.max(1);
        let max_iterations = self.config.iterations.unwrap_or(usize::MAX);
        let max_duration = self.config.duration;

        let mut usage_monitor = GnaUsageMonitor::new();
        let mut completed = 0usize;
        let mut failed = 0usize;

        let mut latency_min = Duration::from_secs(u64::MAX);
        let mut latency_max = Duration::ZERO;
        let mut latency_sum = Duration::ZERO;

        // Queue to track in-flight requests: (request_id, submit_instant)
        let mut in_flight: Vec<(u32, Instant)> = Vec::with_capacity(concurrency);

        let start_time = Instant::now();

        while completed + failed < max_iterations {
            if let Some(dur) = max_duration
                && start_time.elapsed() >= dur
            {
                break;
            }

            // Enqueue up to concurrency limit
            while in_flight.len() < concurrency
                && (completed + failed + in_flight.len()) < max_iterations
            {
                if let Some(dur) = max_duration
                    && start_time.elapsed() >= dur
                {
                    break;
                }

                let req_start = Instant::now();
                match request_config.enqueue() {
                    Ok(req_id) => {
                        in_flight.push((req_id, req_start));
                    }
                    Err(err) => {
                        failed += 1;
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

            if in_flight.is_empty() {
                break;
            }

            // Wait for oldest request in queue
            let (req_id, req_start) = in_flight.remove(0);
            match request_config.wait(req_id, self.config.timeout_per_request_ms) {
                Ok(()) => {
                    let lat = req_start.elapsed();
                    completed += 1;
                    latency_sum += lat;
                    if lat < latency_min {
                        latency_min = lat;
                    }
                    if lat > latency_max {
                        latency_max = lat;
                    }

                    if self.config.track_hw_usage
                        && let Ok(stats) = request_config.get_performance_stats()
                    {
                        usage_monitor.record(&stats);
                    }
                }
                Err(_) => {
                    failed += 1;
                }
            }
        }

        // Drain any remaining in-flight requests (drain in-place so deallocation stays outside elapsed timing)
        for (req_id, req_start) in in_flight.drain(..) {
            match request_config.wait(req_id, self.config.timeout_per_request_ms) {
                Ok(()) => {
                    let lat = req_start.elapsed();
                    completed += 1;
                    latency_sum += lat;
                    if lat < latency_min {
                        latency_min = lat;
                    }
                    if lat > latency_max {
                        latency_max = lat;
                    }

                    if self.config.track_hw_usage
                        && let Ok(stats) = request_config.get_performance_stats()
                    {
                        usage_monitor.record(&stats);
                    }
                }
                Err(_) => {
                    failed += 1;
                }
            }
        }

        let elapsed = if max_iterations == 0 {
            Duration::ZERO
        } else {
            start_time.elapsed()
        };
        let ips = if elapsed.as_secs_f64() > 0.0 {
            (completed as f64) / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let latency_avg = if completed > 0 {
            latency_sum / (completed as u32)
        } else {
            Duration::ZERO
        };

        if latency_min == Duration::from_secs(u64::MAX) {
            latency_min = Duration::ZERO;
        }

        Ok(GnaLoadTestReport {
            completed_inferences: completed,
            failed_inferences: failed,
            elapsed_time: elapsed,
            inferences_per_second: ips,
            latency_min,
            latency_max,
            latency_avg,
            usage_monitor,
        })
    }
}

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

        while stats.completed + stats.failed < max_iterations {
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
}

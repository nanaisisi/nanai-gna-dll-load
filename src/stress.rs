use std::time::{Duration, Instant};

use crate::error::{GnaError, Result};
use crate::inference::GnaRequestConfig;
use crate::instrumentation::GnaUsageMonitor;

/// Configuration options for a GNA load test.
#[derive(Debug, Clone)]
pub struct GnaLoadTestConfig {
    /// Number of iterations to run (if duration is not set or whichever completes first).
    pub iterations: Option<usize>,
    /// Maximum duration to run the load test.
    pub duration: Option<Duration>,
    /// Number of concurrent in-flight requests (pipeline depth). Default: 1.
    pub concurrency: usize,
    /// Timeout per inference request in milliseconds. Default: 1000 ms.
    pub timeout_per_request_ms: u32,
    /// Whether to track hardware utilization metrics using instrumentation. Default: true.
    pub track_hw_usage: bool,
}

impl Default for GnaLoadTestConfig {
    fn default() -> Self {
        Self {
            iterations: Some(1000),
            duration: None,
            concurrency: 1,
            timeout_per_request_ms: 1000,
            track_hw_usage: true,
        }
    }
}

impl GnaLoadTestConfig {
    /// Create a new configuration with default 1000 iterations.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set total iterations to execute.
    pub fn with_iterations(mut self, count: usize) -> Self {
        self.iterations = Some(count);
        self
    }

    /// Set total duration to run the test.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set concurrency (number of in-flight requests in queue).
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency.max(1);
        self
    }

    /// Set timeout per request in milliseconds.
    pub fn with_timeout_ms(mut self, timeout_ms: u32) -> Self {
        self.timeout_per_request_ms = timeout_ms;
        self
    }

    /// Enable or disable hardware usage tracking.
    pub fn with_track_hw_usage(mut self, enable: bool) -> Self {
        self.track_hw_usage = enable;
        self
    }
}

/// Comprehensive report containing performance, throughput, and latency metrics from a load test.
#[derive(Debug, Clone)]
pub struct GnaLoadTestReport {
    /// Number of successful inferences completed.
    pub completed_inferences: usize,
    /// Number of failed or timed-out inferences.
    pub failed_inferences: usize,
    /// Total wall-clock time elapsed for the load test.
    pub elapsed_time: Duration,
    /// Throughput in inferences per second (IPS).
    pub inferences_per_second: f64,
    /// Minimum observed inference latency.
    pub latency_min: Duration,
    /// Maximum observed inference latency.
    pub latency_max: Duration,
    /// Average observed inference latency.
    pub latency_avg: Duration,
    /// Hardware utilization monitor containing accumulated cycles & usage rate.
    pub usage_monitor: GnaUsageMonitor,
}

impl GnaLoadTestReport {
    /// Print a formatted summary report to standard output.
    pub fn print_summary(&self) {
        println!("==================================================");
        println!("           GNA Load Test Report                   ");
        println!("==================================================");
        println!("  Completed Inferences : {}", self.completed_inferences);
        println!("  Failed Inferences    : {}", self.failed_inferences);
        println!("  Elapsed Time         : {:.3} s", self.elapsed_time.as_secs_f64());
        println!("  Throughput           : {:.2} inferences/sec", self.inferences_per_second);
        println!("  Latency (Min)        : {:.3} ms", self.latency_min.as_secs_f64() * 1000.0);
        println!("  Latency (Avg)        : {:.3} ms", self.latency_avg.as_secs_f64() * 1000.0);
        println!("  Latency (Max)        : {:.3} ms", self.latency_max.as_secs_f64() * 1000.0);

        if self.usage_monitor.inference_count() > 0 {
            println!("--------------------------------------------------");
            println!("  Hardware Instrumentation Metrics:");
            println!("  Tracked Inferences   : {}", self.usage_monitor.inference_count());
            println!("  Cumulative Total     : {} cycles", self.usage_monitor.cumulative_total_cycles());
            println!("  Cumulative Stall     : {} cycles", self.usage_monitor.cumulative_stall_cycles());
            println!("  Cumulative Active    : {} cycles", self.usage_monitor.cumulative_active_cycles());
            println!("  Weighted HW Usage    : {:.2}%", self.usage_monitor.cumulative_hw_usage_percentage());
            if let Some(avg_t) = self.usage_monitor.average_execution_time() {
                println!("  Avg Hardware Time    : {:.2} cycles/us", avg_t);
            }
        }
        println!("==================================================");
    }
}

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

        let start_time = Instant::now();

        // Queue to track in-flight requests: (request_id, submit_instant)
        let mut in_flight: Vec<(u32, Instant)> = Vec::with_capacity(concurrency);

        while completed + failed < max_iterations {
            if let Some(dur) = max_duration
                && start_time.elapsed() >= dur {
                    break;
                }

            // Enqueue up to concurrency limit
            while in_flight.len() < concurrency && (completed + failed + in_flight.len()) < max_iterations {
                if let Some(dur) = max_duration
                    && start_time.elapsed() >= dur {
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
                        && let Ok(stats) = request_config.get_performance_stats() {
                            usage_monitor.record(&stats);
                        }
                }
                Err(_) => {
                    failed += 1;
                }
            }
        }

        // Drain any remaining in-flight requests
        for (req_id, req_start) in in_flight {
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
                        && let Ok(stats) = request_config.get_performance_stats() {
                            usage_monitor.record(&stats);
                        }
                }
                Err(_) => {
                    failed += 1;
                }
            }
        }

        let elapsed = start_time.elapsed();
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

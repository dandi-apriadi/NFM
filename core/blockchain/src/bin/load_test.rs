use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Instant;

// =====================================================================
// Configuration and Metrics Types
// =====================================================================

#[derive(Debug, Clone)]
struct LoadTestConfig {
    api_host: String,
    api_port: u16,
    concurrent_workers: usize,
    requests_per_worker: usize,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            api_host: "127.0.0.1".to_string(),
            api_port: 8080,
            concurrent_workers: 4,
            requests_per_worker: 100,
        }
    }
}

#[derive(Debug, Clone)]
struct RequestMetric {
    endpoint: String,
    method: String,
    response_time_ms: f64,
    status_code: u16,
    success: bool,
}

#[derive(Debug, Clone)]
struct EndpointStats {
    endpoint: String,
    method: String,
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    min_latency_ms: f64,
    max_latency_ms: f64,
    mean_latency_ms: f64,
    p50_latency_ms: f64,
    p95_latency_ms: f64,
    p99_latency_ms: f64,
    throughput_rps: f64,
}

#[derive(Debug, Clone)]
struct LoadTestReport {
    total_requests: usize,
    total_successful: usize,
    total_failed: usize,
    total_duration_secs: f64,
    overall_throughput_rps: f64,
    endpoint_stats: Vec<EndpointStats>,
}

// =====================================================================
// Load Test Runner
// =====================================================================

struct LoadTestRunner {
    config: LoadTestConfig,
    metrics: Arc<Mutex<Vec<RequestMetric>>>,
}

impl LoadTestRunner {
    fn new(config: LoadTestConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn run(&self) -> LoadTestReport {
        println!("\n========================================");
        println!("  NFM Blockchain Load Test");
        println!("========================================");
        println!("Config:");
        println!("  Endpoint: {}:{}", self.config.api_host, self.config.api_port);
        println!("  Workers: {}", self.config.concurrent_workers);
        println!("  Requests/Worker: {}", self.config.requests_per_worker);
        println!();

        let start = Instant::now();

        // Spawn concurrent workers
        let mut handles = vec![];

        for worker_id in 0..self.config.concurrent_workers {
            let config = self.config.clone();
            let metrics = Arc::clone(&self.metrics);

            let handle = std::thread::spawn(move || {
                Self::worker_loop(worker_id, config, metrics);
            });

            handles.push(handle);
        }

        // Wait for all workers to complete
        for handle in handles {
            let _ = handle.join();
        }

        let duration = start.elapsed();

        // Analyze results
        self.generate_report(duration)
    }

    fn worker_loop(worker_id: usize, config: LoadTestConfig, metrics: Arc<Mutex<Vec<RequestMetric>>>) {
        println!("[Worker {}] Starting {} requests", worker_id, config.requests_per_worker);

        let endpoints = vec![
            ("GET", "/api/status"),
            ("GET", "/api/brain/status"),
            ("POST", "/api/brain/route"),
            ("POST", "/api/brain/benchmark"),
            ("POST", "/api/transfer/create"),
            ("GET", "/api/wallets"),
            ("GET", "/api/blocks"),
            ("GET", "/api/mempool"),
        ];

        let mut endpoint_idx = 0;

        for req_num in 0..config.requests_per_worker {
            let (method, path) = endpoints[endpoint_idx % endpoints.len()];
            endpoint_idx += 1;

            let metric = Self::send_request(&config, method, path);
            metrics.lock().unwrap().push(metric);

            if (req_num + 1) % 10 == 0 {
                println!("[Worker {}] Completed {}/{} requests", worker_id, req_num + 1, config.requests_per_worker);
            }
        }

        println!("[Worker {}] DONE", worker_id);
    }

    fn send_request(config: &LoadTestConfig, method: &str, path: &str) -> RequestMetric {
        let start = Instant::now();
        let addr = format!("{}:{}", config.api_host, config.api_port);

        let (status_code, success) = match TcpStream::connect(&addr) {
            Ok(mut stream) => {
                let body = match method {
                    "POST" => Self::get_request_body(path),
                    _ => String::new(),
                };

                let request = match method {
                    "POST" => format!(
                        "POST {} HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        path,
                        config.api_host,
                        config.api_port,
                        body.len(),
                        body
                    ),
                    _ => format!(
                        "GET {} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n\r\n",
                        path,
                        config.api_host,
                        config.api_port
                    ),
                };

                if stream.write_all(request.as_bytes()).is_err() {
                    return RequestMetric {
                        endpoint: path.to_string(),
                        method: method.to_string(),
                        response_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                        status_code: 0,
                        success: false,
                    };
                }

                let _ = stream.shutdown(std::net::Shutdown::Write);

                let mut response = String::new();
                if stream.read_to_string(&mut response).is_err() {
                    return RequestMetric {
                        endpoint: path.to_string(),
                        method: method.to_string(),
                        response_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                        status_code: 0,
                        success: false,
                    };
                }

                let status = response
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|code| code.parse::<u16>().ok())
                    .unwrap_or(0);

                (status, status >= 200 && status < 400)
            }
            Err(_) => (0, false),
        };

        let elapsed = start.elapsed().as_secs_f64() * 1000.0; // Convert to ms

        RequestMetric {
            endpoint: path.to_string(),
            method: method.to_string(),
            response_time_ms: elapsed,
            status_code,
            success,
        }
    }

    fn get_request_body(path: &str) -> String {
        match path {
            "/api/brain/route" => format!(r#"{{"requester_node_id":"load_tester","user_latitude":-6.2088,"user_longitude":106.8456,"data_class":"global","critical":false}}"#),
            "/api/brain/benchmark" => format!(r#"{{"requester_node_id":"load_tester","user_latitude":-6.2088,"user_longitude":106.8456,"data_class":"global","critical":false}}"#),
            "/api/transfer/create" => format!(r#"{{"from":"test_sender","to":"test_receiver","amount":1.5}}"#),
            _ => "{}".to_string(),
        }
    }

    fn generate_report(&self, total_duration: std::time::Duration) -> LoadTestReport {
        let metrics = self.metrics.lock().unwrap();
        let total_duration_secs = total_duration.as_secs_f64();

        let total_requests = metrics.len();
        let total_successful = metrics.iter().filter(|m| m.success).count();
        let total_failed = metrics.iter().filter(|m| !m.success).count();
        let overall_throughput_rps = total_requests as f64 / total_duration_secs;

        // Group metrics by endpoint
        let mut endpoint_map: std::collections::HashMap<(String, String), Vec<f64>> = std::collections::HashMap::new();

        for metric in metrics.iter() {
            let key = (metric.endpoint.clone(), metric.method.clone());
            endpoint_map
                .entry(key)
                .or_insert_with(Vec::new)
                .push(metric.response_time_ms);
        }

        let mut endpoint_stats = vec![];

        for ((endpoint, method), mut latencies) in endpoint_map {
            latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let count = latencies.len();
            let successful = metrics
                .iter()
                .filter(|m| m.endpoint == endpoint && m.method == method && m.success)
                .count();
            let failed = count - successful;

            let min = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = latencies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let mean = latencies.iter().sum::<f64>() / count as f64;

            let p50_idx = (count as f64 * 0.50) as usize;
            let p95_idx = (count as f64 * 0.95) as usize;
            let p99_idx = (count as f64 * 0.99) as usize;

            let p50 = latencies.get(p50_idx).copied().unwrap_or(0.0);
            let p95 = latencies.get(p95_idx).copied().unwrap_or(0.0);
            let p99 = latencies.get(p99_idx).copied().unwrap_or(0.0);

            let throughput = count as f64 / total_duration_secs;

            endpoint_stats.push(EndpointStats {
                endpoint,
                method,
                total_requests: count,
                successful_requests: successful,
                failed_requests: failed,
                min_latency_ms: min,
                max_latency_ms: max,
                mean_latency_ms: mean,
                p50_latency_ms: p50,
                p95_latency_ms: p95,
                p99_latency_ms: p99,
                throughput_rps: throughput,
            });
        }

        // Sort by endpoint name
        endpoint_stats.sort_by(|a, b| a.endpoint.cmp(&b.endpoint));

        LoadTestReport {
            total_requests,
            total_successful,
            total_failed,
            total_duration_secs,
            overall_throughput_rps,
            endpoint_stats,
        }
    }
}

// =====================================================================
// Report Formatter
// =====================================================================

fn print_report(report: &LoadTestReport) {
    println!("\n========================================");
    println!("  LOAD TEST RESULTS");
    println!("========================================\n");

    println!("OVERALL METRICS");
    println!("  Total Requests: {}", report.total_requests);
    println!("  Successful: {} ({:.1}%)",
        report.total_successful,
        (report.total_successful as f64 / report.total_requests as f64) * 100.0
    );
    println!("  Failed: {} ({:.1}%)",
        report.total_failed,
        (report.total_failed as f64 / report.total_requests as f64) * 100.0
    );
    println!("  Duration: {:.2} seconds", report.total_duration_secs);
    println!("  Overall Throughput: {:.2} req/sec", report.overall_throughput_rps);
    println!();

    println!("ENDPOINT BREAKDOWN");
    println!("{:<30} {:<8} {:<8} {:<10} {:<10} {:<10} {:<10} {:<10}",
        "Endpoint", "Method", "N Reqs", "Success%", "Min(ms)", "P50(ms)", "P95(ms)", "P99(ms)");
    println!("{}", "-".repeat(110));

    for stat in &report.endpoint_stats {
        let success_pct = (stat.successful_requests as f64 / stat.total_requests as f64) * 100.0;
        let endpoint_display = if stat.endpoint.len() > 28 {
            format!("{}...", &stat.endpoint[0..25])
        } else {
            stat.endpoint.clone()
        };

        println!("{:<30} {:<8} {:<8} {:<10.1} {:<10.2} {:<10.2} {:<10.2} {:<10.2}",
            endpoint_display,
            stat.method,
            stat.total_requests,
            success_pct,
            stat.min_latency_ms,
            stat.p50_latency_ms,
            stat.p95_latency_ms,
            stat.p99_latency_ms
        );
    }

    println!("\nBOTTLENECK ANALYSIS");

    // Find slowest endpoints (by p99)
    let mut sorted = report.endpoint_stats.clone();
    sorted.sort_by(|a, b| b.p99_latency_ms.partial_cmp(&a.p99_latency_ms).unwrap());

    println!("Top 3 Slowest Endpoints (by p99):");
    for (idx, stat) in sorted.iter().take(3).enumerate() {
        println!("  {}. {} {} - p99: {:.2}ms",
            idx + 1,
            stat.method,
            stat.endpoint,
            stat.p99_latency_ms
        );
    }

    // Find highest error rates
    let mut sorted_errors: Vec<_> = report.endpoint_stats.iter()
        .map(|s| (s, s.failed_requests as f64 / s.total_requests as f64))
        .collect();
    sorted_errors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    if !sorted_errors.is_empty() && sorted_errors[0].1 > 0.0 {
        println!("\nError Rates:");
        for (stat, error_rate) in sorted_errors.iter().take(3) {
            if *error_rate > 0.0 {
                println!("  {} {} - {:.1}% failure",
                    stat.method,
                    stat.endpoint,
                    error_rate * 100.0
                );
            }
        }
    }

    println!("\n========================================");
}

// =====================================================================
// Main
// =====================================================================

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse command line arguments
    let mut config = LoadTestConfig::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--host" if i + 1 < args.len() => {
                config.api_host = args[i + 1].clone();
                i += 2;
            }
            "--port" if i + 1 < args.len() => {
                config.api_port = args[i + 1].parse().unwrap_or(8080);
                i += 2;
            }
            "--workers" if i + 1 < args.len() => {
                config.concurrent_workers = args[i + 1].parse().unwrap_or(4);
                i += 2;
            }
            "--requests" if i + 1 < args.len() => {
                config.requests_per_worker = args[i + 1].parse().unwrap_or(100);
                i += 2;
            }
            "-h" | "--help" => {
                println!("NFM Blockchain Load Test");
                println!();
                println!("Usage: load_test [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --host <HOST>              API host (default: 127.0.0.1)");
                println!("  --port <PORT>              API port (default: 8080)");
                println!("  --workers <NUM>            Concurrent workers (default: 4)");
                println!("  --requests <NUM>           Requests per worker (default: 100)");
                println!("  -h, --help                 Show this help message");
                println!();
                println!("Example:");
                println!("  load_test --host 127.0.0.1 --port 5000 --workers 8 --requests 200");
                return;
            }
            _ => {
                i += 1;
            }
        }
    }

    // Run load test
    let runner = LoadTestRunner::new(config);
    let report = runner.run();
    print_report(&report);

    // Summary for CI/CD
    println!("\nRun this to start the blockchain API (if not already running):");
    println!("  cd core/blockchain && cargo run --release");
}

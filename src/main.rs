use clap::Parser;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Instant, Duration};
use std::collections::HashMap;
use futures::stream::{FuturesUnordered, StreamExt};

pub mod client;
pub mod utils;

use client::ErrorStats;
use utils::{percentile, is_success_status};

#[derive(Parser)]
#[command(version, about = "HTTP/3 load testing tool")]
struct Cli {
    #[arg(long)]
    target: String,

    #[arg(long, default_value = "443")]
    port: u16,

    #[arg(short = 'n', long, default_value = "1000")]
    requests: usize,

    #[arg(short = 'w', long, default_value = "1")]
    workers: usize,

    #[arg(short = 't', long, default_value = "30")]
    duration: u64,

    #[arg(long, default_value = "/")]
    path: String,

    #[arg(long)]
    insecure: bool,

    #[arg(long, default_value = "false")]
    verbose: bool,

    #[arg(long, default_value = "2xx", help = "HTTP status codes to consider as success (e.g., '2xx', '2xx,3xx', or specific codes '200,201,301')")]
    success_status: String,

    #[arg(short = 'c', long, default_value = "1", help = "Number of concurrent in-flight requests per worker")]
    concurrency: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.workers == 0 {
        eprintln!("workers must be at least 1");
        std::process::exit(1);
    }

    let host = cli.target.clone();

    if cli.concurrency == 0 {
        eprintln!("concurrency must be at least 1");
        std::process::exit(1);
    }

    println!("Starting HTTP/3 load test:");
    println!("  Target: {}:{}", cli.target, cli.port);
    println!("  Host: {}", host);
    println!("  Path: {}", cli.path);
    println!("  Workers: {}", cli.workers);
    println!("  Concurrency per worker: {}", cli.concurrency);
    println!("  Total requests: {}", cli.requests);
    println!("  Duration: {}s", cli.duration);
    println!("  Insecure: {}", cli.insecure);
    if cli.verbose {
        println!("  Verbose: enabled");
    }
    println!();

    let start_time = Instant::now();
    let deadline = start_time + Duration::from_secs(cli.duration);
    let deadline = Arc::new(deadline);
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    let mut total_errors = ErrorStats::default();
    let mut status_code_counts: HashMap<u16, usize> = HashMap::new();
    let mut worker_failures = 0;
    let mut all_latencies = Vec::new();

    let mut handles = vec![];

    // Distribute requests: quotient to all workers, remainder to first N workers
    let quotient = cli.requests / cli.workers;
    let remainder = cli.requests % cli.workers;

    for worker_id in 0..cli.workers {
        let target = cli.target.clone();
        let port = cli.port;
        let host = host.clone();
        let path = cli.path.clone();
        let insecure = cli.insecure;
        let verbose = cli.verbose;
        let success_status = cli.success_status.clone();
        let requests_per_worker = quotient + if worker_id < remainder { 1 } else { 0 };
        let concurrency = cli.concurrency;
        let deadline = Arc::clone(&deadline);

        let handle = tokio::spawn(async move {
            let mut success = 0usize;
            let mut fail = 0usize;
            let mut total_errors = ErrorStats::default();
            let mut status_codes: HashMap<u16, usize> = HashMap::new();
            let mut latencies = Vec::new();

            // Shared counter of how many requests have been dispatched so far.
            let dispatched = Arc::new(AtomicUsize::new(0));

            // Spawn a new independent task per in-flight slot; each owns its own
            // Http3Client so there is no &mut aliasing between concurrent futures.
            let make_request = |req_index: usize| {
                let target = target.clone();
                let host = host.clone();
                let path = path.clone();
                let success_status = success_status.clone();
                tokio::spawn(async move {
                    let mut c = client::h3_client::Http3Client::new(insecure)
                        .map_err(|e| format!("init: {e}"))?;
                    let result = c.send_request(&target, port, &host, &path, verbose)
                        .await
                        .map_err(|e| format!("{e}"))?;
                    let ok = is_success_status(result.status_code, &success_status);
                    Ok::<_, String>((req_index, ok, result))
                })
            };

            // Seed the in-flight set up to `concurrency` slots.
            let mut in_flight: FuturesUnordered<_> = FuturesUnordered::new();
            for _ in 0..concurrency {
                if Instant::now() >= *deadline {
                    break;
                }
                let i = dispatched.fetch_add(1, Ordering::Relaxed);
                if i >= requests_per_worker {
                    break;
                }
                in_flight.push(make_request(i));
            }

            // Drive the set: as each completes, record the result and backfill.
            while let Some(join_result) = in_flight.next().await {
                match join_result {
                    Ok(Ok((_i, ok, result))) => {
                        *status_codes.entry(result.status_code).or_insert(0) += 1;
                        if ok { success += 1; } else { fail += 1; }
                        total_errors.send_errors += result.errors.send_errors;
                        total_errors.recv_errors += result.errors.recv_errors;
                        total_errors.quic_errors += result.errors.quic_errors;
                        total_errors.stream_reset_errors += result.errors.stream_reset_errors;
                        latencies.push(result.latency_ms);
                    }
                    Ok(Err(e)) => {
                        eprintln!("Worker {worker_id}: request failed: {e}");
                        fail += 1;
                    }
                    Err(e) => {
                        eprintln!("Worker {worker_id}: task panicked: {e}");
                        fail += 1;
                    }
                }

                // Backfill: try to keep `concurrency` slots busy.
                if Instant::now() < *deadline {
                    let i = dispatched.fetch_add(1, Ordering::Relaxed);
                    if i < requests_per_worker {
                        in_flight.push(make_request(i));
                    }
                }
            }

            (success, fail, total_errors, status_codes, latencies)
        });

        handles.push(handle);
    }

    for (worker_id, handle) in handles.into_iter().enumerate() {
        match handle.await {
            Ok((s, f, errors, status_codes, latencies)) => {
                total_requests += s + f;
                successful_requests += s;
                failed_requests += f;
                total_errors.send_errors += errors.send_errors;
                total_errors.recv_errors += errors.recv_errors;
                total_errors.quic_errors += errors.quic_errors;
                total_errors.stream_reset_errors += errors.stream_reset_errors;

                // Aggregate status code counts
                for (code, count) in status_codes {
                    *status_code_counts.entry(code).or_insert(0) += count;
                }

                // Aggregate latencies
                all_latencies.extend(latencies);
            }
            Err(join_err) => {
                worker_failures += 1;
                if join_err.is_panic() {
                    eprintln!("Worker {}: Panicked", worker_id);
                } else if join_err.is_cancelled() {
                    eprintln!("Worker {}: Cancelled", worker_id);
                } else {
                    eprintln!("Worker {}: Failed with unknown error", worker_id);
                }
            }
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let hit_duration_limit = Instant::now() >= *deadline;

    println!("\nLoad test completed:");
    println!("  Total time: {:.2}s", elapsed);
    println!("  Total requests: {}", total_requests);
    println!("  Successful requests: {}", successful_requests);
    println!("  Failed requests: {}", failed_requests);
    println!("  Requests/sec: {:.2}", if elapsed > 0.0 { total_requests as f64 / elapsed } else { 0.0 });

    if hit_duration_limit {
        println!("  Completion reason: Duration limit ({:.0}s) reached", cli.duration);
    } else {
        println!("  Completion reason: All {} requests completed", cli.requests);
    }

    // Report error breakdown
    let has_errors = total_errors.send_errors > 0
        || total_errors.recv_errors > 0
        || total_errors.quic_errors > 0
        || total_errors.stream_reset_errors > 0;

    if has_errors {
        println!("\nError breakdown:");
        if total_errors.send_errors > 0 {
            println!("  Network send errors: {}", total_errors.send_errors);
        }
        if total_errors.recv_errors > 0 {
            println!("  Network recv errors: {}", total_errors.recv_errors);
        }
        if total_errors.quic_errors > 0 {
            println!("  QUIC/protocol errors: {}", total_errors.quic_errors);
        }
        if total_errors.stream_reset_errors > 0 {
            println!("  Stream reset errors: {}", total_errors.stream_reset_errors);
        }
    }

    // Report HTTP status code breakdown
    if !status_code_counts.is_empty() {
        println!("\nHTTP Status code breakdown:");
        let mut sorted_codes: Vec<_> = status_code_counts.iter().collect();
        sorted_codes.sort_by_key(|&(code, _)| code);

        for (code, count) in sorted_codes {
            let status_desc = match *code {
                200..=299 => "2xx Success",
                300..=399 => "3xx Redirect",
                400..=499 => "4xx Client Error",
                500..=599 => "5xx Server Error",
                _ => "Unknown",
            };
            println!("  {}: {} ({})", code, status_desc, count);
        }
    }

    // Report latency metrics
    if !all_latencies.is_empty() {
        println!("\nLatency metrics (ms):");

        let mut sorted_latencies = all_latencies.clone();
        sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = sorted_latencies[0];
        let max = sorted_latencies[sorted_latencies.len() - 1];
        let avg = sorted_latencies.iter().sum::<f64>() / sorted_latencies.len() as f64;
        let p50 = percentile(&sorted_latencies, 50.0);
        let p90 = percentile(&sorted_latencies, 90.0);
        let p95 = percentile(&sorted_latencies, 95.0);
        let p99 = percentile(&sorted_latencies, 99.0);

        println!("  Min:  {:.2}", min);
        println!("  Max:  {:.2}", max);
        println!("  Avg:  {:.2}", avg);
        println!("  p50:  {:.2}", p50);
        println!("  p90:  {:.2}", p90);
        println!("  p95:  {:.2}", p95);
        println!("  p99:  {:.2}", p99);
    }

    // Report worker failures
    if worker_failures > 0 {
        eprintln!(
            "\nWarning: {} worker(s) failed or panicked",
            worker_failures
        );
        eprintln!(
            "This may indicate system instability or resource exhaustion during the load test."
        );
        return Err(format!(
            "{} worker failure(s) detected",
            worker_failures
        )
        .into());
    }

    // Verify that all requested requests were sent (only if we didn't hit duration limit)
    if !hit_duration_limit && total_requests != cli.requests {
        eprintln!(
            "Warning: Request count mismatch! Expected {}, but sent {}",
            cli.requests, total_requests
        );
        return Err(format!(
            "Request count mismatch: expected {} but sent {}",
            cli.requests, total_requests
        )
        .into());
    }

    Ok(())
}
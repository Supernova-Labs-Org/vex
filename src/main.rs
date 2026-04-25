use clap::Parser;
use std::sync::Arc;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use futures::stream::FuturesUnordered;

pub mod client;
pub mod utils;

use client::ErrorStats;
use utils::{percentile, is_success_status, parse_target, resolve_target};

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

    // Parse target into bare hostname and effective port.
    // server_name is used for SNI (must be host-only, no port).
    // authority is used for the HTTP/3 :authority header (host:port when non-default).
    // peer_addr is resolved once here and shared across all workers/slots.
    let (server_name, effective_port) = parse_target(&cli.target, cli.port)?;
    let authority = if effective_port == 443 {
        server_name.clone()
    } else {
        format!("{}:{}", server_name, effective_port)
    };
    let peer_addr = resolve_target(&cli.target, cli.port)?;

    if cli.concurrency == 0 {
        eprintln!("concurrency must be at least 1");
        std::process::exit(1);
    }

    println!("Starting HTTP/3 load test:");
    println!("  Target: {}:{}", cli.target, effective_port);
    println!("  Host: {}", authority);
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
        let server_name = server_name.clone();
        let authority = authority.clone();
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

            // One Http3Client per worker: all concurrent streams share one QUIC
            // connection. On connection failure we reconnect and keep going.
            let mut h3 = match client::h3_client::Http3Client::new(insecure, peer_addr) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Worker {worker_id}: init failed: {e}");
                    return (0, 1, ErrorStats::default(), HashMap::new(), Vec::new());
                }
            };

            // Establish the connection before dispatching any streams.
            if let Err(e) = h3.ensure_connected(&server_name).await {
                eprintln!("Worker {worker_id}: connect failed: {e}");
                return (0, 1, ErrorStats::default(), HashMap::new(), Vec::new());
            }

            let mut dispatched: usize = 0;

            // Receivers for the currently in-flight streams.
            // We use FuturesUnordered to await them concurrently while also
            // driving the shared poll loop.
            let mut pending: FuturesUnordered<
                tokio::task::JoinHandle<(bool, Result<client::ResponseResult, String>)>
            > = FuturesUnordered::new();

            // Seed up to `concurrency` streams.
            for _ in 0..concurrency {
                if Instant::now() >= *deadline || dispatched >= requests_per_worker { break; }
                dispatched += 1;

                match h3.dispatch(&authority, &path, verbose) {
                    Ok((_sid, rx)) => {
                        let success_status = success_status.clone();
                        pending.push(tokio::spawn(async move {
                            let result = rx.await
                                .unwrap_or_else(|_| Err("channel closed".into()));
                            let ok = result.as_ref()
                                .map(|r| is_success_status(r.status_code, &success_status))
                                .unwrap_or(false);
                            (ok, result)
                        }));
                    }
                    Err(e) => {
                        eprintln!("Worker {worker_id}: dispatch failed: {e}");
                        fail += 1;
                    }
                }
            }

            // Drive the connection and collect results. We select between the
            // QUIC poll loop and the first completed receiver so neither starves
            // the other. poll_once() has an internal timeout so it yields
            // promptly even when no packets arrive.
            use futures::stream::StreamExt as _;
            loop {
                if pending.is_empty() {
                    break;
                }

                tokio::select! {
                    // Drive QUIC I/O and H3 event dispatch.
                    // When the connection dies, poll_once drains any in-flight
                    // streams with a "connection replaced" error so their tasks
                    // complete and flow through pending.next() below.
                    _alive = h3.poll_once(), if h3.has_in_flight() => {}
                    // A stream receiver resolved.
                    Some(join_result) = pending.next() => {
                        let (ok, result) = match join_result {
                            Ok(pair) => pair,
                            Err(e) => {
                                eprintln!("Worker {worker_id}: task panicked: {e}");
                                fail += 1;
                                continue;
                            }
                        };
                        match result {
                            Ok(r) => {
                                *status_codes.entry(r.status_code).or_insert(0) += 1;
                                if ok { success += 1; } else { fail += 1; }
                                total_errors.send_errors += r.errors.send_errors;
                                total_errors.recv_errors += r.errors.recv_errors;
                                total_errors.quic_errors += r.errors.quic_errors;
                                total_errors.stream_reset_errors += r.errors.stream_reset_errors;
                                latencies.push(r.latency_ms);
                            }
                            Err(ref e) if e.contains("connection replaced") => {
                                // Stream was killed when the connection closed mid-flight.
                                // Undo the dispatch count so the backfill below re-queues it.
                                dispatched = dispatched.saturating_sub(1);
                            }
                            Err(e) => {
                                eprintln!("Worker {worker_id}: request failed: {e}");
                                fail += 1;
                            }
                        }

                        // Backfill: keep concurrency slots full.
                        if Instant::now() < *deadline && dispatched < requests_per_worker {
                            dispatched += 1;
                            if !h3.is_connected() {
                                if let Err(e) = h3.ensure_connected(&server_name).await {
                                    eprintln!("Worker {worker_id}: reconnect failed: {e}");
                                    fail += 1;
                                    continue;
                                }
                            }
                            match h3.dispatch(&authority, &path, verbose) {
                                Ok((_sid, rx)) => {
                                    let success_status = success_status.clone();
                                    pending.push(tokio::spawn(async move {
                                        let result = rx.await
                                            .unwrap_or_else(|_| Err("channel closed".into()));
                                        let ok = result.as_ref()
                                            .map(|r| is_success_status(r.status_code, &success_status))
                                            .unwrap_or(false);
                                        (ok, result)
                                    }));
                                }
                                Err(e) => {
                                    eprintln!("Worker {worker_id}: dispatch failed: {e}");
                                    fail += 1;
                                }
                            }
                        }
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
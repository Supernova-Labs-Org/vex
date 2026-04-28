use clap::Parser;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

pub mod client;
pub mod utils;

use client::{ErrorStats, ResponseResult};
use utils::{
    is_success_status, parse_target, percentile, resolve_target, sni_server_name,
    validate_success_pattern,
};

const MAX_DISPATCH_ATTEMPTS: usize = 5;
const DISPATCH_RETRY_BACKOFF_MS: u64 = 5;

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

    #[arg(
        long,
        default_value = "2xx",
        help = "HTTP status codes to consider as success (e.g., '2xx', '2xx,3xx', or specific codes '200,201,301')"
    )]
    success_status: String,

    #[arg(
        short = 'c',
        long,
        default_value = "1",
        help = "Number of concurrent in-flight requests per worker"
    )]
    concurrency: usize,

    #[arg(
        long,
        default_value = "5000",
        help = "Per-request response timeout in milliseconds"
    )]
    request_timeout_ms: u64,

    #[arg(
        long,
        default_value = "5000",
        help = "Connection handshake timeout in milliseconds"
    )]
    connect_timeout_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompletionReason {
    AllRequestsCompleted,
    DurationLimitReached,
}

enum PendingOutcome {
    Completed(Result<ResponseResult, String>),
    TimedOut,
}

fn requests_for_worker(total_requests: usize, workers: usize, worker_id: usize) -> usize {
    let quotient = total_requests / workers;
    let remainder = total_requests % workers;
    quotient + if worker_id < remainder { 1 } else { 0 }
}

fn spawn_stream_waiter(
    join_set: &mut tokio::task::JoinSet<(u64, PendingOutcome)>,
    stream_id: u64,
    rx: oneshot::Receiver<Result<ResponseResult, String>>,
    request_timeout: Duration,
) {
    join_set.spawn(async move {
        match tokio::time::timeout(request_timeout, rx).await {
            Ok(Ok(result)) => (stream_id, PendingOutcome::Completed(result)),
            Ok(Err(_)) => (
                stream_id,
                PendingOutcome::Completed(Err("channel closed".into())),
            ),
            Err(_) => (stream_id, PendingOutcome::TimedOut),
        }
    });
}

async fn dispatch_with_retry(
    h3: &mut client::h3_client::Http3Client,
    server_name: &str,
    authority: &str,
    path: &str,
    verbose: bool,
    connect_timeout: Duration,
) -> Result<(u64, oneshot::Receiver<Result<ResponseResult, String>>), String> {
    for attempt in 1..=MAX_DISPATCH_ATTEMPTS {
        if !h3.is_connected() {
            h3.ensure_connected(server_name, connect_timeout)
                .await
                .map_err(|e| format!("reconnect failed: {e}"))?;
        }

        match h3.dispatch(authority, path, verbose) {
            Ok(pair) => return Ok(pair),
            Err(err) if err.is_retryable() && attempt < MAX_DISPATCH_ATTEMPTS => {
                let _ = h3.poll_once().await;
                tokio::time::sleep(Duration::from_millis(DISPATCH_RETRY_BACKOFF_MS)).await;
            }
            Err(err) => {
                return Err(format!(
                    "dispatch failed after {} attempts: {}",
                    attempt, err
                ));
            }
        }
    }

    Err("dispatch failed unexpectedly".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.workers == 0 {
        eprintln!("workers must be at least 1");
        std::process::exit(1);
    }

    if cli.concurrency == 0 {
        eprintln!("concurrency must be at least 1");
        std::process::exit(1);
    }

    if !cli.path.starts_with('/') {
        eprintln!("path must start with '/' (got: {})", cli.path);
        std::process::exit(1);
    }

    if let Err(err) = validate_success_pattern(&cli.success_status) {
        eprintln!("invalid --success-status pattern: {}", err);
        std::process::exit(1);
    }

    // Parse target into host and effective port.
    // host_for_authority is used for the HTTP/3 :authority header.
    // server_name is used for TLS SNI and must be bracket-free for IPv6.
    let (host_for_authority, effective_port) = parse_target(&cli.target, cli.port)?;
    let server_name = sni_server_name(&host_for_authority).to_string();
    let authority = if effective_port == 443 {
        host_for_authority.clone()
    } else {
        format!("{}:{}", host_for_authority, effective_port)
    };
    let peer_addr = resolve_target(&cli.target, cli.port)?;

    println!("Starting HTTP/3 load test:");
    println!("  Target: {}:{}", cli.target, effective_port);
    println!("  Host: {}", authority);
    println!("  Path: {}", cli.path);
    println!("  Workers: {}", cli.workers);
    println!("  Concurrency per worker: {}", cli.concurrency);
    println!("  Total requests: {}", cli.requests);
    println!("  Duration: {}s", cli.duration);
    println!("  Request timeout: {}ms", cli.request_timeout_ms);
    println!("  Connect timeout: {}ms", cli.connect_timeout_ms);
    println!("  Insecure: {}", cli.insecure);
    if cli.verbose {
        println!("  Verbose: enabled");
    }
    println!();

    let start_time = Instant::now();
    let deadline = Arc::new(start_time + Duration::from_secs(cli.duration));
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    let mut timed_out_requests = 0usize;
    let mut total_errors = ErrorStats::default();
    let mut status_code_counts: HashMap<u16, usize> = HashMap::new();
    let mut worker_failures = 0;
    let mut all_latencies = Vec::new();
    let mut hit_duration_limit = false;

    let mut handles = vec![];

    for worker_id in 0..cli.workers {
        let server_name = server_name.clone();
        let authority = authority.clone();
        let path = cli.path.clone();
        let insecure = cli.insecure;
        let verbose = cli.verbose;
        let success_status = cli.success_status.clone();
        let requests_per_worker = requests_for_worker(cli.requests, cli.workers, worker_id);
        let concurrency = cli.concurrency;
        let request_timeout = Duration::from_millis(cli.request_timeout_ms);
        let connect_timeout = Duration::from_millis(cli.connect_timeout_ms);
        let deadline = Arc::clone(&deadline);

        let handle = tokio::spawn(async move {
            let mut success = 0usize;
            let mut fail = 0usize;
            let mut timed_out = 0usize;
            let mut total_errors = ErrorStats::default();
            let mut status_codes: HashMap<u16, usize> = HashMap::new();
            let mut latencies = Vec::new();
            let mut duration_limited = false;

            if requests_per_worker == 0 {
                return (
                    success,
                    fail,
                    timed_out,
                    total_errors,
                    status_codes,
                    latencies,
                    duration_limited,
                );
            }

            let mut h3 = match client::h3_client::Http3Client::new(insecure, peer_addr) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Worker {worker_id}: init failed: {e}");
                    return (
                        0,
                        requests_per_worker,
                        0,
                        ErrorStats::default(),
                        HashMap::new(),
                        Vec::new(),
                        false,
                    );
                }
            };

            if let Err(e) = h3.ensure_connected(&server_name, connect_timeout).await {
                eprintln!("Worker {worker_id}: connect failed: {e}");
                return (
                    0,
                    requests_per_worker,
                    0,
                    ErrorStats::default(),
                    HashMap::new(),
                    Vec::new(),
                    false,
                );
            }

            let mut dispatched = 0usize;
            let mut pending: tokio::task::JoinSet<(u64, PendingOutcome)> =
                tokio::task::JoinSet::new();
            let deadline_instant = tokio::time::Instant::from_std(*deadline);

            'worker: loop {
                while pending.len() < concurrency
                    && Instant::now() < *deadline
                    && dispatched < requests_per_worker
                {
                    dispatched += 1;
                    match dispatch_with_retry(
                        &mut h3,
                        &server_name,
                        &authority,
                        &path,
                        verbose,
                        connect_timeout,
                    )
                    .await
                    {
                        Ok((stream_id, rx)) => {
                            spawn_stream_waiter(&mut pending, stream_id, rx, request_timeout);
                        }
                        Err(e) => {
                            eprintln!("Worker {worker_id}: {e}");
                            fail += 1;
                        }
                    }
                }

                if pending.is_empty() {
                    if dispatched < requests_per_worker && Instant::now() >= *deadline {
                        duration_limited = true;
                    }
                    break;
                }

                tokio::select! {
                    _ = tokio::time::sleep_until(deadline_instant) => {
                        // Hard-cancel all in-flight stream waiters so they don't
                        // continue running detached after the duration deadline.
                        pending.abort_all();
                        duration_limited = true;
                        break 'worker;
                    }
                    _ = h3.poll_once(), if h3.has_in_flight() => {}
                    Some(join_result) = pending.join_next() => {
                        let (stream_id, outcome) = match join_result {
                            Ok(pair) => pair,
                            Err(e) if e.is_cancelled() => continue,
                            Err(e) => {
                                eprintln!("Worker {worker_id}: task panicked: {e}");
                                fail += 1;
                                continue;
                            }
                        };

                        match outcome {
                            PendingOutcome::TimedOut => {
                                h3.abandon_stream(stream_id);
                                timed_out += 1;
                                fail += 1;
                            }
                            PendingOutcome::Completed(result) => {
                                match result {
                                    Ok(r) => {
                                        *status_codes.entry(r.status_code).or_insert(0) += 1;
                                        if is_success_status(r.status_code, &success_status) {
                                            success += 1;
                                        } else {
                                            fail += 1;
                                        }
                                        total_errors.send_errors += r.errors.send_errors;
                                        total_errors.recv_errors += r.errors.recv_errors;
                                        total_errors.quic_errors += r.errors.quic_errors;
                                        total_errors.stream_reset_errors += r.errors.stream_reset_errors;
                                        latencies.push(r.latency_ms);
                                    }
                                    Err(ref e) if e.contains("connection replaced") => {
                                        dispatched = dispatched.saturating_sub(1);
                                    }
                                    Err(e) => {
                                        eprintln!("Worker {worker_id}: request failed: {e}");
                                        fail += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            (
                success,
                fail,
                timed_out,
                total_errors,
                status_codes,
                latencies,
                duration_limited,
            )
        });

        handles.push(handle);
    }

    for (worker_id, handle) in handles.into_iter().enumerate() {
        match handle.await {
            Ok((s, f, t, errors, status_codes, latencies, duration_limited)) => {
                total_requests += s + f;
                successful_requests += s;
                failed_requests += f;
                timed_out_requests += t;
                total_errors.send_errors += errors.send_errors;
                total_errors.recv_errors += errors.recv_errors;
                total_errors.quic_errors += errors.quic_errors;
                total_errors.stream_reset_errors += errors.stream_reset_errors;
                hit_duration_limit |= duration_limited;

                for (code, count) in status_codes {
                    *status_code_counts.entry(code).or_insert(0) += count;
                }

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
    if total_requests < cli.requests {
        hit_duration_limit = true;
    }

    let completion_reason = if hit_duration_limit {
        CompletionReason::DurationLimitReached
    } else {
        CompletionReason::AllRequestsCompleted
    };

    // For throughput, use the configured duration as the denominator when
    // duration-limited so that in-flight work draining past the deadline
    // does not deflate the reported req/s.
    let throughput_window = if hit_duration_limit {
        cli.duration as f64
    } else {
        elapsed
    };

    println!("\nLoad test completed:");
    println!("  Total time: {:.2}s", elapsed);
    println!("  Total requests: {}", total_requests);
    println!("  Successful requests: {}", successful_requests);
    println!("  Failed requests: {}", failed_requests);
    if timed_out_requests > 0 {
        println!(
            "  Timed out requests: {} (>{}ms)",
            timed_out_requests, cli.request_timeout_ms
        );
    }
    println!(
        "  Requests/sec: {:.2}",
        if throughput_window > 0.0 {
            total_requests as f64 / throughput_window
        } else {
            0.0
        }
    );

    match completion_reason {
        CompletionReason::DurationLimitReached => {
            println!(
                "  Completion reason: Duration limit ({:.0}s) reached (actual: {:.2}s)",
                cli.duration, elapsed
            );
        }
        CompletionReason::AllRequestsCompleted => {
            println!(
                "  Completion reason: All {} requests completed",
                cli.requests
            );
        }
    }

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
            println!(
                "  Stream reset errors: {}",
                total_errors.stream_reset_errors
            );
        }
    }

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

    if !all_latencies.is_empty() {
        println!("\nLatency metrics (ms):");

        let mut sorted_latencies = all_latencies;
        sorted_latencies.sort_by(|a, b| a.total_cmp(b));

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

    if worker_failures > 0 {
        eprintln!(
            "\nWarning: {} worker(s) failed or panicked",
            worker_failures
        );
        eprintln!(
            "This may indicate system instability or resource exhaustion during the load test."
        );
        return Err(format!("{} worker failure(s) detected", worker_failures).into());
    }

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

#[cfg(test)]
mod tests {
    use super::requests_for_worker;

    #[test]
    fn distributes_requests_evenly_across_workers() {
        assert_eq!(requests_for_worker(10, 3, 0), 4);
        assert_eq!(requests_for_worker(10, 3, 1), 3);
        assert_eq!(requests_for_worker(10, 3, 2), 3);
    }

    #[test]
    fn zero_assigned_when_workers_exceed_requests() {
        assert_eq!(requests_for_worker(2, 4, 0), 1);
        assert_eq!(requests_for_worker(2, 4, 1), 1);
        assert_eq!(requests_for_worker(2, 4, 2), 0);
        assert_eq!(requests_for_worker(2, 4, 3), 0);
    }
}

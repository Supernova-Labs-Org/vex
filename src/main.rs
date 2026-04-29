use clap::{Parser, ValueEnum};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

pub mod client;
pub mod utils;

use client::{ErrorStats, RequestError, RequestErrorKind, ResponseResult};
use utils::{
    is_success_status, parse_target, percentile, resolve_target, sni_server_name,
    validate_success_pattern,
};

const MAX_DISPATCH_ATTEMPTS: usize = 5;
const DISPATCH_RETRY_BACKOFF_MS: u64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum StopPolicy {
    HardCutoff,
    GracefulDrain,
}

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

    #[arg(long, value_enum, default_value = "hard-cutoff")]
    stop_policy: StopPolicy,

    #[arg(
        long,
        default_value = "1000",
        help = "Additional drain window (ms) after duration when using graceful-drain"
    )]
    drain_grace_ms: u64,

    #[arg(long, default_value = "false", help = "Emit results as JSON to stdout")]
    json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompletionReason {
    AllRequestsCompleted,
    DurationLimitReached,
}

impl CompletionReason {
    fn as_str(self) -> &'static str {
        match self {
            CompletionReason::AllRequestsCompleted => "all_requests_completed",
            CompletionReason::DurationLimitReached => "duration_limit_reached",
        }
    }
}

#[derive(Debug)]
enum DispatchAttemptError {
    ReconnectFailed(String),
    DispatchFailed(client::h3_client::DispatchError),
}

struct DispatchSuccess {
    stream_id: u64,
    rx: oneshot::Receiver<Result<ResponseResult, RequestError>>,
    reconnect_handshake_ms: Option<f64>,
}

#[derive(Default)]
struct WorkerResult {
    success: usize,
    fail: usize,
    timed_out: usize,
    deadline_aborted: usize,
    errors: ErrorStats,
    status_codes: HashMap<u16, usize>,
    success_latencies: Vec<f64>,
    failure_latencies: Vec<f64>,
    conn_attempts: usize,
    conn_failures: usize,
    handshake_latencies: Vec<f64>,
    peak_concurrency: usize,
    duration_limited: bool,
    drain_started: bool,
    drain_completed: bool,
}

#[derive(Serialize)]
struct Report {
    target: String,
    workers: usize,
    concurrency: usize,
    total_time_s: f64,
    duration_limited: bool,
    completion_reason: String,
    stop_policy: String,
    drain: DrainReport,
    requests: RequestCounts,
    throughput: ThroughputReport,
    errors: ErrorReport,
    status_codes: HashMap<u16, usize>,
    latency_success_ms: Option<LatencyReport>,
    latency_failure_ms: Option<LatencyReport>,
    connections: ConnectionReport,
}

#[derive(Serialize)]
struct DrainReport {
    started: bool,
    completed: bool,
}

#[derive(Serialize)]
struct RequestCounts {
    total: usize,
    successful: usize,
    failed: usize,
    timed_out: usize,
    deadline_aborted: usize,
}

#[derive(Serialize)]
struct ThroughputReport {
    rps_in_duration: f64,
    rps_end_to_end: f64,
}

#[derive(Serialize)]
struct ErrorReport {
    send: usize,
    recv: usize,
    quic: usize,
    stream_reset: usize,
}

#[derive(Serialize)]
struct ConnectionReport {
    attempts: usize,
    failures: usize,
    peak_stream_concurrency: usize,
    handshake_latency_ms: Option<LatencyReport>,
}

#[derive(Serialize)]
struct LatencyReport {
    min: f64,
    max: f64,
    avg: f64,
    p50: f64,
    p90: f64,
    p95: f64,
    p99: f64,
}

fn requests_for_worker(total_requests: usize, workers: usize, worker_id: usize) -> usize {
    let quotient = total_requests / workers;
    let remainder = total_requests % workers;
    quotient + if worker_id < remainder { 1 } else { 0 }
}

fn spawn_stream_waiter(
    join_set: &mut tokio::task::JoinSet<(u64, Result<ResponseResult, RequestError>)>,
    stream_id: u64,
    rx: oneshot::Receiver<Result<ResponseResult, RequestError>>,
    request_timeout: Duration,
) {
    join_set.spawn(async move {
        match tokio::time::timeout(request_timeout, rx).await {
            Ok(Ok(result)) => (stream_id, result),
            Ok(Err(_)) => (
                stream_id,
                Err(RequestError::new(
                    RequestErrorKind::ChannelClosed,
                    "response channel closed unexpectedly",
                )
                .with_stream_id(stream_id)),
            ),
            Err(_) => (
                stream_id,
                Err(
                    RequestError::new(RequestErrorKind::TimedOut, "request timed out")
                        .with_stream_id(stream_id),
                ),
            ),
        }
    });
}

/// Returns the dispatched (stream_id, rx) pair plus any handshake latency
/// recorded during reconnects (None if no reconnect was needed).
async fn dispatch_with_retry(
    h3: &mut client::h3_client::Http3Client,
    server_name: &str,
    authority: &str,
    path: &str,
    verbose: bool,
    connect_timeout: Duration,
) -> Result<DispatchSuccess, DispatchAttemptError> {
    let mut reconnect_hs_ms: Option<f64> = None;

    for attempt in 1..=MAX_DISPATCH_ATTEMPTS {
        if !h3.is_connected() {
            match h3.ensure_connected(server_name, connect_timeout).await {
                Ok(hs) => reconnect_hs_ms = hs,
                Err(e) => return Err(DispatchAttemptError::ReconnectFailed(e.to_string())),
            }
        }

        match h3.dispatch(authority, path, verbose) {
            Ok((stream_id, rx)) => {
                return Ok(DispatchSuccess {
                    stream_id,
                    rx,
                    reconnect_handshake_ms: reconnect_hs_ms,
                });
            }
            Err(err) if err.is_retryable() && attempt < MAX_DISPATCH_ATTEMPTS => {
                let _ = h3.poll_once().await;
                tokio::time::sleep(Duration::from_millis(DISPATCH_RETRY_BACKOFF_MS)).await;
            }
            Err(err) => return Err(DispatchAttemptError::DispatchFailed(err)),
        }
    }

    Err(DispatchAttemptError::ReconnectFailed(
        "dispatch retries exhausted".to_string(),
    ))
}

fn print_latency_block(label: &str, sorted: &[f64]) {
    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    let avg = sorted.iter().sum::<f64>() / sorted.len() as f64;
    println!("\n{label}:");
    println!("  Min:  {:.2}", min);
    println!("  Max:  {:.2}", max);
    println!("  Avg:  {:.2}", avg);
    println!("  p50:  {:.2}", percentile(sorted, 50.0));
    println!("  p90:  {:.2}", percentile(sorted, 90.0));
    println!("  p95:  {:.2}", percentile(sorted, 95.0));
    println!("  p99:  {:.2}", percentile(sorted, 99.0));
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

    let (host_for_authority, effective_port) = parse_target(&cli.target, cli.port)?;
    let server_name = sni_server_name(&host_for_authority).to_string();
    let authority = if effective_port == 443 {
        host_for_authority.clone()
    } else {
        format!("{}:{}", host_for_authority, effective_port)
    };
    let peer_addr = resolve_target(&cli.target, cli.port)?;

    if !cli.json {
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
        println!("  Stop policy: {:?}", cli.stop_policy);
        println!("  Drain grace: {}ms", cli.drain_grace_ms);
        println!("  Insecure: {}", cli.insecure);
        if cli.verbose {
            println!("  Verbose: enabled");
        }
        println!();
    }

    let start_time = Instant::now();
    let deadline = Arc::new(start_time + Duration::from_secs(cli.duration));
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    let mut timed_out_requests = 0usize;
    let mut deadline_aborted_requests = 0usize;
    let mut total_errors = ErrorStats::default();
    let mut status_code_counts: HashMap<u16, usize> = HashMap::new();
    let mut worker_failures = 0;
    let mut success_latencies: Vec<f64> = Vec::new();
    let mut failure_latencies: Vec<f64> = Vec::new();
    let mut conn_attempts = 0usize;
    let mut conn_failures = 0usize;
    let mut handshake_latencies: Vec<f64> = Vec::new();
    let mut peak_concurrency = 0usize;
    let mut hit_duration_limit = false;
    let mut drain_started = false;
    let mut drain_completed = true;

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
        let stop_policy = cli.stop_policy;
        let drain_grace = Duration::from_millis(cli.drain_grace_ms);
        let deadline = Arc::clone(&deadline);

        let handle = tokio::spawn(async move {
            let mut result = WorkerResult::default();

            if requests_per_worker == 0 {
                return result;
            }

            let mut h3 = match client::h3_client::Http3Client::new(insecure, peer_addr) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Worker {worker_id}: init failed: {e}");
                    result.fail = requests_per_worker;
                    return result;
                }
            };

            result.conn_attempts += 1;
            match h3.ensure_connected(&server_name, connect_timeout).await {
                Ok(Some(ms)) => result.handshake_latencies.push(ms),
                Ok(None) => {}
                Err(e) => {
                    eprintln!("Worker {worker_id}: connect failed: {e}");
                    result.conn_failures += 1;
                    result.fail = requests_per_worker;
                    return result;
                }
            }

            let mut dispatched = 0usize;
            let mut pending: tokio::task::JoinSet<(u64, Result<ResponseResult, RequestError>)> =
                tokio::task::JoinSet::new();
            let mut in_flight_streams: HashSet<u64> = HashSet::new();
            let deadline_instant = tokio::time::Instant::from_std(*deadline);
            let mut deadline_reached = false;
            let mut drain_deadline: Option<tokio::time::Instant> = None;

            loop {
                while !deadline_reached
                    && pending.len() < concurrency
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
                        Ok(success) => {
                            if let Some(ms) = success.reconnect_handshake_ms {
                                result.conn_attempts += 1;
                                result.handshake_latencies.push(ms);
                            }

                            in_flight_streams.insert(success.stream_id);
                            spawn_stream_waiter(
                                &mut pending,
                                success.stream_id,
                                success.rx,
                                request_timeout,
                            );
                            result.peak_concurrency = result.peak_concurrency.max(pending.len());
                        }
                        Err(DispatchAttemptError::ReconnectFailed(e)) => {
                            eprintln!("Worker {worker_id}: reconnect failed: {e}");
                            result.conn_attempts += 1;
                            result.conn_failures += 1;
                            result.fail += 1;
                        }
                        Err(DispatchAttemptError::DispatchFailed(e)) => {
                            eprintln!("Worker {worker_id}: dispatch failed: {e}");
                            result.fail += 1;
                        }
                    }
                }

                if pending.is_empty() {
                    if dispatched < requests_per_worker
                        && (deadline_reached || Instant::now() >= *deadline)
                    {
                        result.duration_limited = true;
                    }
                    break;
                }

                if !deadline_reached {
                    tokio::select! {
                        _ = tokio::time::sleep_until(deadline_instant) => {
                            deadline_reached = true;
                            result.duration_limited = true;
                            result.drain_started = true;
                            match stop_policy {
                                StopPolicy::HardCutoff => {
                                    let aborted = in_flight_streams.len();
                                    for stream_id in in_flight_streams.drain() {
                                        h3.abandon_stream(stream_id);
                                    }
                                    pending.abort_all();
                                    result.deadline_aborted += aborted;
                                    result.fail += aborted;
                                    result.drain_completed = aborted == 0;
                                    break;
                                }
                                StopPolicy::GracefulDrain => {
                                    drain_deadline = Some(tokio::time::Instant::now() + drain_grace);
                                }
                            }
                        }
                        _ = h3.poll_once(), if h3.has_in_flight() => {}
                        Some(join_result) = pending.join_next() => {
                            let (stream_id, req_result) = match join_result {
                                Ok(pair) => pair,
                                Err(e) if e.is_cancelled() => continue,
                                Err(e) => {
                                    eprintln!("Worker {worker_id}: task panicked: {e}");
                                    result.fail += 1;
                                    continue;
                                }
                            };
                            in_flight_streams.remove(&stream_id);

                            match req_result {
                                Ok(r) => {
                                    *result.status_codes.entry(r.status_code).or_insert(0) += 1;
                                    result.errors.send_errors += r.errors.send_errors;
                                    result.errors.recv_errors += r.errors.recv_errors;
                                    result.errors.quic_errors += r.errors.quic_errors;
                                    result.errors.stream_reset_errors += r.errors.stream_reset_errors;
                                    if is_success_status(r.status_code, &success_status) {
                                        result.success += 1;
                                        result.success_latencies.push(r.latency_ms);
                                    } else {
                                        result.fail += 1;
                                        result.failure_latencies.push(r.latency_ms);
                                    }
                                }
                                Err(err) => match err.kind {
                                    RequestErrorKind::ConnectionReplaced => {
                                        dispatched = dispatched.saturating_sub(1);
                                    }
                                    RequestErrorKind::TimedOut => {
                                        result.timed_out += 1;
                                        result.fail += 1;
                                        result.failure_latencies.push(request_timeout.as_secs_f64() * 1000.0);
                                    }
                                    _ => {
                                        eprintln!("Worker {worker_id}: request failed: {err}");
                                        result.fail += 1;
                                    }
                                },
                            }
                        }
                    }
                } else {
                    let drain_wait_until = drain_deadline.unwrap_or_else(tokio::time::Instant::now);

                    tokio::select! {
                        _ = tokio::time::sleep_until(drain_wait_until) => {
                            let aborted = in_flight_streams.len();
                            for stream_id in in_flight_streams.drain() {
                                h3.abandon_stream(stream_id);
                            }
                            pending.abort_all();
                            result.deadline_aborted += aborted;
                            result.fail += aborted;
                            result.drain_completed = aborted == 0;
                            break;
                        }
                        _ = h3.poll_once(), if h3.has_in_flight() => {}
                        Some(join_result) = pending.join_next() => {
                            let (stream_id, req_result) = match join_result {
                                Ok(pair) => pair,
                                Err(e) if e.is_cancelled() => continue,
                                Err(e) => {
                                    eprintln!("Worker {worker_id}: task panicked: {e}");
                                    result.fail += 1;
                                    continue;
                                }
                            };
                            in_flight_streams.remove(&stream_id);

                            match req_result {
                                Ok(r) => {
                                    *result.status_codes.entry(r.status_code).or_insert(0) += 1;
                                    result.errors.send_errors += r.errors.send_errors;
                                    result.errors.recv_errors += r.errors.recv_errors;
                                    result.errors.quic_errors += r.errors.quic_errors;
                                    result.errors.stream_reset_errors += r.errors.stream_reset_errors;
                                    if is_success_status(r.status_code, &success_status) {
                                        result.success += 1;
                                        result.success_latencies.push(r.latency_ms);
                                    } else {
                                        result.fail += 1;
                                        result.failure_latencies.push(r.latency_ms);
                                    }
                                }
                                Err(err) => match err.kind {
                                    RequestErrorKind::ConnectionReplaced => {
                                        dispatched = dispatched.saturating_sub(1);
                                    }
                                    RequestErrorKind::TimedOut => {
                                        result.timed_out += 1;
                                        result.fail += 1;
                                        result.failure_latencies.push(request_timeout.as_secs_f64() * 1000.0);
                                    }
                                    _ => {
                                        eprintln!("Worker {worker_id}: request failed: {err}");
                                        result.fail += 1;
                                    }
                                },
                            }

                            if pending.is_empty() {
                                result.drain_completed = true;
                                break;
                            }
                        }
                    }
                }
            }

            result
        });

        handles.push(handle);
    }

    for (worker_id, handle) in handles.into_iter().enumerate() {
        match handle.await {
            Ok(worker) => {
                total_requests += worker.success + worker.fail;
                successful_requests += worker.success;
                failed_requests += worker.fail;
                timed_out_requests += worker.timed_out;
                deadline_aborted_requests += worker.deadline_aborted;
                total_errors.send_errors += worker.errors.send_errors;
                total_errors.recv_errors += worker.errors.recv_errors;
                total_errors.quic_errors += worker.errors.quic_errors;
                total_errors.stream_reset_errors += worker.errors.stream_reset_errors;
                hit_duration_limit |= worker.duration_limited;
                drain_started |= worker.drain_started;
                drain_completed &= worker.drain_completed;
                conn_attempts += worker.conn_attempts;
                conn_failures += worker.conn_failures;
                handshake_latencies.extend(worker.handshake_latencies);
                peak_concurrency = peak_concurrency.max(worker.peak_concurrency);

                for (code, count) in worker.status_codes {
                    *status_code_counts.entry(code).or_insert(0) += count;
                }

                success_latencies.extend(worker.success_latencies);
                failure_latencies.extend(worker.failure_latencies);
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

    let duration_window = cli.duration as f64;

    if !cli.json {
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
        if deadline_aborted_requests > 0 {
            println!("  Deadline-aborted requests: {}", deadline_aborted_requests);
        }
        if hit_duration_limit {
            println!(
                "  Req/s (in-duration):  {:.2}",
                if duration_window > 0.0 {
                    total_requests as f64 / duration_window
                } else {
                    0.0
                }
            );
            println!(
                "  Req/s (end-to-end):   {:.2}",
                if elapsed > 0.0 {
                    total_requests as f64 / elapsed
                } else {
                    0.0
                }
            );
        } else {
            println!(
                "  Requests/sec: {:.2}",
                if elapsed > 0.0 {
                    total_requests as f64 / elapsed
                } else {
                    0.0
                }
            );
        }

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

        println!(
            "  Drain policy: {:?}, started: {}, completed: {}",
            cli.stop_policy, drain_started, drain_completed
        );
    }

    let has_errors = total_errors.send_errors > 0
        || total_errors.recv_errors > 0
        || total_errors.quic_errors > 0
        || total_errors.stream_reset_errors > 0;

    if has_errors && !cli.json {
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

    if !status_code_counts.is_empty() && !cli.json {
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

    success_latencies.sort_by(|a, b| a.total_cmp(b));
    failure_latencies.sort_by(|a, b| a.total_cmp(b));
    handshake_latencies.sort_by(|a, b| a.total_cmp(b));

    let rps_in_duration = if duration_window > 0.0 {
        total_requests as f64 / duration_window
    } else {
        0.0
    };
    let rps_end_to_end = if elapsed > 0.0 {
        total_requests as f64 / elapsed
    } else {
        0.0
    };

    if cli.json {
        let report = build_report(
            &cli.target,
            cli.workers,
            cli.concurrency,
            elapsed,
            completion_reason,
            total_requests,
            successful_requests,
            failed_requests,
            timed_out_requests,
            deadline_aborted_requests,
            rps_in_duration,
            rps_end_to_end,
            hit_duration_limit,
            cli.stop_policy,
            drain_started,
            drain_completed,
            &total_errors,
            &status_code_counts,
            &success_latencies,
            &failure_latencies,
            conn_attempts,
            conn_failures,
            handshake_latencies.as_slice(),
            peak_concurrency,
        );
        let mut stdout = std::io::stdout().lock();
        serde_json::to_writer(&mut stdout, &report)?;
        use std::io::Write as _;
        stdout.write_all(b"\n")?;
    } else {
        if !success_latencies.is_empty() {
            print_latency_block("Latency (successful requests, ms)", &success_latencies);
        }
        if !failure_latencies.is_empty() {
            print_latency_block("Latency (failed requests, ms)", &failure_latencies);
        }

        println!("\nConnection diagnostics:");
        println!("  Connection attempts:  {}", conn_attempts);
        println!("  Connection failures:  {}", conn_failures);
        println!("  Peak stream concurrency: {}", peak_concurrency);
        if !handshake_latencies.is_empty() {
            print_latency_block("Handshake latency (ms)", &handshake_latencies);
        }
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

#[allow(clippy::too_many_arguments)]
fn build_report(
    target: &str,
    workers: usize,
    concurrency: usize,
    elapsed: f64,
    completion_reason: CompletionReason,
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    timed_out_requests: usize,
    deadline_aborted_requests: usize,
    rps_in_duration: f64,
    rps_end_to_end: f64,
    hit_duration_limit: bool,
    stop_policy: StopPolicy,
    drain_started: bool,
    drain_completed: bool,
    errors: &ErrorStats,
    status_codes: &HashMap<u16, usize>,
    success_lats: &[f64],
    failure_lats: &[f64],
    conn_attempts: usize,
    conn_failures: usize,
    handshake_lats: &[f64],
    peak_concurrency: usize,
) -> Report {
    Report {
        target: target.to_string(),
        workers,
        concurrency,
        total_time_s: elapsed,
        duration_limited: hit_duration_limit,
        completion_reason: completion_reason.as_str().to_string(),
        stop_policy: match stop_policy {
            StopPolicy::HardCutoff => "hard-cutoff".to_string(),
            StopPolicy::GracefulDrain => "graceful-drain".to_string(),
        },
        drain: DrainReport {
            started: drain_started,
            completed: drain_completed,
        },
        requests: RequestCounts {
            total: total_requests,
            successful: successful_requests,
            failed: failed_requests,
            timed_out: timed_out_requests,
            deadline_aborted: deadline_aborted_requests,
        },
        throughput: ThroughputReport {
            rps_in_duration,
            rps_end_to_end,
        },
        errors: ErrorReport {
            send: errors.send_errors,
            recv: errors.recv_errors,
            quic: errors.quic_errors,
            stream_reset: errors.stream_reset_errors,
        },
        status_codes: status_codes.clone(),
        latency_success_ms: latency_report(success_lats),
        latency_failure_ms: latency_report(failure_lats),
        connections: ConnectionReport {
            attempts: conn_attempts,
            failures: conn_failures,
            peak_stream_concurrency: peak_concurrency,
            handshake_latency_ms: latency_report(handshake_lats),
        },
    }
}

fn latency_report(lats: &[f64]) -> Option<LatencyReport> {
    if lats.is_empty() {
        return None;
    }

    let min = lats[0];
    let max = lats[lats.len() - 1];
    let avg = lats.iter().sum::<f64>() / lats.len() as f64;

    Some(LatencyReport {
        min,
        max,
        avg,
        p50: percentile(lats, 50.0),
        p90: percentile(lats, 90.0),
        p95: percentile(lats, 95.0),
        p99: percentile(lats, 99.0),
    })
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

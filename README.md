# vex

A minimal load testing tool for HTTP/3 servers built in Rust.

---

## Overview

- HTTP/3 support using `quiche` for direct QUIC implementation
- Concurrent request execution using Tokio async runtime
- Comprehensive metrics collection: success/failure counts, RPS, and latency percentiles
- Self-signed certificate support for local testing
- Simple console output with detailed statistics

---

## Features

- [x] CLI interface with configurable parameters
- [x] Rich latency metrics (avg, p50, p90, p95, p99, min, max)
- [x] Error tracking and classification (send errors, recv errors, QUIC errors, stream resets)
- [x] HTTP status code validation and breakdown
- [x] Verbose mode for debugging
- [ ] Export results as JSON/CSV
- [ ] Custom headers and POST payload support
- [ ] Connection keep-alive vs fresh-per-request options
- [ ] Ramp-up traffic mode (gradual increase)
- [ ] Distributed load testing across multiple nodes

---

## Implementation

The load testing tool includes:

1. **HTTP/3 Client**
   - Uses `quiche` library for direct QUIC/HTTP3 implementation
   - Configurable QUIC parameters
   - Supports both secure and insecure (self-signed) connections
   - Async UDP socket handling with proper timeout management

2. **Concurrent Worker System**
   - Spawns multiple async tasks using `tokio::spawn`
   - Distributes total requests across workers using quotient + remainder logic
   - Each worker is assigned base requests, with first N workers getting one extra
   - Stops when either all requests are completed OR duration expires (whichever comes first)
   - Tracks worker panics and cancellations

3. **Metrics Collection**
   - Per-request latency measurement
   - Aggregation of status codes with per-code counts
   - Error categorization: network send/recv, QUIC protocol, stream resets
   - Percentile latency calculation with linear interpolation

4. **Console Reporting**
   - Total time elapsed and requests per second
   - Completion reason (all requests done or duration limit reached)
   - HTTP status code breakdown
   - Latency percentiles (min, max, avg, p50, p90, p95, p99)
   - Error summary with counts by error type
   - Worker failure warnings

---

## Usage

```bash
# Build
cargo build --release

# Run with default settings (443, localhost, 1000 requests, 1 worker)
./target/release/vex --target example.com

# Run 5000 requests with 100 concurrent workers
cargo run --release -- --target example.com --workers 100 --requests 5000

# Run for 60 seconds with 200 workers
cargo run --release -- --target example.com --workers 200 --duration 60 --insecure

# Test specific endpoint with custom host header
cargo run --release -- --target service.local --port 8443 --workers 50 --requests 1000 --path "/api/health" --host "service.local" --insecure

# Full example with all options
cargo run --release -- --protocol h3 --target example.com --port 443 --workers 100 --requests 10000 --duration 120 --path "/api/v1/test" --host "api.example.com"

# Verbose output (prints response headers)
cargo run --release -- --target example.com --workers 10 --requests 100 --verbose
```

### CLI Options

- `--protocol h3` - Protocol (currently only HTTP/3 supported, default: h3)
- `--target TARGET` - Target host or IP address (required)
- `--port PORT` - Target port (default: 443)
- `--workers N` - Number of concurrent workers (default: 1, minimum: 1)
- `--requests N` - Total number of requests to send (default: 1000)
- `--duration SECS` - Maximum duration in seconds (default: 30)
- `--path PATH` - Request path (default: /)
- `--host HOST` - Host header value (defaults to target if not specified)
- `--insecure` - Disable TLS certificate verification
- `--verbose` - Print response headers for each request

---

## Request Distribution

For N total requests across M workers:
- Each worker gets at least `N / M` requests
- The first `N % M` workers get one additional request
- This ensures all N requests are distributed exactly, with no gaps or duplicates

Example: 10 requests across 3 workers
- Worker 0: 4 requests (2 + 1 extra)
- Worker 1: 4 requests (2 + 1 extra)
- Worker 2: 2 requests (2)

---

## Duration and Request Behavior

The load test continues until one of these conditions is met:

1. All requested requests have been sent
2. The duration limit has expired
3. A worker panics or is cancelled

The completion reason is reported in the output. If the duration expires before all requests are sent, some workers may stop early.

---

## Exit Codes

- 0: Success (all tests completed without worker failures and requests matched expected count)
- 1: Worker failure or panic detected
- 1: Request count mismatch when not hitting duration limit

---

## Dependencies

- Tokio - Async runtime for concurrent request handling
- Quiche - Rust implementation of QUIC and HTTP/3 protocols
- Clap - Command line argument parser
- Rand - Random number generation for connection IDs

---

## Development

Written in Rust for performance and fine-grained control over concurrency. Uses the Tokio async runtime for efficient concurrent request handling with proper timeout and error management.
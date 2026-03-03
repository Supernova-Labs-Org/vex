# vex - HTTP/3 Load Testing Tool

A minimal load testing tool for HTTP/3 servers built in Rust.

## Features

- HTTP/3 support using `quiche` for direct QUIC implementation
- Concurrent request execution using Tokio async runtime
- Comprehensive metrics: success/failure counts, RPS, and latency percentiles
- Self-signed certificate support for local testing
- Rich latency metrics: min, max, avg, p50, p90, p95, p99
- Error tracking and classification
- Verbose mode for debugging

## Quick Start

```bash
# Build
cargo build --release

# Run a basic test
./target/release/vex --target example.com --workers 100 --requests 5000
```

## Documentation

- [Getting Started](GETTING_STARTED.md) - Installation and first test
- [CLI Reference](CLI_REFERENCE.md) - All available options
- [Examples](EXAMPLES.md) - Real-world usage patterns
- [Metrics](METRICS.md) - Understanding results

## Basic Usage

Test a server with 100 concurrent workers:

```bash
vex --target example.com --workers 100 --requests 5000
```

Test for 60 seconds:

```bash
vex --target example.com --workers 100 --duration 60
```

## Key Concepts

### Request Distribution
Requests are distributed evenly across workers. If 1000 requests are sent to 3 workers:
- Worker 0: 334 requests
- Worker 1: 333 requests
- Worker 2: 333 requests

### Duration Behavior
Tests stop when either:
1. All requests are completed, OR
2. Duration expires

Whichever happens first.

### Metrics
Every test reports:
- **Throughput**: Requests per second
- **Latency**: Min, max, average, and percentiles (p50, p90, p95, p99)
- **Status codes**: HTTP response distribution
- **Errors**: Categorized by type (network, QUIC, stream reset)

## Next Steps

- Read [Getting Started](GETTING_STARTED.md) for detailed setup
- Check [CLI Reference](CLI_REFERENCE.md) for all options
- See [Examples](EXAMPLES.md) for common scenarios

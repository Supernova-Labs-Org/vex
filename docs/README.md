# Documentation

This directory contains documentation for vex, a minimal HTTP/3 load testing tool.

## Quick Links

- **[Getting Started](GETTING_STARTED.md)** - Installation and first load test
- **[CLI Reference](CLI_REFERENCE.md)** - Complete option documentation
- **[Examples](EXAMPLES.md)** - Real-world usage patterns
- **[Metrics](METRICS.md)** - Understanding output and interpreting results

## Documentation Overview

### GETTING_STARTED.md

Quick start guide covering:
- Installation
- Running your first test
- Common scenarios
- Understanding output

Start here if you're new to vex.

### CLI_REFERENCE.md

Detailed reference for all CLI options:
- Required options
- Load configuration
- Connection settings
- Output control

Use this to understand what each flag does.

### EXAMPLES.md

Practical examples for:
- Baseline testing
- Concurrency testing
- Time-based testing
- Endpoint testing
- Local service testing
- Debugging
- Performance comparison
- High throughput testing

Covers common use cases and testing strategies.

### METRICS.md

Detailed explanation of:
- Summary metrics (throughput, request counts)
- Status code breakdown
- Latency percentiles and interpretation
- Error categories
- Common patterns and analysis

Use this to understand what the metrics mean and how to interpret results.

## Architecture

The vex tool consists of:

- **HTTP/3 Client**: QUIC-based HTTP/3 implementation using quiche
- **Worker System**: Async tasks distributing requests across concurrent workers
- **Metrics Collection**: Per-request latency tracking and error categorization
- **Result Reporting**: Console output with aggregated statistics

See the main README.md in the project root for architectural overview.

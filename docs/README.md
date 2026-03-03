# vex Documentation

Comprehensive guides for using vex, an HTTP/3 load testing tool.

## Getting Started

New to vex? Start here:

- **[Getting Started](GETTING_STARTED.md)** - Installation and first load test
- **[CLI Reference](CLI_REFERENCE.md)** - Complete option documentation
- **[Examples](EXAMPLES.md)** - Real-world usage patterns
- **[Metrics](METRICS.md)** - Understanding output and interpreting results

## Documentation Sections

### Getting Started
Quick start guide covering installation, running your first test, common scenarios, and understanding output.

### CLI Reference
Detailed reference for all CLI options including required arguments, load configuration, connection settings, and output control.

### Examples
Practical examples for baseline testing, concurrency testing, duration-based testing, endpoint testing, and performance comparison.

### Metrics
Detailed explanation of summary metrics, status code breakdown, latency percentiles, error categories, and analysis patterns.

## Building Documentation

Install dependencies:
```bash
pip install -r docs-requirements.txt
```

Serve documentation locally:
```bash
mkdocs serve
```

Then visit `http://localhost:8000` in your browser.

Build static site:
```bash
mkdocs build
```

Output will be in the `site/` directory.

## Project Structure

- `HTTP/3 Client`: QUIC-based implementation using quiche
- `Worker System`: Async tasks distributing requests across concurrent workers
- `Metrics Collection`: Per-request latency tracking and error categorization
- `Result Reporting`: Console output with aggregated statistics

See the main README.md for architectural details.

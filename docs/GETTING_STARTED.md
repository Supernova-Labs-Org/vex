# Getting Started

## Installation

Clone the repository and build the project:

```bash
git clone https://github.com/yourusername/vex.git
cd vex
cargo build --release
```

The binary will be available at `./target/release/vex`.

## Your First Load Test

The simplest test requires only a target:

```bash
cargo run --release -- --target example.com
```

This sends 1000 requests with 1 worker to example.com on port 443.

## Common Scenarios

### Test a Local Service

```bash
cargo run --release -- --target 127.0.0.1 --port 8080 --insecure
```

### Run for a Fixed Duration

```bash
cargo run --release -- --target example.com --workers 50 --duration 60
```

Runs for 60 seconds with 50 concurrent workers. The test stops when the duration expires or all requests complete, whichever comes first.

### Test a Specific Endpoint

```bash
cargo run --release -- --target api.example.com --path "/v1/users" --workers 100 --requests 10000
```

## Understanding Output

After the test completes, you'll see:

- Total time and requests/second
- Success and failure counts
- HTTP status code breakdown
- Latency percentiles (min, max, avg, p50, p90, p95, p99)
- Any errors encountered by category

Example:

```
Load test completed:
  Total time: 5.42s
  Total requests: 1000
  Successful requests: 995
  Failed requests: 5
  Requests/sec: 184.50

HTTP Status code breakdown:
  200: 2xx Success (995)
  500: 5xx Server Error (5)

Latency metrics (ms):
  Min:  2.34
  Max:  156.78
  Avg:  24.56
  p50:  18.90
  p90:  45.32
  p95:  62.14
  p99:  120.45
```

## Next Steps

- See [CLI_REFERENCE.md](CLI_REFERENCE.md) for all available options
- See [EXAMPLES.md](EXAMPLES.md) for advanced usage patterns

# Metrics and Output

## Summary Metrics

### Throughput

```
Total time: 5.42s
Total requests: 1000
Requests/sec: 184.50
```

- Total time: Elapsed duration from test start to completion
- Total requests: Sum of successful + failed requests sent
- Requests/sec: Total requests divided by total time (throughput indicator)

### Request Counts

```
Successful requests: 995
Failed requests: 5
```

- **Success**: HTTP requests matching the `--success-status` criteria (default: 2xx only)
- **Failure**: Any other outcome (non-matching status codes, network errors, timeouts)

By default, only HTTP 2xx status codes (200-299) are counted as successful. You can customize this with `--success-status`:
- `--success-status 2xx` (default): Only 2xx status codes
- `--success-status 2xx,3xx`: Both 2xx and 3xx status codes
- `--success-status 200,201,301`: Specific status codes

### Completion Reason

```
Completion reason: All 1000 requests completed
```

or

```
Completion reason: Duration limit (60s) reached
```

Indicates why the test stopped:
- All requests completed: Reached the `--requests` limit
- Duration limit reached: Hit the `--duration` timeout before all requests sent

## Status Code Breakdown

```
HTTP Status code breakdown:
  200: 2xx Success (800)
  201: 2xx Success (195)
  400: 4xx Client Error (3)
  500: 5xx Server Error (2)
```

Shows count and category for each unique HTTP status code encountered.

## Latency Metrics

```
Latency metrics (ms):
  Min:   2.34    Fastest request
  Max:  156.78   Slowest request
  Avg:   24.56   Average latency
  p50:   18.90   Median (50th percentile)
  p90:   45.32   90th percentile
  p95:   62.14   95th percentile
  p99:  120.45   99th percentile
```

All values in milliseconds.

### Interpretation

- Min/Max: Range of observed latencies
- Avg: Mean latency (influenced by outliers)
- p50: Typical user experience (median)
- p90/p95: Most users see this latency or better
- p99: Worst-case for typical scenario (tail latency)

Use percentiles rather than average for performance tuning. A high p99 indicates occasional slowness that averages won't show.

## Error Breakdown

```
Error breakdown:
  Network send errors: 2
  Network recv errors: 1
  QUIC/protocol errors: 0
  Stream reset errors: 1
```

Categorized count of errors encountered:

- Send errors: Failed to transmit packet to server
- Recv errors: Failed to receive packet from server
- QUIC/protocol errors: Protocol-level issues (invalid packets, state errors)
- Stream reset errors: Server reset the HTTP/3 stream

## Worker Failures

```
Warning: 1 worker(s) failed or panicked
This may indicate system instability or resource exhaustion during the load test.
```

Indicates that one or more worker tasks failed or panicked. This reduces actual concurrency and skews metrics. Check system resources (CPU, memory, file descriptors).

## Interpreting Results

### Stable Performance

- Low standard deviation between p50 and p99
- Consistent requests/sec across runs
- No errors or worker failures

Example:
```
Avg: 25ms, p50: 23ms, p99: 28ms → Stable
```

### Performance Degradation

- High p99 compared to p50
- Increasing latency as test progresses
- Rising error counts

Example:
```
Avg: 25ms, p50: 20ms, p99: 200ms → Unstable under load
```

### Resource Exhaustion

- Worker failures or panics
- Increasing send/recv errors over time
- Connection reset errors

### Server Issues

- High 5xx error rate
- Stream reset errors
- QUIC protocol errors

## Common Patterns

### Finding Maximum Throughput

Run with increasing worker counts until p99 latency becomes unacceptable:

```bash
vex --target example.com --workers 10 --requests 10000
vex --target example.com --workers 50 --requests 10000
vex --target example.com --workers 100 --requests 10000
```

Find the worker count where p99 latency is still acceptable.

### Comparing Versions

Run identical tests against old and new versions:

```bash
vex --target v1.example.com --workers 50 --requests 5000
vex --target v2.example.com --workers 50 --requests 5000
```

Compare p50, p99, and requests/sec.

### Stability Testing

Run for extended duration with moderate load:

```bash
vex --target example.com --workers 50 --duration 600
```

Watch for increasing error counts or p99 latency drift (indicates memory leaks or resource issues).

### Spike Testing

Test behavior under brief high load:

```bash
vex --target example.com --workers 500 --duration 10
```

Measure how quickly the system recovers and if errors occur.

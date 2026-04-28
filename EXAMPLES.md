# Usage Examples

## Quick Baseline

Test a server with reasonable defaults:

```bash
vex --target example.com
```

Runs 1000 requests with 1 worker to establish baseline latency.

## Concurrency Testing

Measure performance under increasing load:

```bash
# Light load: 10 workers
vex --target example.com --workers 10 --requests 1000

# Medium load: 50 workers
vex --target example.com --workers 50 --requests 5000

# Heavy load: 200 workers
vex --target example.com --workers 200 --requests 10000
```

## Time-Based Testing

Run for a fixed duration to find sustained throughput:

```bash
# 1 minute test
vex --target example.com --workers 100 --duration 60

# 5 minute test
vex --target example.com --workers 50 --duration 300

# Long soak test
vex --target example.com --workers 25 --duration 3600
```

## Endpoint Testing

Test specific API endpoints:

```bash
# REST endpoint
vex --target api.example.com --path /v1/users --workers 50 --requests 5000

# GraphQL endpoint
vex --target api.example.com --path /graphql --workers 100 --requests 10000

# Health check endpoint
vex --target service.local --path /health --workers 10 --requests 1000
```

## Local Service Testing

Test local services with custom ports:

```bash
# Local HTTP/3 service on port 8443
vex --target 127.0.0.1 --port 8443 --insecure --workers 50

# IPv6 localhost
vex --target "[::1]" --port 8443 --insecure
```

## Debugging

Enable verbose mode to see request/response details:

```bash
# Small test with header output
vex --target example.com --workers 1 --requests 10 --verbose
```

Shows:
- HTTP status and headers for each request
- Any errors encountered

## Mixed Constraints

Use both request count and duration to explore different scenarios:

```bash
# Stop when 10000 requests complete OR 60 seconds elapse
vex --target example.com --workers 100 --requests 10000 --duration 60

# Will complete when:
# - All 10000 requests are sent, OR
# - 60 seconds have passed
# Whichever comes first
```

Check the output for "Completion reason" to see which limit was reached:

```
Completion reason: All 10000 requests completed
```

or

```
Completion reason: Duration limit (60s) reached
```

## Performance Comparison

Compare two configurations:

```bash
# Baseline
vex --target service1.example.com --workers 50 --requests 5000

# New version
vex --target service2.example.com --workers 50 --requests 5000
```

Compare the latency metrics and request/sec values.

## High Throughput Testing

Maximize throughput with many workers:

```bash
vex --target example.com \
    --workers 500 \
    --requests 100000 \
    --duration 120
```

Monitor:
- Requests/sec value
- p99 latency for tail performance
- Error counts for stability

## Diagnostic Run

Small test to verify connectivity and get baseline:

```bash
vex --target example.com \
    --workers 5 \
    --requests 100 \
    --verbose
```

Output shows:
- If connection succeeds
- Response times
- Any errors
- HTTP status distribution

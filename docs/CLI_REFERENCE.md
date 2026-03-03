# CLI Reference

## Command Structure

```
vex [OPTIONS] --target <TARGET>
```

## Required Options

### `--target <TARGET>`

The target host to test. Accepts:
- Hostname: `example.com`
- IPv4 address: `192.0.2.1`
- IPv6 address: `[2001:db8::1]`
- With port: `example.com:8443`, `[::1]:9000`

Examples:

```bash
vex --target example.com
vex --target 192.0.2.1
vex --target [2001:db8::1]:8443
```

## Load Configuration

### `--workers <N>`

Number of concurrent workers (tasks) executing requests in parallel.

- Default: 1
- Minimum: 1
- Affects concurrency level and throughput

```bash
vex --target example.com --workers 100
```

### `--requests <N>`

Total number of requests to send across all workers.

- Default: 1000
- Distributed evenly across workers using quotient + remainder logic
- If combined with `--duration`, whichever limit is reached first stops the test

```bash
vex --target example.com --requests 5000
```

### `--duration <SECS>`

Maximum time to run the load test in seconds.

- Default: 30
- Test stops when duration expires or all requests complete (whichever comes first)
- Workers may not complete their assigned requests if duration limit is reached

```bash
vex --target example.com --duration 60
```

## Connection Configuration

### `--port <PORT>`

Target port number.

- Default: 443
- Port embedded in target takes precedence over this option

```bash
vex --target example.com --port 8443
vex --target example.com:9000 --port 443  # Uses port 9000
```

### `--host <HOST>`

Host header value for the HTTP request.

- Default: Same as `--target` value
- Use when target is an IP but server expects a specific Host header

```bash
vex --target 10.0.0.5 --host api.example.com
```

### `--path <PATH>`

Request path for each HTTP request.

- Default: /
- Must start with /

```bash
vex --target example.com --path /api/v1/test
```

### `--protocol <PROTOCOL>`

HTTP protocol to use.

- Default: h3
- Currently only h3 (HTTP/3) is supported

```bash
vex --target example.com --protocol h3
```

### `--insecure`

Disable TLS certificate verification.

- Default: Disabled (certificates are verified)
- Use for self-signed certificates or testing

```bash
vex --target localhost --port 8443 --insecure
```

## Output Configuration

### `--verbose`

Enable verbose output.

- Default: Disabled
- Prints response headers for each request (may reduce throughput)
- Useful for debugging

```bash
vex --target example.com --verbose
```

## Full Example

All options combined:

```bash
vex --protocol h3 \
    --target api.example.com \
    --port 8443 \
    --workers 50 \
    --requests 5000 \
    --duration 120 \
    --path /api/v2/test \
    --host api.example.com \
    --insecure \
    --verbose
```

## Short Options

Some options support short forms:

- `-n <N>` for `--requests`
- `-w <N>` for `--workers`
- `-t <SECS>` for `--duration`

Example:

```bash
vex --target example.com -w 100 -n 5000 -t 60
```

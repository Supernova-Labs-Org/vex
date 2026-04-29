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

### `--path <PATH>`

Request path for each HTTP request.

- Default: /
- Must start with /

```bash
vex --target example.com --path /api/v1/test
```

### `--insecure`

Disable TLS certificate verification.

- Default: Disabled (certificates are verified)
- Use for self-signed certificates or testing

```bash
vex --target localhost --port 8443 --insecure
```

### `--request-timeout-ms <MILLISECONDS>`

Per-request completion timeout.

- Default: 5000
- Applies to each in-flight request
- Timed-out requests are counted as failed and timed out

```bash
vex --target example.com --request-timeout-ms 2000
```

### `--connect-timeout-ms <MILLISECONDS>`

Handshake/connect timeout for connection establishment and reconnects.

- Default: 5000
- Affects initial connect and reconnect attempts

```bash
vex --target example.com --connect-timeout-ms 3000
```

### `--stop-policy <hard-cutoff|graceful-drain>`

Behavior after `--duration` is reached.

- `hard-cutoff` (default): immediately abort in-flight requests
- `graceful-drain`: stop new dispatches, allow in-flight requests to finish during drain grace window

```bash
vex --target example.com --duration 30 --stop-policy graceful-drain
```

### `--drain-grace-ms <MILLISECONDS>`

Additional drain window used by `--stop-policy graceful-drain`.

- Default: 1000
- Ignored when `--stop-policy hard-cutoff`

```bash
vex --target example.com --duration 30 --stop-policy graceful-drain --drain-grace-ms 5000
```

## Output Configuration

### `--success-status <PATTERN>`

Define which HTTP status codes count as successful requests.

- Default: `2xx` (HTTP 200-299 only)
- Pattern syntax supports class (2xx, 3xx, 4xx, 5xx) or specific codes (comma-separated)

Examples:

```bash
# Default: only 2xx counts as success
vex --target example.com

# Count both 2xx and 3xx as success
vex --target example.com --success-status 2xx,3xx

# Count specific status codes as success
vex --target example.com --success-status 200,201,204,301,302
```

This affects the "Successful/Failed requests" counts in the output.
Invalid tokens (for example `2xy` or `700`) are rejected at startup.

### `--verbose`

Enable verbose output.

- Default: Disabled
- Prints response headers for each request (may reduce throughput)
- Useful for debugging

```bash
vex --target example.com --verbose
```

### `--json`

Emit machine-readable JSON to stdout.

- Default: Disabled
- stdout contains JSON only
- Human-readable logs remain disabled on stdout in this mode

```bash
vex --target example.com --json
```

## Full Example

All options combined:

```bash
vex --target api.example.com \
    --port 8443 \
    --workers 50 \
    --requests 5000 \
    --duration 120 \
    --path /api/v2/test \
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

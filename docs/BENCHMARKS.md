# Benchmark Results

Results captured on 2026-04-24 using the release build.

---

## google.com — 1000 requests, 10 workers, 100 concurrency

```bash
cargo run --release -- --target google.com --insecure \
  --success-status 2xx,3xx -n 1000 --workers 10 -c 100
```

```
Starting HTTP/3 load test:
  Target: google.com:443
  Host: google.com
  Path: /
  Workers: 10
  Concurrency per worker: 100
  Total requests: 1000
  Duration: 30s
  Insecure: true

Load test completed:
  Total time: 1.10s
  Total requests: 1000
  Successful requests: 1000
  Failed requests: 0
  Requests/sec: 907.55
  Completion reason: All 1000 requests completed

HTTP Status code breakdown:
  302: 3xx Redirect (1000)

Latency metrics (ms):
  Min:   283.63
  Max:  1052.91
  Avg:   690.64
  p50:   685.00
  p90:   854.11
  p95:   864.72
  p99:   930.92
```

**Notes:**
- Google closes the QUIC connection after every response (one request per connection).
- Each request pays a full TLS+QUIC handshake round trip (~300-1000ms to Google's servers).
- Throughput of ~907 req/s is achieved by running 10 workers × 100 concurrent streams in parallel, saturating the reconnect pipeline.
- Latency figures reflect handshake cost, not server processing time. A server that reuses connections will show much lower p50/p99.

---

## Interpreting Results

| Metric | What it tells you |
|--------|-------------------|
| `Requests/sec` | Overall throughput across all workers |
| `p50` | Median request latency — typical user experience |
| `p90` / `p95` | Tail latency — affects 1 in 10 / 1 in 20 requests |
| `p99` | Worst-tail latency — affects 1 in 100 requests |
| `Failed requests` | Requests that did not receive a valid response |

A healthy server with persistent QUIC connections will show latency well under 100ms p99 and near-zero failed requests.

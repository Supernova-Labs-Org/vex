# Benchmarking proxy.spooky.local

Target: `127.0.0.1:9889` (proxy.spooky.local)
Configuration: `/` (1 upstream) vs `/api` (2 upstreams)

## Quick Commands

**Connectivity test:**
```bash
cargo run --release -- --target 127.0.0.1 --port 9889 --insecure --workers 1 --requests 10 --verbose --path /api
```

**Single upstream (root path):**
```bash
cargo run --release -- --target 127.0.0.1 --port 9889 --insecure --path /api/v1 --workers 100 --requests 1000
```

**Dual upstream (API path):**
```bash
cargo run --release -- --target 127.0.0.1 --port 9889 --insecure --path /api --workers 100 --requests 10000
```

## Full Test Suite

Save as `benchmark.sh`:

```bash
#!/bin/bash

TARGET="127.0.0.1"
PORT="9889"

echo "=== BASELINE ==="
cargo run --release -- --target $TARGET --port $PORT --insecure --path "/" --workers 1 --requests 1000
cargo run --release -- --target $TARGET --port $PORT --insecure --path "/api" --workers 1 --requests 1000

echo -e "\n=== CONCURRENCY (50 workers) ==="
cargo run --release -- --target $TARGET --port $PORT --insecure --path "/" --workers 50 --requests 5000
cargo run --release -- --target $TARGET --port $PORT --insecure --path "/api" --workers 50 --requests 5000

echo -e "\n=== HEAVY LOAD (100 workers) ==="
cargo run --release -- --target $TARGET --port $PORT --insecure --path "/" --workers 100 --requests 10000
cargo run --release -- --target $TARGET --port $PORT --insecure --path "/api" --workers 100 --requests 10000

echo -e "\n=== DURATION TEST (60s) ==="
cargo run --release -- --target $TARGET --port $PORT --insecure --path "/" --workers 100 --duration 60
cargo run --release -- --target $TARGET --port $PORT --insecure --path "/api" --workers 100 --duration 60
```

Run: `chmod +x benchmark.sh && ./benchmark.sh`

## Comparison Script

Save as `compare.sh`:

```bash
#!/bin/bash

TARGET="127.0.0.1"
PORT="9889"
HOST="proxy.spooky.local"

run_test() {
  echo ">>> $1"
  cargo run --release -- --target $TARGET --port $PORT --insecure --path "$2" --workers $3 --requests $4
  echo ""
}

echo "=== SINGLE UPSTREAM vs DUAL UPSTREAM ==="

run_test "Single (10w, 1000r)" "/" 10 1000
run_test "Dual (10w, 1000r)" "/api" 10 1000

run_test "Single (50w, 5000r)" "/" 50 5000
run_test "Dual (50w, 5000r)" "/api" 50 5000

run_test "Single (100w, 10000r)" "/" 100 10000
run_test "Dual (100w, 10000r)" "/api" 100 10000
```

Run: `chmod +x compare.sh && ./compare.sh`

## What to Look For

- **Dual should be ~2x faster** (2 upstream servers)
- **Single upstream**: Higher p99 latency under load
- **Dual upstream**: Better error resilience
- Compare `req/s` and `p99` metrics between paths

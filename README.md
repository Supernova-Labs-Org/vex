# vex ⚡  

A minimal load testing tool for HTTP/3 servers built in Rust

---

## ✨ Vision

- **HTTP/3 support** with `reqwest` and `rustls-tls`
- **Concurrent request execution** using Tokio async runtime
- **Basic metrics collection**: success/failure counts and RPS calculation
- **Self-signed certificate support** for local testing
- **Simple console output** with request statistics

---

## 🧩 Planned Features  

- [ ] **CLI interface** with configurable parameters (`-c`, `-d`, `-n`)
- [ ] **Rich latency metrics** (avg, p50, p95, p99, max)
- [ ] **Export results** as JSON/CSV
- [ ] **Custom headers** & **POST payloads** support
- [ ] **Connection keep-alive** vs fresh-per-request options
- [ ] **Histogram reporting** in console
- [ ] **Ramp-up traffic mode** (gradual increase)
- [ ] **Distributed load testing** across multiple nodes

---

## 🏗️ Current Implementation  

The current implementation demonstrates the core architecture:

1. **HTTP/3 Client Setup**  
   - Uses `reqwest` with `http3_prior_knowledge()` for HTTP/3 support
   - Configured to accept self-signed certificates for local testing

2. **Concurrent Worker System**  
   - Spawns multiple async tasks using `tokio::spawn`
   - Each worker handles a portion of the total requests
   - Uses atomic counters for thread-safe metrics collection

3. **Basic Metrics Collection**  
   - Tracks successful and failed requests
   - Calculates requests per second (RPS)
   - Measures total elapsed time

4. **Simple Reporting**  
   - Console output with basic statistics
   - Success/failure counts and performance metrics

---

## 📊 Current Usage  

```bash
# Compile and run (currently hardcoded parameters)
cargo run

# Current configuration in main.rs:
# - URL: https://127.0.0.1:7777
# - Total requests: 1000
# - Concurrency: 50 workers
```

## 📊 Planned CLI Usage  

```bash
# Run 5000 requests with 100 concurrent workers
vex -c 100 -n 5000 https://api.example.com/

# Run load test for 30s with 200 concurrency
vex -c 200 -d 30s https://service.local/ping

# Send POST requests with JSON body
vex -c 50 -n 1000 -X POST -H "Content-Type: application/json" \
  -d '{"msg": "hello"}' https://api.example.com/echo
```

---

## 📅 Roadmap

* **Phase 1** ✅ → Basic HTTP/3 load testing with concurrent workers
* **Phase 2** → CLI interface, latency percentiles, and export features
* **Phase 3** → Advanced features (keep-alive, custom headers, POST support)
* **Future** → TCP/WebSocket support, distributed mode

---

## 🛠️ Dependencies

- **Tokio** - Async runtime for concurrent request handling
- **Reqwest** - HTTP client with HTTP/3 support via rustls
- **Futures** - Async utilities for joining concurrent tasks

---

## 📝 Development Notes

* **Rust chosen** for performance and fine-grained control over concurrency
* **HTTP/3 focus** - modern protocol with better performance characteristics
* **Atomic operations** used for thread-safe metrics collection
* **Async/await pattern** for efficient concurrent request handling
* Keep it **minimal like `wrk`** - focus on core functionality first

---

## 🔖 License

Apache 2.0
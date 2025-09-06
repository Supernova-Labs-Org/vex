# vex ⚡  
*A minimal load testing tool for HTTP servers*  

`vex` is a lightweight, high-performance load tester (inspired by `wrk` and `hey`) designed to simulate **concurrent users** and measure **throughput, latency, and error rates**.  

---

## 🚀 Vision  

- Stress test APIs and web services with **thousands of concurrent requests**.  
- Collect rich **latency metrics** (avg, p50, p95, p99, max).  
- Provide a simple **CLI interface** with flexible parameters.  
- Future-proof: modular enough to extend for **TCP/WebSocket** testing.  

---

## 🧩 Planned Features  

- [ ] Run with **concurrency (`-c`)** and **duration (`-d`)** or **requests (`-n`)**.  
- [ ] Measure **RPS (requests/sec)** and **latency stats**.  
- [ ] Export results as **JSON/CSV**.  
- [ ] Support **custom headers** & **POST payloads**.  
- [ ] Connection **keep-alive** vs fresh-per-request.  
- [ ] Simple **histogram reporting** in console.  
- [ ] (Later) Ramp-up traffic mode (gradual increase).  
- [ ] (Later) Support for **distributed load testing** across multiple nodes.  

---

## ⚙️ Workflow (Implementation Plan)  

1. **Input & Config Parser**  
   - Parse CLI args (url, concurrency, requests, duration, headers, body).  

2. **Worker Engine**  
   - Spawn workers (goroutines or async tasks).  
   - Each worker sends requests until done.  
   - Record latency + status.  

3. **Aggregator**  
   - Collect results from workers.  
   - Calculate throughput, latency distribution, errors.  

4. **Reporter**  
   - Console output (human-readable).  
   - Optional JSON/CSV export.  

5. **Optimizations (Phase 2)**  
   - Connection reuse.  
   - Efficient timers.  
   - Lock-free channels (Rust) / minimal contention (Go).  

---

## 📊 Example Usage (planned)  

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

* **Month 1** → MVP with basic concurrency + stats.
* **Month 2** → Latency percentiles, exports, keep-alive, error handling.
* **Future** → TCP/WebSocket support, distributed mode.

---

## 📝 Notes to Future Me

* Keep it **minimal like `wrk`**, not bloated.
* Decide early: **Go = simple goroutines**, **Rust = perf + fine-grained control**.
* Benchmark against `wrk` to see where we stand.
* Don’t over-optimize before MVP — focus on reliability first.

---

## 🔖 License

MIT (can switch later if needed).

# References

## Core Concurrency
- [AtomicUsize](https://doc.rust-lang.org/std/sync/atomic/index.html) - lock-free counters
- [Arc<T>](https://doc.rust-lang.org/std/sync/struct.Arc.html) - shared ownership across threads
- [Ordering::Relaxed](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html) - non-synchronizing atomics

## HTTP/3 Implementation
- [Quinn connection setup](https://quinn-rs.github.io/quinn/quinn/set-up-connection.html) - native QUIC protocol
- [Endpoint client](https://docs.rs/quinn/latest/quinn/struct.Endpoint.html) - connection management

Phase 1: Minimal Changes (Core Fix)
Update CLI args: --connections-per-worker, --max-concurrent-streams
Refactor Http3Client struct with Arc<Mutex<ConnectionPoolState>>
Implement ensure_connected() (handshake once per worker)
Rewrite send_request() to use HTTP/3 streams (not full connections)
Update worker loop to reuse connections
Add metrics tracking




Phase 2: Testing
Unit tests for connection pool
Integration tests with mock server
Load test comparison (before/after)



Phase 3: Advanced Features (Optional)
Connection pool with fallback
Exponential backoff on failures
Stream ID limits


Phase 4: Production Ready
Documentation
Performance benchmarks
CLI validation
Key Code Changes
Before:


// src/client/h3_client.rs
pub struct Http3Client {
    config: quiche::Config,  // Config only
    pub insecure: bool,
}

// Every request:
pub async fn send_request() {
    let socket = UdpSocket::bind(...).await?;  // NEW socket
    let mut conn = quiche::connect(...)?;  // NEW connection
    // Handshake + request + close
}
After:


// src/client/h3_client.rs
pub struct Http3Client {
    config: quiche::Config,
    insecure: bool,
    pool: Arc<Mutex<ConnectionPoolState>>,  // Persistent
}

pub struct ConnectionPoolState {
    quic_conn: Option<quiche::Connection>,  // Reused
    h3_conn: Option<quiche::h3::Connection>,
    socket: Option<UdpSocket>,
    next_stream_id: u64,  // Track stream IDs
    reuse_count: usize,
}

// Per worker:
pub async fn ensure_connected() {  // Called once, handshake happens here
    // Create connection if not exists
}

// Per request:
pub async fn send_request() {
    // Allocate new stream ID (0, 4, 8, 12...)
    // Send request on existing connection
    // Receive response
    // Stream closes, connection stays open
}
Would you like me to:
Start Phase 1 implementation (most critical fixes)?
Deep dive into any specific part (pseudocode, async locking, GOAWAY handling)?
Create the detailed specification documents?
Set up the testing strategy first?
What's your priority?
use quiche::h3;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

/// Error statistics for a request
#[derive(Debug, Clone, Default)]
pub struct ErrorStats {
    pub send_errors: usize,
    pub recv_errors: usize,
    pub quic_errors: usize,
    pub stream_reset_errors: usize,
}

/// Result of a single HTTP/3 request
#[derive(Debug, Clone)]
pub struct ResponseResult {
    pub status_code: u16,
    pub bytes_received: usize,
    pub errors: ErrorStats,
    pub latency_ms: f64,
    /// Body content only captured in verbose mode for debugging
    pub body: Option<String>,
}

/// Persistent connection pool state per worker
///
/// Maintains a single QUIC connection and H3 connection for reuse across
/// multiple requests within a worker task. Tracks stream IDs and connection state.
pub struct ConnectionPoolState {
    pub quic_conn: Option<quiche::Connection>,
    pub h3_conn: Option<h3::Connection>,
    pub socket: Option<Arc<UdpSocket>>,
    pub local_addr: Option<SocketAddr>,
    pub peer_addr: Option<SocketAddr>,
    pub next_stream_id: u64,
    pub reuse_count: usize,
    pub failed: bool,
}

impl Default for ConnectionPoolState {
    fn default() -> Self {
        Self {
            quic_conn: None,
            h3_conn: None,
            socket: None,
            local_addr: None,
            peer_addr: None,
            next_stream_id: 0,
            reuse_count: 0,
            failed: false,
        }
    }
}

impl ConnectionPoolState {
    /// Allocate next stream ID and increment counter
    ///
    /// QUIC uses 0, 4, 8, 12... for bidirectional streams from client
    pub fn allocate_stream_id(&mut self) -> u64 {
        let id = self.next_stream_id;
        self.next_stream_id += 4;
        self.reuse_count += 1;
        id
    }

    /// Reset connection state (e.g., after GOAWAY)
    pub fn mark_failed(&mut self) {
        self.failed = true;
    }

    /// Check if connection should be reused
    pub fn is_usable(&self) -> bool {
        self.quic_conn.is_some() && self.h3_conn.is_some() && self.socket.is_some() && !self.failed
    }
}

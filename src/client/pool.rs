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
}

/// Persistent connection pool state per worker
///
/// Maintains a single QUIC connection and H3 connection for reuse across
/// multiple requests within a worker task. `reuse_count` tracks how many
/// requests this worker has dispatched.
#[derive(Default)]
pub struct ConnectionPoolState {
    pub quic_conn: Option<quiche::Connection>,
    pub h3_conn: Option<h3::Connection>,
    pub socket: Option<Arc<UdpSocket>>,
    pub local_addr: Option<SocketAddr>,
    pub peer_addr: Option<SocketAddr>,
    pub reuse_count: usize,
    pub failed: bool,
}

impl ConnectionPoolState {
    /// Mark connection as failed (e.g., after GOAWAY or timeout)
    pub fn mark_failed(&mut self) {
        self.failed = true;
    }

    /// Check if connection should be reused
    pub fn is_usable(&self) -> bool {
        let quic_open = self
            .quic_conn
            .as_ref()
            .is_some_and(|quic_conn| !quic_conn.is_closed());

        quic_open && self.h3_conn.is_some() && self.socket.is_some() && !self.failed
    }
}

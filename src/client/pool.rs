use quiche::h3;
use std::fmt;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestErrorKind {
    ConnectionReplaced,
    ChannelClosed,
    TimedOut,
    DeadlineAborted,
    StreamReset,
    MissingStatus,
    ConnectionLost,
    NetworkSend,
    NetworkRecv,
    Quic,
    H3,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestError {
    pub kind: RequestErrorKind,
    pub message: String,
    pub stream_id: Option<u64>,
}

impl RequestError {
    pub fn new(kind: RequestErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            stream_id: None,
        }
    }

    pub fn with_stream_id(mut self, stream_id: u64) -> Self {
        self.stream_id = Some(stream_id);
        self
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(stream_id) = self.stream_id {
            write!(f, "{} (stream {})", self.message, stream_id)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for RequestError {}

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

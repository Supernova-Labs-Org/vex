//! Configuration constants for HTTP/3 client
//!
//! Centralized magic numbers for QUIC, network, and timeout parameters.
//! Adjust these values to tune performance for different scenarios.

pub mod quic {
    /// Maximum idle timeout for QUIC connections in milliseconds
    pub const MAX_IDLE_TIMEOUT_MS: u64 = 5_000;

    /// Maximum UDP payload size for sending packets (bytes)
    pub const MAX_SEND_UDP_PAYLOAD_SIZE: usize = 65_527;

    /// Maximum UDP payload size for receiving packets (bytes)
    pub const MAX_RECV_UDP_PAYLOAD_SIZE: usize = 65_527;

    /// Initial maximum data allowed on connection (bytes)
    pub const INITIAL_MAX_DATA: u64 = 10_000_000;

    /// Initial maximum data for bidirectional streams (bytes)
    pub const INITIAL_MAX_STREAM_DATA_BIDI: u64 = 1_000_000;

    /// Initial maximum data for unidirectional streams (bytes)
    pub const INITIAL_MAX_STREAM_DATA_UNI: u64 = 1_000_000;

    /// Maximum number of bidirectional streams
    pub const MAX_STREAMS_BIDI: u64 = 100;

    /// Maximum number of unidirectional streams
    pub const MAX_STREAMS_UNI: u64 = 100;
}

pub mod network {
    /// Buffer size for UDP packet data (bytes)
    pub const BUFFER_SIZE: usize = 65_535;

    /// Handshake completion timeout (seconds)
    pub const HANDSHAKE_TIMEOUT_SECS: u64 = 5;

    /// Response completion timeout (seconds)
    pub const RESPONSE_TIMEOUT_SECS: u64 = 5;

    /// Poll timeout during handshake (milliseconds)
    pub const HANDSHAKE_POLL_TIMEOUT_MS: u64 = 50;

    /// Poll timeout during response handling (milliseconds)
    pub const RESPONSE_POLL_TIMEOUT_MS: u64 = 100;
}

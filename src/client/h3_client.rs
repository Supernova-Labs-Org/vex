use super::{
    constants,
    pool::{ConnectionPoolState, ErrorStats, ResponseResult},
};
use quiche::{
    self,
    h3::{Header, NameValue},
};
use rand::RngCore;
use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{net::UdpSocket, sync::oneshot};

#[derive(Debug)]
pub enum DispatchError {
    ConnectionLost,
    StreamBlocked,
    H3(quiche::h3::Error),
}

impl DispatchError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            DispatchError::ConnectionLost | DispatchError::StreamBlocked
        )
    }
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DispatchError::ConnectionLost => write!(f, "connection lost"),
            DispatchError::StreamBlocked => write!(f, "stream blocked"),
            DispatchError::H3(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for DispatchError {}

fn bind_addr_for_peer(peer_addr: SocketAddr) -> SocketAddr {
    if peer_addr.is_ipv6() {
        SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0))
    } else {
        SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))
    }
}

// Per-stream state accumulated while the poll loop runs.
struct StreamState {
    status_code: Option<u16>,
    bytes_received: usize,
    errors: ErrorStats,
    start: Instant,
    verbose: bool,
    tx: oneshot::Sender<Result<ResponseResult, String>>,
}

pub struct Http3Client {
    pub insecure: bool,
    peer_addr: SocketAddr,
    pool: ConnectionPoolState,
    // In-flight streams waiting for their Finished event.
    in_flight: HashMap<u64, StreamState>,
}

impl Http3Client {
    pub fn new(insecure: bool, peer_addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            insecure,
            peer_addr,
            pool: ConnectionPoolState::default(),
            in_flight: HashMap::new(),
        })
    }

    fn build_quic_config(&self) -> Result<quiche::Config, Box<dyn std::error::Error>> {
        let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
        config.set_application_protos(quiche::h3::APPLICATION_PROTOCOL)?;
        config.set_max_idle_timeout(constants::quic::MAX_IDLE_TIMEOUT_MS);
        config.set_max_recv_udp_payload_size(constants::quic::MAX_RECV_UDP_PAYLOAD_SIZE);
        config.set_max_send_udp_payload_size(constants::quic::MAX_SEND_UDP_PAYLOAD_SIZE);
        config.set_initial_max_data(constants::quic::INITIAL_MAX_DATA);
        config
            .set_initial_max_stream_data_bidi_local(constants::quic::INITIAL_MAX_STREAM_DATA_BIDI);
        config
            .set_initial_max_stream_data_bidi_remote(constants::quic::INITIAL_MAX_STREAM_DATA_BIDI);
        config.set_initial_max_stream_data_uni(constants::quic::INITIAL_MAX_STREAM_DATA_UNI);
        config.set_initial_max_streams_bidi(constants::quic::MAX_STREAMS_BIDI);
        config.set_initial_max_streams_uni(constants::quic::MAX_STREAMS_UNI);
        config.enable_early_data();
        config.verify_peer(!self.insecure);
        Ok(config)
    }

    pub async fn ensure_connected(
        &mut self,
        server_name: &str,
        connect_timeout: Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.pool.is_usable() {
            return Ok(());
        }

        // poll_once drains in_flight before returning false, so by the time
        // ensure_connected is called there should be no orphaned streams.
        // Drain defensively in case ensure_connected is called from other paths.
        for (_, state) in self.in_flight.drain() {
            let _ = state
                .tx
                .send(Err("connection replaced before stream completed".into()));
        }

        let peer_addr = self.peer_addr;
        let bind_addr = bind_addr_for_peer(peer_addr);
        let socket = UdpSocket::bind(bind_addr).await?;
        let local_addr = socket.local_addr()?;

        let mut scid_bytes = [0u8; quiche::MAX_CONN_ID_LEN];
        rand::thread_rng().fill_bytes(&mut scid_bytes);
        let scid = quiche::ConnectionId::from_ref(&scid_bytes);

        let mut config = self.build_quic_config()?;
        let mut quic_conn =
            quiche::connect(Some(server_name), &scid, local_addr, peer_addr, &mut config)?;

        let mut out = [0u8; constants::network::BUFFER_SIZE];
        let mut buf = [0u8; constants::network::BUFFER_SIZE];
        let handshake_deadline = Instant::now() + connect_timeout;
        let mut h3_conn: Option<quiche::h3::Connection> = None;

        loop {
            if Instant::now() > handshake_deadline {
                return Err("Handshake timeout".into());
            }

            if quic_conn.is_established() && h3_conn.is_some() {
                break;
            }

            tokio::task::yield_now().await;

            loop {
                match quic_conn.send(&mut out) {
                    Ok((write, send_info)) => {
                        socket.send_to(&out[..write], send_info.to).await?;
                    }
                    Err(quiche::Error::Done) => break,
                    Err(e) => return Err(format!("send failed: {:?}", e).into()),
                }
            }

            if quic_conn.is_established() && h3_conn.is_none() {
                let h3_config = quiche::h3::Config::new()?;
                h3_conn = Some(quiche::h3::Connection::with_transport(
                    &mut quic_conn,
                    &h3_config,
                )?);
            }

            let quic_timeout = quic_conn.timeout().unwrap_or(Duration::from_millis(
                constants::network::HANDSHAKE_POLL_TIMEOUT_MS,
            ));
            let timeout = quic_timeout.min(Duration::from_millis(
                constants::network::HANDSHAKE_POLL_TIMEOUT_MS,
            ));
            match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
                Ok(Ok((len, from))) => {
                    let recv_info = quiche::RecvInfo {
                        from,
                        to: local_addr,
                    };
                    match quic_conn.recv(&mut buf[..len], recv_info) {
                        Ok(_) | Err(quiche::Error::Done) => {}
                        Err(err) => {
                            return Err(
                                format!("quic recv failed during handshake: {:?}", err).into()
                            );
                        }
                    }
                }
                Ok(Err(err)) => {
                    return Err(format!("socket recv failed during handshake: {}", err).into());
                }
                Err(_) => {
                    quic_conn.on_timeout();
                }
            }
        }

        self.pool.quic_conn = Some(quic_conn);
        self.pool.h3_conn = h3_conn;
        self.pool.socket = Some(Arc::new(socket));
        self.pool.local_addr = Some(local_addr);
        self.pool.peer_addr = Some(peer_addr);
        self.pool.failed = false;

        Ok(())
    }

    // Dispatch one HTTP/3 stream and return a receiver that resolves when the
    // stream's Finished event arrives. The caller must drive the connection
    // (via `poll_once`) while awaiting the receiver.
    pub fn dispatch(
        &mut self,
        authority: &str,
        path: &str,
        verbose: bool,
    ) -> Result<(u64, oneshot::Receiver<Result<ResponseResult, String>>), DispatchError> {
        let pool = &mut self.pool;
        let quic_conn = pool
            .quic_conn
            .as_mut()
            .ok_or(DispatchError::ConnectionLost)?;
        let h3_conn = pool.h3_conn.as_mut().ok_or(DispatchError::ConnectionLost)?;

        let req = vec![
            Header::new(b":method", b"GET"),
            Header::new(b":scheme", b"https"),
            Header::new(b":authority", authority.as_bytes()),
            Header::new(b":path", path.as_bytes()),
            Header::new(b"user-agent", b"vex-h3-client"),
        ];
        let stream_id = h3_conn.send_request(quic_conn, &req, true).map_err(|e| {
            if matches!(
                e,
                quiche::h3::Error::StreamBlocked | quiche::h3::Error::Done
            ) {
                DispatchError::StreamBlocked
            } else {
                DispatchError::H3(e)
            }
        })?;

        let (tx, rx) = oneshot::channel();
        self.in_flight.insert(
            stream_id,
            StreamState {
                status_code: None,
                bytes_received: 0,
                errors: ErrorStats::default(),
                start: Instant::now(),
                verbose,
                tx,
            },
        );

        Ok((stream_id, rx))
    }

    fn drain_in_flight_with_error(&mut self, message: &str, bump: fn(&mut ErrorStats)) {
        for (_, mut state) in self.in_flight.drain() {
            bump(&mut state.errors);
            let _ = state.tx.send(Err(message.to_string()));
        }
    }

    pub fn abandon_stream(&mut self, stream_id: u64) {
        if let Some(state) = self.in_flight.remove(&stream_id) {
            let _ = state.tx.send(Err("request timed out".into()));
        }

        if let Some(quic_conn) = self.pool.quic_conn.as_mut() {
            let _ = quic_conn.stream_shutdown(stream_id, quiche::Shutdown::Read, 0);
        }
    }

    // Run one iteration of the QUIC I/O + H3 poll loop, routing events to
    // their waiting StreamState entries. Returns true if the connection is
    // still alive.
    pub async fn poll_once(&mut self) -> bool {
        let mut buf = [0u8; constants::network::BUFFER_SIZE];
        let mut out = [0u8; constants::network::BUFFER_SIZE];

        let (socket, local_addr) = match (self.pool.socket.clone(), self.pool.local_addr) {
            (Some(s), Some(a)) => (s, a),
            _ => return false,
        };

        let connection_closed;
        {
            let quic_conn = match self.pool.quic_conn.as_mut() {
                Some(c) => c,
                None => return false,
            };

            if !quic_conn.is_closed() {
                let quic_timeout = quic_conn.timeout().unwrap_or(Duration::from_millis(
                    constants::network::RESPONSE_POLL_TIMEOUT_MS,
                ));
                let timeout = quic_timeout.min(Duration::from_millis(
                    constants::network::RESPONSE_POLL_TIMEOUT_MS,
                ));

                match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
                    Ok(Ok((len, from))) => {
                        let recv_info = quiche::RecvInfo {
                            from,
                            to: local_addr,
                        };
                        if let Err(e) = quic_conn.recv(&mut buf[..len], recv_info)
                            && e != quiche::Error::Done
                        {
                            self.pool.mark_failed();
                            let msg = format!("quic recv error: {:?}", e);
                            self.drain_in_flight_with_error(&msg, |errors| {
                                errors.quic_errors += 1;
                            });
                            return false;
                        }
                    }
                    Ok(Err(err)) => {
                        self.pool.mark_failed();
                        let msg = format!("socket recv error: {err}");
                        self.drain_in_flight_with_error(&msg, |errors| {
                            errors.recv_errors += 1;
                        });
                        return false;
                    }
                    Err(_) => {
                        quic_conn.on_timeout();
                    }
                }

                while let Ok((write, send_info)) = quic_conn.send(&mut out) {
                    if let Err(err) = socket.send_to(&out[..write], send_info.to).await {
                        self.pool.mark_failed();
                        let msg = format!("socket send error: {err}");
                        self.drain_in_flight_with_error(&msg, |errors| {
                            errors.send_errors += 1;
                        });
                        return false;
                    }
                }
            }

            // Re-check after processing — recv may have caused connection close.
            connection_closed = quic_conn.is_closed();
        }

        // Drain H3 events and route to per-stream state.
        let mut finished_streams: Vec<u64> = Vec::new();
        let mut got_goaway = false;

        {
            let quic_conn = match self.pool.quic_conn.as_mut() {
                Some(c) => c,
                None => return false,
            };
            let h3_conn = match self.pool.h3_conn.as_mut() {
                Some(c) => c,
                None => return false,
            };

            loop {
                match h3_conn.poll(quic_conn) {
                    Ok((id, quiche::h3::Event::Headers { list, .. })) => {
                        if let Some(state) = self.in_flight.get_mut(&id) {
                            for h in list {
                                let name = String::from_utf8_lossy(h.name());
                                let value = String::from_utf8_lossy(h.value());
                                if name == ":status"
                                    && let Ok(code) = value.parse::<u16>()
                                {
                                    state.status_code = Some(code);
                                }
                                if state.verbose {
                                    println!("{name}: {value}");
                                }
                            }
                        }
                    }
                    Ok((id, quiche::h3::Event::Data)) => {
                        if let Some(state) = self.in_flight.get_mut(&id) {
                            loop {
                                match h3_conn.recv_body(quic_conn, id, &mut buf) {
                                    Ok(read) => {
                                        state.bytes_received += read;
                                    }
                                    Err(quiche::h3::Error::Done) => break,
                                    Err(e) => {
                                        state.errors.quic_errors += 1;
                                        eprintln!("recv_body error on stream {id}: {:?}", e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Ok((id, quiche::h3::Event::Finished)) => {
                        finished_streams.push(id);
                    }
                    Ok((_id, quiche::h3::Event::GoAway)) => {
                        // Mark failed after the borrow scope ends — GoAway means
                        // "don't open new streams", not "abort existing ones".
                        got_goaway = true;
                    }
                    Ok((_id, quiche::h3::Event::Reset(sid))) => {
                        if let Some(mut state) = self.in_flight.remove(&sid) {
                            state.errors.stream_reset_errors += 1;
                            let _ = state.tx.send(Err(format!("stream {sid} reset by peer")));
                        }
                    }
                    Ok((_id, quiche::h3::Event::PriorityUpdate)) => {}
                    Err(quiche::h3::Error::Done) => break,
                    Err(e) => {
                        eprintln!("h3 poll error: {:?}", e);
                        self.pool.mark_failed();
                        let msg = format!("h3 poll error: {:?}", e);
                        self.drain_in_flight_with_error(&msg, |errors| {
                            errors.quic_errors += 1;
                        });
                        break;
                    }
                }
            }
        }

        if got_goaway {
            self.pool.mark_failed();
        }

        // Resolve finished streams.
        for id in finished_streams {
            if let Some(state) = self.in_flight.remove(&id) {
                let latency_ms = state.start.elapsed().as_secs_f64() * 1000.0;
                let result = match state.status_code {
                    None => Err("stream finished without :status header".into()),
                    Some(code) => Ok(ResponseResult {
                        status_code: code,
                        bytes_received: state.bytes_received,
                        errors: state.errors,
                        latency_ms,
                    }),
                };
                let _ = state.tx.send(result);
            }
        }

        if connection_closed {
            self.pool.mark_failed();
            // Any streams that didn't get a Finished event before the connection
            // closed will never complete on this connection. Signal them so their
            // tasks unblock; callers should retry these streams.
            for (_, state) in self.in_flight.drain() {
                let _ = state
                    .tx
                    .send(Err("connection replaced before stream completed".into()));
            }
            return false;
        }

        true
    }

    pub fn has_in_flight(&self) -> bool {
        !self.in_flight.is_empty()
    }

    pub fn is_connected(&self) -> bool {
        self.pool.is_usable()
    }
}

#[cfg(test)]
mod tests {
    use super::bind_addr_for_peer;
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

    #[test]
    fn bind_addr_matches_ipv4_peer_family() {
        let peer = SocketAddr::from((Ipv4Addr::new(203, 0, 113, 10), 443));
        let bind = bind_addr_for_peer(peer);
        assert!(bind.is_ipv4());
    }

    #[test]
    fn bind_addr_matches_ipv6_peer_family() {
        let peer = SocketAddr::from((Ipv6Addr::LOCALHOST, 443));
        let bind = bind_addr_for_peer(peer);
        assert!(bind.is_ipv6());
    }
}

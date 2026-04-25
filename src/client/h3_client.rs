use quiche::{self, h3::{Header, NameValue}};
use rand::RngCore;
use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    time::{Duration, Instant},
    sync::Arc,
};
use tokio::{net::UdpSocket, sync::oneshot};
use super::{constants, pool::{ConnectionPoolState, ErrorStats, ResponseResult}};

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
    body: Vec<u8>,
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
        config.set_initial_max_stream_data_bidi_local(constants::quic::INITIAL_MAX_STREAM_DATA_BIDI);
        config.set_initial_max_stream_data_bidi_remote(constants::quic::INITIAL_MAX_STREAM_DATA_BIDI);
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.pool.is_usable() {
            return Ok(());
        }

        // Discard any in-flight state from the dead connection.
        for (_, state) in self.in_flight.drain() {
            let _ = state.tx.send(Err("connection reset before stream completed".into()));
        }

        let peer_addr = self.peer_addr;
        let bind_addr = bind_addr_for_peer(peer_addr);
        let socket = UdpSocket::bind(bind_addr).await?;
        let local_addr = socket.local_addr()?;

        let mut scid_bytes = [0u8; quiche::MAX_CONN_ID_LEN];
        rand::thread_rng().fill_bytes(&mut scid_bytes);
        let scid = quiche::ConnectionId::from_ref(&scid_bytes);

        let mut config = self.build_quic_config()?;
        let mut quic_conn = quiche::connect(Some(server_name), &scid, local_addr, peer_addr, &mut config)?;

        let mut out = [0u8; constants::network::BUFFER_SIZE];
        let mut buf = [0u8; constants::network::BUFFER_SIZE];
        let handshake_deadline = Instant::now() + Duration::from_secs(constants::network::HANDSHAKE_TIMEOUT_SECS);
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
                    Ok((write, send_info)) => { socket.send_to(&out[..write], send_info.to).await?; }
                    Err(quiche::Error::Done) => break,
                    Err(e) => return Err(format!("send failed: {:?}", e).into()),
                }
            }

            if quic_conn.is_established() && h3_conn.is_none() {
                let h3_config = quiche::h3::Config::new()?;
                h3_conn = Some(quiche::h3::Connection::with_transport(&mut quic_conn, &h3_config)?);
            }

            let timeout = quic_conn.timeout().unwrap_or(Duration::from_millis(constants::network::HANDSHAKE_POLL_TIMEOUT_MS));
            match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
                Ok(Ok((len, from))) => {
                    let recv_info = quiche::RecvInfo { from, to: local_addr };
                    match quic_conn.recv(&mut buf[..len], recv_info) {
                        Ok(_) | Err(quiche::Error::Done) => {}
                        Err(err) => return Err(format!("quic recv failed during handshake: {:?}", err).into()),
                    }
                }
                Ok(Err(err)) => return Err(format!("socket recv failed during handshake: {}", err).into()),
                Err(_) => { quic_conn.on_timeout(); }
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
    ) -> Result<(u64, oneshot::Receiver<Result<ResponseResult, String>>), Box<dyn std::error::Error>> {
        let pool = &mut self.pool;
        let quic_conn = pool.quic_conn.as_mut().ok_or("Connection lost")?;
        let h3_conn = pool.h3_conn.as_mut().ok_or("Connection lost")?;

        let req = vec![
            Header::new(b":method", b"GET"),
            Header::new(b":scheme", b"https"),
            Header::new(b":authority", authority.as_bytes()),
            Header::new(b":path", path.as_bytes()),
            Header::new(b"user-agent", b"vex-h3-client"),
        ];
        let stream_id = h3_conn.send_request(quic_conn, &req, true)?;

        let (tx, rx) = oneshot::channel();
        self.in_flight.insert(stream_id, StreamState {
            status_code: None,
            bytes_received: 0,
            body: Vec::new(),
            errors: ErrorStats::default(),
            start: Instant::now(),
            verbose,
            tx,
        });

        Ok((stream_id, rx))
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

        {
            let quic_conn = match self.pool.quic_conn.as_mut() {
                Some(c) => c,
                None => return false,
            };

            if quic_conn.is_closed() {
                self.pool.mark_failed();
                return false;
            }

            let timeout = quic_conn.timeout()
                .unwrap_or(Duration::from_millis(constants::network::RESPONSE_POLL_TIMEOUT_MS));

            match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
                Ok(Ok((len, from))) => {
                    let recv_info = quiche::RecvInfo { from, to: local_addr };
                    if let Err(e) = quic_conn.recv(&mut buf[..len], recv_info) {
                        if e != quiche::Error::Done {
                            // Propagate quic error to all waiters.
                            self.pool.mark_failed();
                            let msg = format!("quic recv error: {:?}", e);
                            for (_, state) in self.in_flight.drain() {
                                let _ = state.tx.send(Err(msg.clone()));
                            }
                            return false;
                        }
                    }
                }
                Ok(Err(_)) => {}
                Err(_) => { quic_conn.on_timeout(); }
            }

            loop {
                match quic_conn.send(&mut out) {
                    Ok((write, send_info)) => {
                        let _ = socket.send_to(&out[..write], send_info.to).await;
                    }
                    Err(quiche::Error::Done) | Err(_) => break,
                }
            }
        }

        // Drain H3 events and route to per-stream state.
        let mut finished_streams: Vec<u64> = Vec::new();

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
                                if name == ":status" {
                                    if let Ok(code) = value.parse::<u16>() {
                                        state.status_code = Some(code);
                                    }
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
                                        if state.verbose {
                                            state.body.extend_from_slice(&buf[..read]);
                                        }
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
                        self.pool.mark_failed();
                        let msg = "connection received GOAWAY".to_string();
                        for (_, state) in self.in_flight.drain() {
                            let _ = state.tx.send(Err(msg.clone()));
                        }
                        return false;
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
                        break;
                    }
                }
            }
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
                        body: if state.verbose {
                            Some(String::from_utf8_lossy(&state.body).into_owned())
                        } else {
                            None
                        },
                    }),
                };
                let _ = state.tx.send(result);
            }
        }

        true
    }

    pub fn has_in_flight(&self) -> bool {
        !self.in_flight.is_empty()
    }

    pub fn is_connected(&self) -> bool {
        self.pool.is_usable()
    }

    // High-level send_request: dispatch + drive until this stream resolves.
    // Used when the caller wants a simple sequential interface.
    pub async fn send_request(
        &mut self,
        server_name: &str,
        authority: &str,
        path: &str,
        verbose: bool,
    ) -> Result<ResponseResult, Box<dyn std::error::Error>> {
        self.ensure_connected(server_name).await?;

        if !self.pool.is_usable() {
            return Err("Connection lost".into());
        }
        self.pool.reuse_count += 1;

        let (_stream_id, mut rx) = self.dispatch(authority, path, verbose)?;

        let deadline = Instant::now() + Duration::from_secs(constants::network::RESPONSE_TIMEOUT_SECS);

        loop {
            if Instant::now() >= deadline {
                self.pool.mark_failed();
                return Err("timeout waiting for response".into());
            }

            // Check if our stream already resolved (e.g. result delivered in poll_once).
            match rx.try_recv() {
                Ok(result) => return result.map_err(|e| e.into()),
                Err(oneshot::error::TryRecvError::Closed) => {
                    return Err("response channel closed unexpectedly".into());
                }
                Err(oneshot::error::TryRecvError::Empty) => {}
            }

            if !self.poll_once().await {
                // Connection died; check if our result was delivered before the death.
                return Err("connection closed before response completed".into());
            }

            // Re-check after poll_once delivered events.
            match rx.try_recv() {
                Ok(result) => return result.map_err(|e| e.into()),
                Err(oneshot::error::TryRecvError::Closed) => {
                    return Err("response channel closed unexpectedly".into());
                }
                Err(oneshot::error::TryRecvError::Empty) => {}
            }
        }
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

use quiche::{self, h3::{Header, NameValue}};
use rand::RngCore;
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
    sync::Arc,
};
use tokio::net::UdpSocket;
use crate::utils::resolve_target;
use super::pool::{ConnectionPoolState, ErrorStats, ResponseResult};

pub struct Http3Client {
    config: quiche::Config,
    pub insecure: bool,
    pool: ConnectionPoolState,
}

impl Http3Client {
    pub fn new(insecure: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
        config.set_application_protos(quiche::h3::APPLICATION_PROTOCOL)?;
        config.set_max_idle_timeout(5_000);
        config.set_max_recv_udp_payload_size(65_527);
        config.set_max_send_udp_payload_size(65_527);
        config.set_initial_max_data(10_000_000);
        config.set_initial_max_stream_data_bidi_local(1_000_000);
        config.set_initial_max_stream_data_bidi_remote(1_000_000);
        config.set_initial_max_stream_data_uni(1_000_000);
        config.set_initial_max_streams_bidi(100);
        config.set_initial_max_streams_uni(100);
        config.enable_early_data();
        config.verify_peer(!insecure);

        Ok(Self {
            config,
            insecure,
            pool: ConnectionPoolState::default(),
        })
    }

    pub async fn ensure_connected(
        &mut self,
        target: &str,
        port: u16,
        host: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pool = &mut self.pool;

        // Always create fresh connection
        // TODO: Fix HTTP/3 stream reuse - current impl has issues with stream ID tracking
        // Disabling connection reuse for stability; each request gets a fresh connection

        // Resolve target
        let peer_addr = resolve_target(target, port)?;
        let bind_addr: SocketAddr = "0.0.0.0:0".parse()?;
        let socket = UdpSocket::bind(bind_addr).await?;
        let local_addr = socket.local_addr()?;

        // Create new QUIC connection
        let mut scid_bytes = [0u8; quiche::MAX_CONN_ID_LEN];
        rand::thread_rng().fill_bytes(&mut scid_bytes);
        let scid = quiche::ConnectionId::from_ref(&scid_bytes);

        let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
        config.set_application_protos(quiche::h3::APPLICATION_PROTOCOL)?;
        config.set_max_idle_timeout(5_000);
        config.set_max_recv_udp_payload_size(65_527);
        config.set_max_send_udp_payload_size(65_527);
        config.set_initial_max_data(10_000_000);
        config.set_initial_max_stream_data_bidi_local(1_000_000);
        config.set_initial_max_stream_data_bidi_remote(1_000_000);
        config.set_initial_max_stream_data_uni(1_000_000);
        config.set_initial_max_streams_bidi(100);
        config.set_initial_max_streams_uni(100);
        config.enable_early_data();
        config.verify_peer(!self.insecure);

        let mut quic_conn = quiche::connect(Some(host), &scid, local_addr, peer_addr, &mut config)?;

        // Perform handshake
        let mut out = [0u8; 65_535];
        let mut buf = [0u8; 65_535];
        let handshake_deadline = Instant::now() + Duration::from_secs(5);
        let mut h3_conn: Option<quiche::h3::Connection> = None;

        loop {
            if Instant::now() > handshake_deadline {
                return Err("Handshake timeout".into());
            }

            if quic_conn.is_established() && h3_conn.is_some() {
                break;
            }

            // Send pending packets
            loop {
                match quic_conn.send(&mut out) {
                    Ok((write, send_info)) => {
                        socket.send_to(&out[..write], send_info.to).await?;
                    }
                    Err(quiche::Error::Done) => break,
                    Err(e) => return Err(format!("send failed: {:?}", e).into()),
                }
            }

            // Initialize H3 once established
            if quic_conn.is_established() && h3_conn.is_none() {
                let h3_config = quiche::h3::Config::new()?;
                h3_conn = Some(quiche::h3::Connection::with_transport(&mut quic_conn, &h3_config)?);
            }

            // Receive packets with timeout
            let timeout = quic_conn.timeout().unwrap_or(Duration::from_millis(50));
            match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
                Ok(Ok((len, from))) => {
                    let recv_info = quiche::RecvInfo { from, to: local_addr };
                    let _ = quic_conn.recv(&mut buf[..len], recv_info);
                }
                Ok(Err(_)) => {
                    return Err("socket recv failed".into());
                }
                Err(_) => {
                    quic_conn.on_timeout();
                }
            }
        }

        // Store in pool
        pool.quic_conn = Some(quic_conn);
        pool.h3_conn = h3_conn;
        pool.socket = Some(Arc::new(socket));
        pool.local_addr = Some(local_addr);
        pool.peer_addr = Some(peer_addr);
        pool.failed = false;

        Ok(())
    }

    pub async fn send_request(
        &mut self,
        target: &str,
        port: u16,
        host: &str,
        path: &str,
        verbose: bool,
    ) -> Result<ResponseResult, Box<dyn std::error::Error>> {
        let start = Instant::now();

        // Ensure connection is established (reuses if available)
        self.ensure_connected(target, port, host).await?;

        // Increment reuse count for metrics
        {
            let pool = &mut self.pool;
            if pool.quic_conn.is_none() || pool.h3_conn.is_none() || pool.socket.is_none() {
                return Err("Connection lost".into());
            }
            pool.reuse_count += 1;
        }

        let mut errors = ErrorStats::default();
        let mut out = [0u8; 65_535];
        let mut buf = [0u8; 65_535];
        let mut stream_id: Option<u64> = None;
        let mut request_sent = false;

        // Send request on new stream
        {
            let pool = &mut self.pool;
            let quic_conn = pool.quic_conn.as_mut().ok_or("Connection lost")?;
            let h3_conn = pool.h3_conn.as_mut().ok_or("Connection lost")?;

            let req = vec![
                Header::new(b":method", b"GET"),
                Header::new(b":scheme", b"https"),
                Header::new(b":authority", host.as_bytes()),
                Header::new(b":path", path.as_bytes()),
                Header::new(b"user-agent", b"vex-h3-client"),
            ];
            h3_conn.send_request(quic_conn, &req, true)?;
            request_sent = true;
        }

        // Flush QUIC packets and handle response with minimal locking
        let mut response_done = false;
        let mut status_code = 0u16;
        let mut bytes_received = 0;
        let mut response_body = Vec::new();

        while !response_done && start.elapsed() < Duration::from_secs(10) {
            // Get socket and local_addr outside the critical section
            let (socket, local_addr) = {
                let pool = &self.pool;
                (pool.socket.clone().ok_or("Socket lost")?, pool.local_addr.ok_or("Addr lost")?)
            };

            // Receive and process packets
            {
                let pool = &mut self.pool;
                let quic_conn = pool.quic_conn.as_mut().ok_or("Connection lost")?;

                let timeout = if request_sent {
                    quic_conn.timeout().unwrap_or(Duration::from_millis(100))
                } else {
                    Duration::from_millis(50)
                };

                match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
                    Ok(Ok((len, from))) => {
                        let recv_info = quiche::RecvInfo { from, to: local_addr };
                        let _ = quic_conn.recv(&mut buf[..len], recv_info);
                    }
                    Ok(Err(e)) => {
                        eprintln!("socket recv_from error: {}", e);
                        errors.recv_errors += 1;
                    }
                    Err(_) => {
                        quic_conn.on_timeout();
                    }
                }

                // Send pending packets
                loop {
                    match quic_conn.send(&mut out) {
                        Ok((write, send_info)) => {
                            if let Err(e) = socket.send_to(&out[..write], send_info.to).await {
                                eprintln!("send_to failed: {}", e);
                                errors.send_errors += 1;
                            }
                        }
                        Err(quiche::Error::Done) => break,
                        Err(_) => break,
                    }
                }
            }

            // Poll for stream events
            {
                let pool = &mut self.pool;
                let quic_conn = pool.quic_conn.as_mut().ok_or("Connection lost")?;
                let h3_conn = pool.h3_conn.as_mut().ok_or("Connection lost")?;

                if quic_conn.is_closed() {
                    break;
                }

                loop {
                    match h3_conn.poll(quic_conn) {
                        Ok((id, quiche::h3::Event::Headers { list, .. })) => {
                            if stream_id.is_none() {
                                stream_id = Some(id);
                            }
                            // Only process headers for our stream
                            if stream_id == Some(id) {
                                for h in list {
                                    let name = String::from_utf8_lossy(h.name());
                                    let value = String::from_utf8_lossy(h.value());

                                    if name == ":status" {
                                        if let Ok(code) = value.parse::<u16>() {
                                            status_code = code;
                                        }
                                    }

                                    if verbose {
                                        println!("{name}: {value}");
                                    }
                                }
                            }
                        }
                        Ok((id, quiche::h3::Event::Data)) => {
                            if stream_id.is_none() {
                                stream_id = Some(id);
                            }
                            // Only process data for our stream
                            if stream_id == Some(id) {
                                loop {
                                    match h3_conn.recv_body(quic_conn, id, &mut buf) {
                                        Ok(read) => {
                                            bytes_received += read;
                                            if verbose {
                                                response_body.extend_from_slice(&buf[..read]);
                                            }
                                        }
                                        Err(quiche::h3::Error::Done) => break,
                                        Err(e) => {
                                            eprintln!("recv_body error: {:?}", e);
                                            errors.quic_errors += 1;
                                        }
                                    }
                                }
                            }
                        }
                        Ok((id, quiche::h3::Event::Finished)) => {
                            // Only mark done if this is our stream
                            if stream_id == Some(id) {
                                response_done = true;
                                break;
                            }
                        }
                        Ok((_id, quiche::h3::Event::PriorityUpdate)) => {}
                        Ok((_id, quiche::h3::Event::GoAway)) => {
                            pool.failed = true;
                            response_done = true;
                            break;
                        }
                        Ok((_id, quiche::h3::Event::Reset(sid))) => {
                            if stream_id == Some(sid) {
                                eprintln!("Stream reset by peer");
                                errors.stream_reset_errors += 1;
                                response_done = true;
                                break;
                            }
                        }
                        Err(quiche::h3::Error::Done) => break,
                        Err(e) => {
                            eprintln!("h3 poll error: {:?}", e);
                            errors.quic_errors += 1;
                            break;
                        }
                    }
                }
            }
        }

        if start.elapsed() >= Duration::from_secs(10) && !response_done {
            let pool = &mut self.pool;
            pool.failed = true;
            return Err("timeout waiting for response".into());
        }

        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        let body = if verbose {
            Some(String::from_utf8_lossy(&response_body).to_string())
        } else {
            None
        };

        Ok(ResponseResult {
            status_code,
            bytes_received,
            errors,
            latency_ms,
            body,
        })
    }
}
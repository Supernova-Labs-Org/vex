use quiche::{self, h3::{Header, NameValue}};
use rand::RngCore;
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    time::{Duration, Instant},
    sync::Arc,
};
use tokio::net::UdpSocket;
use super::{constants, pool::{ConnectionPoolState, ErrorStats, ResponseResult}};

fn bind_addr_for_peer(peer_addr: SocketAddr) -> SocketAddr {
    if peer_addr.is_ipv6() {
        SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0))
    } else {
        SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))
    }
}

pub struct Http3Client {
    pub insecure: bool,
    peer_addr: SocketAddr,
    pool: ConnectionPoolState,
}

impl Http3Client {
    pub fn new(insecure: bool, peer_addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            insecure,
            peer_addr,
            pool: ConnectionPoolState::default(),
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
        // Reuse the existing connection if it is healthy.
        if self.pool.is_usable() {
            return Ok(());
        }

        let peer_addr = self.peer_addr;
        let bind_addr = bind_addr_for_peer(peer_addr);
        let socket = UdpSocket::bind(bind_addr).await?;
        let local_addr = socket.local_addr()?;

        // Create new QUIC connection
        let mut scid_bytes = [0u8; quiche::MAX_CONN_ID_LEN];
        rand::thread_rng().fill_bytes(&mut scid_bytes);
        let scid = quiche::ConnectionId::from_ref(&scid_bytes);

        let mut config = self.build_quic_config()?;
        let mut quic_conn = quiche::connect(Some(server_name), &scid, local_addr, peer_addr, &mut config)?;

        // Perform handshake
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

            // Give other tasks a chance to run
            tokio::task::yield_now().await;

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
            let timeout = quic_conn.timeout().unwrap_or(Duration::from_millis(constants::network::HANDSHAKE_POLL_TIMEOUT_MS));
            match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
                Ok(Ok((len, from))) => {
                    let recv_info = quiche::RecvInfo { from, to: local_addr };
                    match quic_conn.recv(&mut buf[..len], recv_info) {
                        Ok(_) | Err(quiche::Error::Done) => {}
                        Err(err) => {
                            return Err(format!("quic recv failed during handshake: {:?}", err).into());
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

        // Store in pool
        self.pool.quic_conn = Some(quic_conn);
        self.pool.h3_conn = h3_conn;
        self.pool.socket = Some(Arc::new(socket));
        self.pool.local_addr = Some(local_addr);
        self.pool.peer_addr = Some(peer_addr);
        self.pool.failed = false;

        Ok(())
    }

    pub async fn send_request(
        &mut self,
        server_name: &str,
        authority: &str,
        path: &str,
        verbose: bool,
    ) -> Result<ResponseResult, Box<dyn std::error::Error>> {
        let start = Instant::now();

        // Ensure connection is established (reuses if available)
        self.ensure_connected(server_name).await?;

        // Verify the connection is actually usable before proceeding
        {
            let pool = &mut self.pool;
            if !pool.is_usable() {
                return Err("Connection lost".into());
            }
            pool.reuse_count += 1;
        }

        let mut errors = ErrorStats::default();
        let mut out = [0u8; constants::network::BUFFER_SIZE];
        let mut buf = [0u8; constants::network::BUFFER_SIZE];

        // Send request and capture the stream ID quiche assigned to it.
        let stream_id = {
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
            h3_conn.send_request(quic_conn, &req, true)?
        };

        // Flush QUIC packets and handle response with minimal locking
        let mut response_done = false;
        let mut premature_close = false;
        let mut status_code: Option<u16> = None;
        let mut bytes_received = 0;
        let mut response_body = Vec::new();

        while !response_done && start.elapsed() < Duration::from_secs(constants::network::RESPONSE_TIMEOUT_SECS) {
            // Get socket and local_addr outside the critical section
            let (socket, local_addr) = {
                let pool = &self.pool;
                (pool.socket.clone().ok_or("Socket lost")?, pool.local_addr.ok_or("Addr lost")?)
            };

            // Receive and process packets
            {
                let pool = &mut self.pool;
                let quic_conn = pool.quic_conn.as_mut().ok_or("Connection lost")?;

                let timeout = quic_conn.timeout().unwrap_or(Duration::from_millis(constants::network::RESPONSE_POLL_TIMEOUT_MS));

                match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
                    Ok(Ok((len, from))) => {
                        let recv_info = quiche::RecvInfo { from, to: local_addr };
                        match quic_conn.recv(&mut buf[..len], recv_info) {
                            Ok(_) | Err(quiche::Error::Done) => {}
                            Err(err) => {
                                eprintln!("quic recv failed: {:?}", err);
                                errors.quic_errors += 1;
                            }
                        }
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
                    pool.mark_failed();
                    premature_close = true;
                    break;
                }

                loop {
                    match h3_conn.poll(quic_conn) {
                        Ok((id, quiche::h3::Event::Headers { list, .. })) => {
                            if id != stream_id {
                                continue;
                            }
                            for h in list {
                                let name = String::from_utf8_lossy(h.name());
                                let value = String::from_utf8_lossy(h.value());

                                if name == ":status"
                                    && let Ok(code) = value.parse::<u16>()
                                {
                                    status_code = Some(code);
                                }

                                if verbose {
                                    println!("{name}: {value}");
                                }
                            }
                        }
                        Ok((id, quiche::h3::Event::Data)) => {
                            if id != stream_id {
                                continue;
                            }
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
                                        response_done = true;
                                        break;
                                    }
                                }
                            }
                        }
                        Ok((id, quiche::h3::Event::Finished)) => {
                            if id == stream_id {
                                response_done = true;
                                break;
                            }
                        }
                        Ok((_id, quiche::h3::Event::PriorityUpdate)) => {}
                        Ok((_id, quiche::h3::Event::GoAway)) => {
                            pool.mark_failed();
                            response_done = true;
                            break;
                        }
                        Ok((_id, quiche::h3::Event::Reset(sid))) => {
                            if sid == stream_id {
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

        if start.elapsed() >= Duration::from_secs(constants::network::RESPONSE_TIMEOUT_SECS) && !response_done {
            self.pool.mark_failed();
            return Err("timeout waiting for response".into());
        }

        if premature_close {
            return Err("connection closed before response completed".into());
        }

        let status_code = match status_code {
            Some(code) => code,
            None => return Err("response completed without :status header".into()),
        };

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

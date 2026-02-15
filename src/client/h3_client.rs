use quiche::{self, h3::Header};
use rand::RngCore;
use std::{
    net::{UdpSocket, SocketAddr},
    time::{Duration, Instant},
};
use quiche::h3::NameValue;
use crate::utils::resolve_target;

pub struct Http3Client {
    config: quiche::Config,
    pub insecure: bool,
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

        Ok(Self { config, insecure })
    }

    pub async fn send_request(
        &mut self,
        target: &str,
        host: &str,
        path: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Resolve target
        let peer_addr: SocketAddr = resolve_target(target, 443)?;

        // Bind local UDP socket
        let bind_addr: SocketAddr = "0.0.0.0:0".parse()?;
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;
        let local_addr = socket.local_addr()?;

        // QUIC connection ID
        let mut scid_bytes = [0u8; quiche::MAX_CONN_ID_LEN];
        rand::thread_rng().fill_bytes(&mut scid_bytes);
        let scid = quiche::ConnectionId::from_ref(&scid_bytes);

        // Connect
        let mut conn = quiche::connect(Some(host), &scid, local_addr, peer_addr, &mut self.config)?;
        let mut h3_conn: Option<quiche::h3::Connection> = None;
        let mut req_sent = false;
        let mut response_done = false;
        let mut response_body = Vec::new();

        let mut out = [0u8; 65_535];
        let mut buf = [0u8; 65_535];
        let start = Instant::now();
        let mut last_timeout = Instant::now();

        // Initial packet send
        if let Ok((write, send_info)) = conn.send(&mut out) {
            let _ = socket.send_to(&out[..write], send_info.to);
        }

        while !response_done && !conn.is_closed() {
            // Send pending packets
            loop {
                match conn.send(&mut out) {
                    Ok((write, send_info)) => {
                        let _ = socket.send_to(&out[..write], send_info.to);
                    }
                    Err(quiche::Error::Done) => break,
                    Err(e) => return Err(format!("send failed: {:?}", e).into()),
                }
            }

            // Set read timeout
            let timeout = conn.timeout().unwrap_or(Duration::from_millis(50));
            socket.set_read_timeout(Some(timeout))?;

            // Receive packets
            match socket.recv_from(&mut buf) {
                Ok((len, from)) => {
                    let recv_info = quiche::RecvInfo { from, to: local_addr };
                    let _ = conn.recv(&mut buf[..len], recv_info);
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    let now = Instant::now();
                    if now >= last_timeout + timeout {
                        conn.on_timeout();
                        last_timeout = now;
                    }
                }
                Err(e) => return Err(e.into()),
            }

            // Initialize H3 once QUIC established
            if conn.is_established() && h3_conn.is_none() {
                let h3_config = quiche::h3::Config::new()?;
                h3_conn = Some(quiche::h3::Connection::with_transport(&mut conn, &h3_config)?);
            }

            // Send request
            if let Some(h3) = h3_conn.as_mut() {
                if !req_sent {
                    let req = vec![
                        Header::new(b":method", b"GET"),
                        Header::new(b":scheme", b"https"),
                        Header::new(b":authority", host.as_bytes()),
                        Header::new(b":path", path.as_bytes()),
                        Header::new(b"user-agent", b"vex-h3-client"),
                    ];
                    h3.send_request(&mut conn, &req, true)?;
                    req_sent = true;
                }

                // Poll for events
                loop {
                    match h3.poll(&mut conn) {
                        Ok((_id, quiche::h3::Event::Headers { list, .. })) => {
                            for h in list {
                                let name = String::from_utf8_lossy(h.name());
                                let value = String::from_utf8_lossy(h.value());
                                println!("{name}: {value}");
                            }
                        }
                        Ok((stream_id, quiche::h3::Event::Data)) => {
                            loop {
                                match h3.recv_body(&mut conn, stream_id, &mut buf) {
                                    Ok(read) => response_body.extend_from_slice(&buf[..read]),
                                    Err(quiche::h3::Error::Done) => break,
                                    Err(e) => return Err(format!("recv_body failed: {:?}", e).into()),
                                }
                            }
                        }
                        Ok((_id, quiche::h3::Event::Finished)) => {
                            response_done = true;
                            break;
                        }
                        Ok((_id, quiche::h3::Event::PriorityUpdate)) => {}
                        Ok((_id, quiche::h3::Event::GoAway)) => {
                            response_done = true;
                            break;
                        }
                        Ok((_id, quiche::h3::Event::Reset(_))) => {
                            return Err("stream reset by peer".into());
                        }
                        Err(quiche::h3::Error::Done) => break,
                        Err(e) => return Err(format!("h3 poll failed: {:?}", e).into()),
                    }
                }
            }

            // Timeout safeguard
            if start.elapsed() > Duration::from_secs(5) && !response_done {
                return Err("timeout waiting for response".into());
            }
        }

        Ok(String::from_utf8_lossy(&response_body).to_string())
    }
}
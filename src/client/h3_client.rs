use quiche::{self, h3::{Header, NameValue}};
use rand::RngCore;
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};
use tokio::net::UdpSocket;
use crate::utils::resolve_target;

#[derive(Debug, Clone, Default)]
pub struct ErrorStats {
    pub send_errors: usize,
    pub recv_errors: usize,
    pub quic_errors: usize,
    pub stream_reset_errors: usize,
}

#[derive(Debug, Clone)]
pub struct ResponseResult {
    pub status_code: u16,
    pub bytes_received: usize,
    pub errors: ErrorStats,
    pub latency_ms: f64,
    /// Body content only captured in verbose mode for debugging
    pub body: Option<String>,
}

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
        port: u16,
        host: &str,
        path: &str,
        verbose: bool,
    ) -> Result<ResponseResult, Box<dyn std::error::Error>> {
        // Resolve target
        let peer_addr: SocketAddr = resolve_target(target, port)?;

        // Bind local UDP socket
        let bind_addr: SocketAddr = "0.0.0.0:0".parse()?;
        let socket = UdpSocket::bind(bind_addr).await?;
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
        let mut bytes_received: usize = 0;
        let mut response_body = Vec::new();
        let mut status_code = 0u16;

        let mut out = [0u8; 65_535];
        let mut buf = [0u8; 65_535];
        let start = Instant::now();
        let mut errors = ErrorStats::default();

        // Initial packet send
        if let Ok((write, send_info)) = conn.send(&mut out)
            && let Err(e) = socket.send_to(&out[..write], send_info.to).await
        {
            eprintln!("Initial send_to failed: {}", e);
            errors.send_errors += 1;
        }

        while !response_done && !conn.is_closed() {
            // Send pending packets
            loop {
                match conn.send(&mut out) {
                    Ok((write, send_info)) => {
                        if let Err(e) = socket.send_to(&out[..write], send_info.to).await {
                            eprintln!("send_to failed: {}", e);
                            errors.send_errors += 1;
                        }
                    }
                    Err(quiche::Error::Done) => break,
                    Err(e) => return Err(format!("send failed: {:?}", e).into()),
                }
            }

            // Get quiche timeout and wait for packet or timeout
            let timeout = conn.timeout().unwrap_or(Duration::from_millis(50));

            match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
                Ok(Ok((len, from))) => {
                    // Packet received, process it
                    let recv_info = quiche::RecvInfo { from, to: local_addr };
                    if let Err(e) = conn.recv(&mut buf[..len], recv_info) {
                        eprintln!("conn.recv error: {:?}", e);
                        errors.quic_errors += 1;
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("socket.recv_from error: {}", e);
                    errors.recv_errors += 1;
                }
                Err(_) => {
                    // Timeout expired, notify quiche
                    conn.on_timeout();
                }
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

                                // Parse :status header
                                if name == ":status" && let Ok(code) = value.parse::<u16>() {
                                    status_code = code;
                                }

                                // Only print headers in verbose mode
                                if verbose {
                                    println!("{name}: {value}");
                                }
                            }
                        }
                        Ok((stream_id, quiche::h3::Event::Data)) => {
                            loop {
                                match h3.recv_body(&mut conn, stream_id, &mut buf) {
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
                            eprintln!("Stream reset by peer");
                            errors.stream_reset_errors += 1;
                            response_done = true;
                            break;
                        }
                        Err(quiche::h3::Error::Done) => break,
                        Err(e) => {
                            eprintln!("h3 poll error: {:?}", e);
                            errors.quic_errors += 1;
                        }
                    }
                }
            }

            // Timeout safeguard
            if start.elapsed() > Duration::from_secs(5) && !response_done {
                return Err("timeout waiting for response".into());
            }
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
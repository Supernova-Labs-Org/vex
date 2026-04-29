use serde_json::Value;
use std::net::{SocketAddr, UdpSocket};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn run_single_request_h3_server() -> (u16, thread::JoinHandle<Result<(), String>>) {
    let (tx, rx) = mpsc::channel::<u16>();

    let handle = thread::spawn(move || {
        let socket = UdpSocket::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        socket
            .set_read_timeout(Some(Duration::from_millis(50)))
            .map_err(|e| e.to_string())?;
        let local_addr = socket.local_addr().map_err(|e| e.to_string())?;
        tx.send(local_addr.port()).map_err(|e| e.to_string())?;

        let mut config =
            quiche::Config::new(quiche::PROTOCOL_VERSION).map_err(|e| e.to_string())?;
        config
            .load_cert_chain_from_pem_file("tests/fixtures/server.crt")
            .map_err(|e| e.to_string())?;
        config
            .load_priv_key_from_pem_file("tests/fixtures/server.key")
            .map_err(|e| e.to_string())?;
        config
            .set_application_protos(quiche::h3::APPLICATION_PROTOCOL)
            .map_err(|e| e.to_string())?;
        config.set_max_idle_timeout(5_000);
        config.set_max_recv_udp_payload_size(65_527);
        config.set_max_send_udp_payload_size(65_527);
        config.set_initial_max_data(10_000_000);
        config.set_initial_max_stream_data_bidi_local(1_000_000);
        config.set_initial_max_stream_data_bidi_remote(1_000_000);
        config.set_initial_max_stream_data_uni(1_000_000);
        config.set_initial_max_streams_bidi(100);
        config.set_initial_max_streams_uni(100);
        config.verify_peer(false);

        let h3_config = quiche::h3::Config::new().map_err(|e| e.to_string())?;
        let mut conn: Option<quiche::Connection> = None;
        let mut h3_conn: Option<quiche::h3::Connection> = None;
        let mut peer: Option<SocketAddr> = None;
        let mut responded = false;

        let started = Instant::now();
        let mut buf = [0u8; 65_535];
        let mut out = [0u8; 65_535];

        while started.elapsed() < Duration::from_secs(8) {
            match socket.recv_from(&mut buf) {
                Ok((len, from)) => {
                    let pkt = &mut buf[..len];
                    let hdr = match quiche::Header::from_slice(pkt, quiche::MAX_CONN_ID_LEN) {
                        Ok(h) => h,
                        Err(_) => continue,
                    };

                    if hdr.version != quiche::PROTOCOL_VERSION {
                        let len = quiche::negotiate_version(&hdr.scid, &hdr.dcid, &mut out)
                            .map_err(|e| e.to_string())?;
                        let _ = socket.send_to(&out[..len], from);
                        continue;
                    }

                    if conn.is_none() {
                        if hdr.ty != quiche::Type::Initial {
                            continue;
                        }
                        let scid_buf = [0xba; quiche::MAX_CONN_ID_LEN];
                        let scid = quiche::ConnectionId::from_ref(&scid_buf);
                        let accepted = quiche::accept(&scid, None, local_addr, from, &mut config)
                            .map_err(|e| e.to_string())?;
                        conn = Some(accepted);
                        peer = Some(from);
                    }

                    let conn_ref = conn.as_mut().expect("connection just initialized");
                    let recv_info = quiche::RecvInfo {
                        from,
                        to: local_addr,
                    };

                    let _ = conn_ref.recv(pkt, recv_info);

                    if conn_ref.is_established() && h3_conn.is_none() {
                        let h3 = quiche::h3::Connection::with_transport(conn_ref, &h3_config)
                            .map_err(|e| e.to_string())?;
                        h3_conn = Some(h3);
                    }

                    if let Some(h3) = h3_conn.as_mut() {
                        loop {
                            match h3.poll(conn_ref) {
                                Ok((stream_id, quiche::h3::Event::Headers { .. })) => {
                                    let headers = vec![
                                        quiche::h3::Header::new(b":status", b"200"),
                                        quiche::h3::Header::new(b"server", b"vex-test"),
                                    ];
                                    if h3
                                        .send_response(conn_ref, stream_id, &headers, true)
                                        .is_ok()
                                    {
                                        responded = true;
                                    }
                                }
                                Ok((_id, quiche::h3::Event::Data)) => {}
                                Ok((_id, quiche::h3::Event::Finished)) => {}
                                Ok((_id, quiche::h3::Event::Reset(_))) => {}
                                Ok((_id, quiche::h3::Event::GoAway)) => {}
                                Ok((_id, quiche::h3::Event::PriorityUpdate)) => {}
                                Err(quiche::h3::Error::Done) => break,
                                Err(_) => break,
                            }
                        }
                    }

                    loop {
                        match conn_ref.send(&mut out) {
                            Ok((write, send_info)) => {
                                let _ = socket.send_to(&out[..write], send_info.to);
                            }
                            Err(quiche::Error::Done) => break,
                            Err(_) => break,
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => return Err(e.to_string()),
            }

            if let Some(conn_ref) = conn.as_mut() {
                if conn_ref.timeout().is_some() {
                    conn_ref.on_timeout();
                }

                if let Some(to) = peer {
                    loop {
                        match conn_ref.send(&mut out) {
                            Ok((write, _)) => {
                                let _ = socket.send_to(&out[..write], to);
                            }
                            Err(quiche::Error::Done) => break,
                            Err(_) => break,
                        }
                    }
                }

                if responded && conn_ref.is_closed() {
                    break;
                }
            }
        }

        if responded {
            Ok(())
        } else {
            Err("server did not observe/respond to request".to_string())
        }
    });

    let port = rx.recv().expect("server should provide port");
    (port, handle)
}

#[test]
fn json_mode_works_against_local_h3_server() {
    if std::env::var("VEX_RUN_H3_IT").ok().as_deref() != Some("1") {
        // Opt-in test: exercises a real local QUIC/H3 loop and may fail on
        // environments with restricted UDP or low thread stack limits.
        return;
    }

    let (port, server_handle) = run_single_request_h3_server();

    let output = Command::new(env!("CARGO_BIN_EXE_vex"))
        .args([
            "--target",
            "127.0.0.1",
            "--port",
            &port.to_string(),
            "--workers",
            "1",
            "--requests",
            "1",
            "--duration",
            "5",
            "--insecure",
            "--json",
        ])
        .output()
        .expect("failed to execute vex");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Some restricted environments block UDP; avoid false negatives there.
    if stderr.contains("Operation not permitted") {
        let _ = server_handle.join();
        return;
    }

    assert_eq!(output.status.code(), Some(0), "stderr: {stderr}");
    let parsed: Value = serde_json::from_str(stdout.trim()).expect("stdout must be valid JSON");

    assert_eq!(
        parsed
            .get("requests")
            .and_then(|r| r.get("successful"))
            .and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        parsed
            .get("requests")
            .and_then(|r| r.get("failed"))
            .and_then(Value::as_u64),
        Some(0)
    );

    let server_result = server_handle.join().expect("server thread panicked");
    assert!(server_result.is_ok(), "server result: {server_result:?}");
}

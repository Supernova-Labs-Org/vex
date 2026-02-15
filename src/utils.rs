use std::net::{SocketAddr, ToSocketAddrs};

/// Resolve target + port to SocketAddr
pub fn resolve_target(target: &str, port: u16) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let stripped = target.trim_start_matches("https://").trim_start_matches("http://");
    let mut addrs_iter = format!("{stripped}:{port}").to_socket_addrs()?;
    addrs_iter
        .next()
        .ok_or_else(|| format!("Could not resolve host: {}", target).into())
}
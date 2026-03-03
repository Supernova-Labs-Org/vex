use std::net::{SocketAddr, ToSocketAddrs};

/// Parse target to extract host and optional port
///
/// Supports the following formats:
/// - `hostname` (uses provided port)
/// - `hostname:port` (uses embedded port)
/// - `127.0.0.1` (uses provided port)
/// - `127.0.0.1:port` (uses embedded port)
/// - `[::1]` (IPv6, uses provided port)
/// - `[::1]:port` (IPv6 with embedded port)
/// - `http://hostname` or `https://hostname` (scheme stripped)
///
/// Port precedence: embedded port > provided port
fn parse_target(target: &str, default_port: u16) -> Result<(String, u16), Box<dyn std::error::Error>> {
    // Strip scheme if present
    let stripped = target
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    // Check if this is an IPv6 address (contains colons but starts with '[')
    if stripped.starts_with('[') {
        // IPv6 format: [::1] or [::1]:port
        if let Some(bracket_end) = stripped.find(']') {
            let host = &stripped[..bracket_end + 1]; // Include the brackets
            let remaining = &stripped[bracket_end + 1..];

            let port = if let Some(port_str) = remaining.strip_prefix(':') {
                // Embedded port found
                port_str
                    .parse::<u16>()
                    .map_err(|_| format!("Invalid port in target: {}", target))?
            } else if remaining.is_empty() {
                // No embedded port, use default
                default_port
            } else {
                return Err(format!("Invalid IPv6 format in target: {}", target).into());
            };

            return Ok((host.to_string(), port));
        } else {
            return Err(format!("Malformed IPv6 address in target: {}", target).into());
        }
    }

    // For non-IPv6, check if there's a port by counting colons
    let colon_count = stripped.matches(':').count();
    if colon_count == 0 {
        // No port embedded
        Ok((stripped.to_string(), default_port))
    } else if colon_count == 1 {
        // One colon, should be host:port
        let parts: Vec<&str> = stripped.split(':').collect();
        let host = parts[0].to_string();
        let port = parts[1]
            .parse::<u16>()
            .map_err(|_| format!("Invalid port in target: {}", target))?;
        Ok((host, port))
    } else {
        // Multiple colons without IPv6 brackets is invalid
        Err(format!("Invalid target format: {}", target).into())
    }
}

/// Resolve target + port to SocketAddr
///
/// # Arguments
/// * `target` - The target address (hostname, IPv4, IPv6, or with embedded port)
/// * `port` - Default port if not embedded in target
///
/// # Examples
/// * `resolve_target("example.com", 443)` → `example.com:443`
/// * `resolve_target("example.com:8443", 443)` → `example.com:8443` (embedded port takes precedence)
/// * `resolve_target("[::1]", 443)` → `[::1]:443`
/// * `resolve_target("[::1]:8443", 443)` → `[::1]:8443`
pub fn resolve_target(target: &str, port: u16) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let (host, resolved_port) = parse_target(target, port)?;
    let addr_str = format!("{}:{}", host, resolved_port);
    let mut addrs_iter = addr_str.to_socket_addrs()?;
    addrs_iter
        .next()
        .ok_or_else(|| format!("Could not resolve host: {}", target).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hostname_only() {
        let (host, port) = parse_target("example.com", 443).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_hostname_with_port() {
        let (host, port) = parse_target("example.com:8443", 443).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 8443);
    }

    #[test]
    fn test_ipv4_only() {
        let (host, port) = parse_target("127.0.0.1", 443).unwrap();
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_ipv4_with_port() {
        let (host, port) = parse_target("127.0.0.1:8080", 443).unwrap();
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_ipv6_only() {
        let (host, port) = parse_target("[::1]", 443).unwrap();
        assert_eq!(host, "[::1]");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_ipv6_with_port() {
        let (host, port) = parse_target("[::1]:8443", 443).unwrap();
        assert_eq!(host, "[::1]");
        assert_eq!(port, 8443);
    }

    #[test]
    fn test_ipv6_full_address() {
        let (host, port) = parse_target("[2001:db8::1]", 443).unwrap();
        assert_eq!(host, "[2001:db8::1]");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_ipv6_full_address_with_port() {
        let (host, port) = parse_target("[2001:db8::1]:8443", 443).unwrap();
        assert_eq!(host, "[2001:db8::1]");
        assert_eq!(port, 8443);
    }

    #[test]
    fn test_https_scheme_stripped() {
        let (host, port) = parse_target("https://example.com", 443).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_http_scheme_stripped() {
        let (host, port) = parse_target("http://example.com:8080", 443).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_invalid_port() {
        assert!(parse_target("example.com:invalid", 443).is_err());
    }

    #[test]
    fn test_malformed_ipv6() {
        assert!(parse_target("[::1:8443", 443).is_err());
    }
}
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
pub fn parse_target(target: &str, default_port: u16) -> Result<(String, u16), Box<dyn std::error::Error>> {
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

/// Compute percentile from sorted values using linear interpolation
///
/// # Arguments
/// * `sorted_values` - Values sorted in ascending order
/// * `p` - Percentile value (0-100)
///
/// # Examples
/// * `percentile(&[1.0, 2.0, 3.0], 50.0)` → 2.0 (median)
/// * `percentile(&[1.0, 2.0, 3.0, 4.0, 5.0], 95.0)` → 4.8
pub fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    if sorted_values.len() == 1 {
        return sorted_values[0];
    }

    let idx = (p / 100.0) * (sorted_values.len() - 1) as f64;
    let lower = idx.floor() as usize;
    let upper = idx.ceil() as usize;
    let weight = idx - idx.floor();

    if lower == upper {
        sorted_values[lower]
    } else {
        sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight
    }
}

/// Parse success status pattern and check if a status code is considered success
///
/// # Arguments
/// * `status_code` - HTTP status code (e.g., 200, 301, 404)
/// * `success_pattern` - Pattern string (e.g., "2xx", "2xx,3xx", "200,201,301")
///
/// # Supported patterns
/// - Class patterns: `2xx`, `3xx`, `4xx`, `5xx`
/// - Specific codes: comma-separated list (e.g., `200,201,301`)
/// - Mixed: `2xx,3xx,500`
///
/// # Examples
/// * `is_success_status(200, "2xx")` → true
/// * `is_success_status(301, "2xx")` → false
/// * `is_success_status(301, "2xx,3xx")` → true
/// * `is_success_status(301, "200,201,301")` → true
pub fn is_success_status(status_code: u16, success_pattern: &str) -> bool {
    for part in success_pattern.split(',') {
        let part = part.trim();
        if part == "2xx" && status_code >= 200 && status_code < 300 {
            return true;
        }
        if part == "3xx" && status_code >= 300 && status_code < 400 {
            return true;
        }
        if part == "4xx" && status_code >= 400 && status_code < 500 {
            return true;
        }
        if part == "5xx" && status_code >= 500 && status_code < 600 {
            return true;
        }
        if let Ok(code) = part.parse::<u16>() {
            if status_code == code {
                return true;
            }
        }
    }
    false
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
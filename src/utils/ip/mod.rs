use axum::http::HeaderMap;
use std::net::SocketAddr;

/// NOTE: Extracts a client IP from Forwarded / X-Forwarded-For / X-Real-IP
/// falling back to the peer socket address.
pub fn extract_client_ip(headers: &HeaderMap, addr: &SocketAddr) -> String {
    // 1) RFC 7239 Forwarded
    if let Some(fwd) = headers.get("forwarded") {
        if let Ok(s) = fwd.to_str() {
            for part in s.split(',').map(|p| p.trim()) {
                for kv in part.split(';') {
                    let kv = kv.trim();
                    if let Some(rest) = kv.strip_prefix("for=") {
                        let cleaned = rest.trim_matches('"').trim_matches('[').trim_matches(']');
                        if !cleaned.is_empty() {
                            return cleaned.to_string();
                        }
                    }
                }
            }
        }
    }

    // 2) X-Forwarded-For
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(s) = xff.to_str() {
            if let Some(first) = s.split(',').next() {
                let cleaned = first.trim();
                if !cleaned.is_empty() {
                    return cleaned.to_string();
                }
            }
        }
    }

    // 3) X-Real-IP
    if let Some(xrip) = headers.get("x-real-ip") {
        if let Ok(s) = xrip.to_str() {
            let cleaned = s.trim();
            if !cleaned.is_empty() {
                return cleaned.to_string();
            }
        }
    }

    // 4) Fallback to proxy/peer IP
    addr.ip().to_string()
}

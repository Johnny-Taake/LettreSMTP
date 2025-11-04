use axum::{
    body::Body,
    extract::connect_info::ConnectInfo,
    http::Request,
    middleware::Next,
    response::Response,
};
use std::net::{IpAddr, SocketAddr};

pub async fn real_ip_layer(
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let ip_from_headers: Option<IpAddr> = {
        let headers = req.headers();

        if let Some(v) = headers.get("x-forwarded-for") {
            if let Ok(s) = v.to_str() {
                if let Some(first) = s.split(',').next() {
                    if let Ok(ip) = first.trim().parse::<IpAddr>() {
                        Some(ip)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else if let Some(v) = headers.get("x-real-ip") {
            if let Ok(s) = v.to_str() {
                s.parse::<IpAddr>().ok()
            } else {
                None
            }
        } else {
            None
        }
    };

    let real_ip = if let Some(ip) = ip_from_headers {
        ip
    } else if let Some(conn_info) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
        conn_info.0.ip()
    } else {
        IpAddr::from([127, 0, 0, 1])
    };

    req.extensions_mut().insert(real_ip);

    next.run(req).await
}

use crate::state::AppState;
use std::net::SocketAddr;
use tokio::sync::broadcast;

pub async fn run_server(port: u16) -> anyhow::Result<()> {
    let (tx, _rx) = broadcast::channel(100);
    let app = crate::routes::build_router(AppState { tx, executors: crate::engine::executor::ExecutorRegistry::with_defaults() });

    // Bind local loopback by default to prevent unauthorized network exposure
    let host_str = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let host_ip: std::net::IpAddr = host_str.parse().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    let addr = SocketAddr::new(host_ip, port);
    println!("🚀 ViciEO Full Rust API Engine running on http://{}:{}", host_ip, port);

    // Bind IPv6 localhost as well so http://localhost:9000 never gets ECONNREFUSED on macOS
    if host_ip.is_loopback() {
        let addr_v6 = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], port));
        let app_v6 = app.clone();
        tokio::spawn(async move {
            if let Ok(listener_v6) = tokio::net::TcpListener::bind(addr_v6).await {
                let _ = axum::serve(listener_v6, app_v6).await;
            }
        });
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

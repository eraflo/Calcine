//! The local network port: the same API, for other devices, over HTTPS
//! only, with keys allowed on the network only, from allowed addresses.
//! Off by default.
//!
//! - `tls`: the self-signed certificate and the TLS acceptor
//! - `allow`: which addresses may connect
//! - `limiter`: addresses sending too many invalid keys are turned away

pub mod allow;
pub mod limiter;
pub mod tls;

use std::net::{IpAddr, SocketAddr};

use axum::Extension;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_util::service::TowerToHyperService;
use tokio::sync::{oneshot, watch};
use tokio_rustls::TlsAcceptor;

/// The network port's default: HTTPS, next to the loopback API's 18181.
pub const DEFAULT_PORT: u16 = 18443;

/// The address a network request came from, set per connection.
#[derive(Debug, Clone, Copy)]
pub struct Peer(pub SocketAddr);

/// Serve `router` over TLS on `listener` until the returned sender fires,
/// which also closes the connections still open. Each request carries its
/// connection's [`Peer`].
pub fn spawn(
    listener: tokio::net::TcpListener,
    acceptor: TlsAcceptor,
    router: axum::Router,
) -> oneshot::Sender<()> {
    let (shutdown, mut stop) = oneshot::channel::<()>();
    let (closing, _) = watch::channel(false);
    tokio::spawn(async move {
        loop {
            let accepted = tokio::select! {
                accepted = listener.accept() => accepted,
                _ = &mut stop => break,
            };
            let Ok((stream, address)) = accepted else {
                continue;
            };
            let acceptor = acceptor.clone();
            let router = router.clone().layer(Extension(Peer(address)));
            let mut closed = closing.subscribe();
            tokio::spawn(async move {
                let serve = async {
                    // Clients that don't complete TLS (port scans, wrong
                    // scheme) are just dropped.
                    let Ok(stream) = acceptor.accept(stream).await else {
                        return;
                    };
                    let served = auto::Builder::new(TokioExecutor::new())
                        .serve_connection(TokioIo::new(stream), TowerToHyperService::new(router))
                        .await;
                    if let Err(err) = served {
                        tracing::debug!(%err, %address, "network connection ended");
                    }
                };
                // Turning the port off cuts kept-alive connections too.
                tokio::select! {
                    () = serve => {}
                    _ = closed.changed() => {}
                }
            });
        }
        let _ = closing.send(true);
    });
    shutdown
}

/// The names other devices can reach this PC by: its name, and the address
/// it uses on the local network.
pub fn local_names() -> Vec<String> {
    let mut names = Vec::new();
    if let Some(name) = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .filter(|name| !name.is_empty())
    {
        names.push(name.to_lowercase());
        // Windows answers `name.local` over mDNS.
        names.push(format!("{}.local", name.to_lowercase()));
    }
    if let Some(address) = local_address() {
        names.push(address.to_string());
    }
    names
}

/// The address this PC uses to reach the network. Connecting a UDP socket
/// sends nothing: it only asks the system which interface it would use.
pub fn local_address() -> Option<IpAddr> {
    let socket = std::net::UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    socket.connect(("192.0.2.1", 9)).ok()?;
    let address = socket.local_addr().ok()?.ip();
    (!address.is_unspecified() && !address.is_loopback()).then_some(address)
}

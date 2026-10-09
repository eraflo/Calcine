//! Request checks applied before any route runs:
//!
//! 1. `Host` must be the loopback address the gateway listens on, which
//!    defeats DNS rebinding (a web page resolving its own domain to 127.0.0.1).
//! 2. A browser `Origin`, when present, must be allowed, so web pages can't
//!    use the API even without a CORS preflight.
//! 3. A valid API key (`Authorization: Bearer calcine_…`), unless keys are
//!    turned off, in which case callers are anonymous with limited rights.
//!    On the Ollama port, callers without a key get the same limited rights:
//!    Ollama clients can't send one.
//!
//! The local network port replaces the `Host` check with the client's
//! address (allowed ranges, too many invalid keys), always needs a key
//! allowed on the network, and never lets it reach local files.
//!
//! Bodies are then checked per route (see [`sanitize`]).

pub mod sanitize;

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::caller::Caller;
use crate::error::api_error;
use crate::network::{Peer, allow};
use crate::state::AppState;

/// Paths that answer without a key (liveness only, no data).
const PUBLIC_PATHS: &[&str] = &["/", "/v1", "/v1/", "/api/version"];

/// Which socket a request came in on. Set per listener, never from headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Listener {
    /// The main port (18181 by default).
    Main,
    /// Ollama's port, for apps that only speak Ollama.
    Ollama { port: u16 },
    /// The local network port, over HTTPS, for other devices.
    Network { port: u16 },
}

pub async fn guard(State(app): State<Arc<AppState>>, mut request: Request, next: Next) -> Response {
    let listener = request
        .extensions()
        .get::<Listener>()
        .copied()
        .unwrap_or(Listener::Main);
    let peer = match listener {
        // Other devices: HTTPS, any host name, from allowed addresses only.
        Listener::Network { .. } => match admit(&app, &request) {
            Ok(peer) => Some(peer),
            Err(refused) => return *refused,
        },
        Listener::Main | Listener::Ollama { .. } => {
            let port = match listener {
                Listener::Ollama { port } => port,
                _ => app.port,
            };
            if !host_allowed(request.headers(), port) {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_request_error",
                    "unexpected Host header",
                );
            }
            None
        }
    };

    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    if let Some(origin) = &origin
        && !app.origin_allowed(origin)
    {
        return api_error(
            StatusCode::FORBIDDEN,
            "permission_error",
            &format!(
                "requests from {origin} aren't allowed. Add it to the allowed origins in Calcine."
            ),
        );
    }

    if request.method() == Method::OPTIONS {
        return with_cors(StatusCode::NO_CONTENT.into_response(), origin.as_deref());
    }

    if !PUBLIC_PATHS.contains(&request.uri().path()) {
        match identify(&app, listener, request.headers(), peer) {
            Ok(caller) => {
                request.extensions_mut().insert(caller);
            }
            Err(refused) => return with_cors(*refused, origin.as_deref()),
        }
    }

    with_cors(next.run(request).await, origin.as_deref())
}

/// The address of a network request, when it may connect at all.
fn admit(app: &AppState, request: &Request) -> Result<IpAddr, Box<Response>> {
    let Some(Peer(peer)) = request.extensions().get::<Peer>().copied() else {
        return Err(Box::new(api_error(
            StatusCode::FORBIDDEN,
            "permission_error",
            "unknown client address",
        )));
    };
    let address = peer.ip();
    let allowed = allow::ranges(&app.settings().network_allowed)
        .iter()
        .any(|range| range.contains(address));
    if !allowed {
        return Err(Box::new(api_error(
            StatusCode::FORBIDDEN,
            "permission_error",
            &format!(
                "Calcine doesn't accept connections from {address}. Allow it in Calcine \
                 (Settings → Local API → Local network)."
            ),
        )));
    }
    if let Some(wait) = app.limiter.blocked(address, Instant::now()) {
        return Err(Box::new(api_error(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limit_error",
            &format!(
                "too many invalid API keys from {address}: try again in {} minutes",
                wait.as_secs().div_ceil(60)
            ),
        )));
    }
    Ok(address)
}

/// Who is calling, or why they're refused. Over the network a key is always
/// needed, it must be allowed there, and it never reaches local files.
fn identify(
    app: &AppState,
    listener: Listener,
    headers: &HeaderMap,
    peer: Option<IpAddr>,
) -> Result<Caller, Box<Response>> {
    let network = matches!(listener, Listener::Network { .. });
    let invalid = || {
        if let Some(address) = peer {
            app.limiter.failed(address, Instant::now());
        }
        Box::new(api_error(
            StatusCode::UNAUTHORIZED,
            "authentication_error",
            "invalid API key",
        ))
    };
    let Some(token) = bearer_token(headers) else {
        return match listener {
            Listener::Ollama { .. } => Ok(Caller::OllamaClient),
            Listener::Main if !app.settings().require_api_key => Ok(Caller::Anonymous),
            _ => Err(Box::new(api_error(
                StatusCode::UNAUTHORIZED,
                "authentication_error",
                "missing API key. Create one in Calcine (Server → API keys) and send it as \
                 `Authorization: Bearer <key>`.",
            ))),
        };
    };
    match app.keys.authenticate(token) {
        // Calcine's own token is for its window on this PC.
        Some(Caller::Calcine) if network => Err(invalid()),
        Some(Caller::App(mut key)) if network => {
            if let Some(address) = peer {
                app.limiter.succeeded(address);
            }
            if !key.network {
                return Err(Box::new(api_error(
                    StatusCode::FORBIDDEN,
                    "permission_error",
                    "this API key can't be used from other devices. Allow it in Calcine \
                     (Server → API keys).",
                )));
            }
            // A path from another device would name a file on this PC.
            key.allow_local_files = false;
            Ok(Caller::App(key))
        }
        Some(caller) => Ok(caller),
        None => Err(invalid()),
    }
}

/// A browser origin: `http(s)://host[:port]`, nothing after.
pub(crate) fn is_origin(value: &str) -> bool {
    let Some(rest) = value
        .strip_prefix("http://")
        .or_else(|| value.strip_prefix("https://"))
    else {
        return false;
    };
    // IPv6 hosts are bracketed: `[::1]:5173`.
    let (host, port) = if let Some(end) = rest.find(']').filter(|_| rest.starts_with('[')) {
        let (host, after) = rest.split_at(end + 1);
        match after.strip_prefix(':') {
            Some(port) => (host, Some(port)),
            None if after.is_empty() => (host, None),
            None => return false,
        }
    } else {
        match rest.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (rest, None),
        }
    };
    let host_ok = host == "[::1]"
        || (!host.is_empty()
            && host
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'));
    let port_ok = port.is_none_or(|port| port.parse::<u16>().is_ok_and(|port| port > 0));
    host_ok && port_ok
}

/// `Host` must name the loopback interface on our port.
fn host_allowed(headers: &HeaderMap, port: u16) -> bool {
    let Some(host) = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    [
        format!("127.0.0.1:{port}"),
        format!("localhost:{port}"),
        format!("[::1]:{port}"),
    ]
    .contains(&host)
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    scheme
        .eq_ignore_ascii_case("bearer")
        .then(|| token.trim())
        .filter(|token| !token.is_empty())
}

/// Reflect an allowed origin (callers already rejected the others).
fn with_cors(mut response: Response, origin: Option<&str>) -> Response {
    if let Some(origin) = origin.and_then(|origin| HeaderValue::from_str(origin).ok()) {
        let headers = response.headers_mut();
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        headers.insert(header::VARY, HeaderValue::from_static("Origin"));
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("authorization, content-type"),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, DELETE, OPTIONS"),
        );
        headers.insert(
            header::ACCESS_CONTROL_EXPOSE_HEADERS,
            HeaderValue::from_static(crate::context::FORGOTTEN_HEADER),
        );
        headers.insert(
            header::ACCESS_CONTROL_MAX_AGE,
            HeaderValue::from_static("600"),
        );
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(header::HeaderName, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(name.clone(), HeaderValue::from_str(value).unwrap());
        }
        map
    }

    #[test]
    fn only_loopback_hosts_on_our_port_are_accepted() {
        assert!(host_allowed(
            &headers(&[(header::HOST, "127.0.0.1:18181")]),
            18181
        ));
        assert!(host_allowed(
            &headers(&[(header::HOST, "LOCALHOST:18181")]),
            18181
        ));
        assert!(!host_allowed(
            &headers(&[(header::HOST, "evil.example:18181")]),
            18181
        ));
        assert!(!host_allowed(
            &headers(&[(header::HOST, "127.0.0.1:9999")]),
            18181
        ));
        assert!(!host_allowed(&HeaderMap::new(), 18181));
    }

    #[test]
    fn recognises_origins() {
        for origin in [
            "http://localhost:3000",
            "https://example.com",
            "http://127.0.0.1:8080",
            "http://[::1]:5173",
        ] {
            assert!(is_origin(origin), "{origin}");
        }
        for value in [
            "localhost:3000",
            "http://localhost:3000/app",
            "http://",
            "ftp://example.com",
            "http://exa mple.com",
            "http://localhost:99999",
        ] {
            assert!(!is_origin(value), "{value}");
        }
    }

    #[test]
    fn reads_bearer_tokens() {
        let auth = |value| headers(&[(header::AUTHORIZATION, value)]);
        assert_eq!(
            bearer_token(&auth("Bearer calcine_abc")),
            Some("calcine_abc")
        );
        assert_eq!(
            bearer_token(&auth("bearer  calcine_abc ")),
            Some("calcine_abc")
        );
        assert_eq!(bearer_token(&auth("Basic xyz")), None);
        assert_eq!(bearer_token(&auth("Bearer ")), None);
    }
}

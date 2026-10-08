//! Request checks applied before any route runs:
//!
//! 1. `Host` must be the loopback address the gateway listens on, which
//!    defeats DNS rebinding (a web page resolving its own domain to 127.0.0.1).
//! 2. A browser `Origin`, when present, must be allowed, so web pages can't
//!    use the API even without a CORS preflight.
//! 3. A valid API key (`Authorization: Bearer calcine_…`), unless keys are
//!    turned off, in which case callers are anonymous with limited rights.
//!
//! Bodies are then checked per route (see [`sanitize`]).

pub mod sanitize;

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::caller::Caller;
use crate::error::api_error;
use crate::state::AppState;

/// Paths that answer without a key (liveness only, no data).
const PUBLIC_PATHS: &[&str] = &["/v1", "/v1/"];

pub async fn guard(State(app): State<Arc<AppState>>, mut request: Request, next: Next) -> Response {
    if !host_allowed(request.headers(), app.port) {
        return api_error(
            StatusCode::BAD_REQUEST,
            "invalid_request_error",
            "unexpected Host header",
        );
    }

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
        let caller = match bearer_token(request.headers()) {
            Some(token) => match app.keys.authenticate(token) {
                Some(caller) => caller,
                None => {
                    return with_cors(
                        api_error(
                            StatusCode::UNAUTHORIZED,
                            "authentication_error",
                            "invalid API key",
                        ),
                        origin.as_deref(),
                    );
                }
            },
            None if !app.settings().require_api_key => Caller::Anonymous,
            None => {
                return with_cors(
                    api_error(
                        StatusCode::UNAUTHORIZED,
                        "authentication_error",
                        "missing API key. Create one in Calcine (Server → API keys) and send it as \
                         `Authorization: Bearer <key>`.",
                    ),
                    origin.as_deref(),
                );
            }
        };
        request.extensions_mut().insert(caller);
    }

    with_cors(next.run(request).await, origin.as_deref())
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

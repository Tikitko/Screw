use hyper::http::{Extensions, HeaderValue};
use hyper::{upgrade, upgrade::Upgraded, HeaderMap, Uri};
use hyper::{Body, Request, Response, StatusCode, Version};
use std::future::Future;
use tokio::task;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::http::Method;
use tokio_tungstenite::tungstenite::protocol::{Role, WebSocketConfig};
use tokio_tungstenite::WebSocketStream;

fn is_get_method(request: &Request<Body>) -> bool {
    request.method() == Method::GET
}
fn is_http_version_11_or_larger(request: &Request<Body>) -> bool {
    request.version() >= Version::HTTP_11
}
fn is_connection_header_upgrade(request: &Request<Body>) -> bool {
    request
        .headers()
        .get("Connection")
        .and_then(|h| h.to_str().ok())
        .map(|h| {
            h.split(|c| c == ' ' || c == ',')
                .any(|p| p.eq_ignore_ascii_case("Upgrade"))
        })
        .unwrap_or(false)
}
fn is_upgrade_header_web_socket(request: &Request<Body>) -> bool {
    request
        .headers()
        .get("Upgrade")
        .and_then(|h| h.to_str().ok())
        .map(|h| h.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}
fn is_web_socket_version_header_13(request: &Request<Body>) -> bool {
    request
        .headers()
        .get("Sec-WebSocket-Version")
        .map(|h| h == "13")
        .unwrap_or(false)
}
fn get_web_socket_key_header(request: &Request<Body>) -> Option<&HeaderValue> {
    request.headers().get("Sec-WebSocket-Key")
}

fn upgrade_response(request: &Request<Body>) -> Result<Response<Body>, ProtocolError> {
    if !is_get_method(&request) {
        return Err(ProtocolError::WrongHttpMethod);
    }

    if !is_http_version_11_or_larger(&request) {
        return Err(ProtocolError::WrongHttpVersion);
    }

    if !is_connection_header_upgrade(&request) {
        return Err(ProtocolError::MissingConnectionUpgradeHeader);
    }

    if !is_upgrade_header_web_socket(&request) {
        return Err(ProtocolError::MissingUpgradeWebSocketHeader);
    }

    if !is_web_socket_version_header_13(&request) {
        return Err(ProtocolError::MissingSecWebSocketVersionHeader);
    }

    let key = get_web_socket_key_header(&request).ok_or(ProtocolError::MissingSecWebSocketKey)?;

    let response = Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .version(request.version())
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Accept", derive_accept_key(key.as_bytes()))
        .body(Body::empty())
        .unwrap();

    Ok(response)
}

async fn upgrade_to_stream(
    request: &mut Request<Body>,
    config: Option<WebSocketConfig>,
) -> Result<WebSocketStream<Upgraded>, hyper::Error> {
    match upgrade::on(request).await {
        Ok(upgraded) => Ok(WebSocketStream::from_raw_socket(upgraded, Role::Server, config).await),
        Err(error) => Err(error),
    }
}

pub type UpgradeResult = Result<
    (
        WebSocketStream<Upgraded>,
        Uri,
        HeaderMap<HeaderValue>,
        Extensions,
    ),
    hyper::Error,
>;

pub fn upgrade<HFn, HFut>(
    request: Request<Body>,
    config: Option<WebSocketConfig>,
    upgrade_result_handler: HFn,
) -> Response<Body>
where
    HFn: FnOnce(UpgradeResult) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = ()> + Send + 'static,
{
    match upgrade_response(&request) {
        Ok(response) => {
            task::spawn(async move {
                let mut request = request;
                let stream_result = upgrade_to_stream(&mut request, config).await;
                let upgrade_result = stream_result.map(|stream| {
                    let (parts, _) = request.into_parts();
                    (stream, parts.uri, parts.headers, parts.extensions)
                });
                upgrade_result_handler(upgrade_result).await;
            });
            response
        }
        Err(_) => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .unwrap(),
    }
}

pub fn is_upgrade_request(request: &Request<Body>) -> bool {
    is_connection_header_upgrade(request) && is_upgrade_header_web_socket(request)
}

use crate::web_socket_request::{WebSocketRequest, WebSocketResponse, WebSocketUpgrade};
use async_trait::async_trait;
use futures_util::{future, ready, FutureExt, Sink, Stream, TryFutureExt};
use hyper::header::HeaderValue;
use hyper::server::conn::Connection;
use hyper::{upgrade, Body, Method, StatusCode, Version};
use screw_core::routing::router::{Request, RequestConverter, Response, ResponseConverter};
use screw_core::DFn;
use std::sync::Arc;
use tokio::task;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::protocol::{Role, WebSocketConfig};
use tokio_tungstenite::tungstenite::WebSocket;
use tokio_tungstenite::WebSocketStream;

fn is_get_method(request: &Request) -> bool {
    request.method() == Method::GET
}
fn is_http_version_11_or_larger(request: &Request) -> bool {
    request.version() >= Version::HTTP_11
}
fn is_connection_header_upgrade(request: &Request) -> bool {
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
fn is_upgrade_header_web_socket(request: &Request) -> bool {
    request
        .headers()
        .get("Upgrade")
        .and_then(|h| h.to_str().ok())
        .map(|h| h.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}
fn is_web_socket_version_header_13(request: &Request) -> bool {
    request
        .headers()
        .get("Sec-WebSocket-Version")
        .map(|h| h == "13")
        .unwrap_or(false)
}
fn get_web_socket_key_header(request: &Request) -> Option<&HeaderValue> {
    request.headers().get("Sec-WebSocket-Key")
}

pub enum WebSocketConverterError {
    HyperError(hyper::Error),
    ProtocolError(ProtocolError),
}

pub struct WebSocketConverter {
    config: Option<WebSocketConfig>,
    error_logger: Arc<dyn Fn(WebSocketConverterError) + Send + Sync + 'static>,
}

#[async_trait]
impl RequestConverter<WebSocketRequest> for WebSocketConverter {
    async fn convert_request(&self, mut request: Request) -> WebSocketRequest {
        fn try_upgrade(request: &mut Request) -> Result<WebSocketUpgrade, ProtocolError> {
            if !is_get_method(request) {
                return Err(ProtocolError::WrongHttpMethod);
            }

            if !is_http_version_11_or_larger(request) {
                return Err(ProtocolError::WrongHttpVersion);
            }

            if !is_connection_header_upgrade(request) {
                return Err(ProtocolError::MissingConnectionUpgradeHeader);
            }

            if !is_upgrade_header_web_socket(request) {
                return Err(ProtocolError::MissingUpgradeWebSocketHeader);
            }

            if !is_web_socket_version_header_13(request) {
                return Err(ProtocolError::MissingSecWebSocketVersionHeader);
            }

            let key = derive_accept_key(
                get_web_socket_key_header(request)
                    .ok_or(ProtocolError::MissingSecWebSocketKey)?
                    .as_bytes(),
            );

            let on_upgrade = upgrade::on(request);

            Ok(WebSocketUpgrade { on_upgrade, key })
        }
        WebSocketRequest {
            upgrade_result: try_upgrade(&mut request),
        }
    }
}

#[async_trait]
impl ResponseConverter<WebSocketResponse> for WebSocketConverter {
    async fn convert_response(&self, response: WebSocketResponse) -> Response {
        let error_logger = self.error_logger.clone();
        match response.upgrade_result {
            Ok(upgrade) => {
                let config = self.config;

                let future = upgrade
                    .on_upgrade
                    .and_then(move |upgraded| {
                        WebSocketStream::from_raw_socket(upgraded, Role::Server, config).map(Ok)
                    })
                    .and_then(move |stream| (response.upgraded_handler)(stream).map(Ok))
                    .map(move |result| {
                        if let Err(hyper_error) = result {
                            error_logger(WebSocketConverterError::HyperError(hyper_error));
                        }
                    });
                task::spawn(future);

                hyper::Response::builder()
                    .status(StatusCode::SWITCHING_PROTOCOLS)
                    .header("Connection", "Upgrade")
                    .header("Upgrade", "websocket")
                    .header("Sec-WebSocket-Accept", upgrade.key)
                    .body(Body::empty())
                    .unwrap()
            }
            Err(protocol_error) => {
                error_logger(WebSocketConverterError::ProtocolError(protocol_error));

                hyper::Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::empty())
                    .unwrap()
            }
        }
    }
}

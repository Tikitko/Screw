use crate::{
    WebSocketContent, WebSocketOriginContent, WebSocketRequest, WebSocketResponse,
    WebSocketStreamConverter, WebSocketUpgradable, WebSocketUpgrade,
};
use async_trait::async_trait;
use futures_util::{FutureExt, TryFutureExt};
use hyper::header::HeaderValue;
use hyper::{upgrade, Body, Method, StatusCode, Version};
use screw_core::routing::{Request, RequestResponseConverter, Response};
use std::sync::Arc;
use tokio::task;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::protocol::{Role, WebSocketConfig};
use tokio_tungstenite::WebSocketStream;

fn is_get_method(request: &hyper::Request<Body>) -> bool {
    request.method() == Method::GET
}
fn is_http_version_11_or_larger(request: &hyper::Request<Body>) -> bool {
    request.version() >= Version::HTTP_11
}
fn is_connection_header_upgrade(request: &hyper::Request<Body>) -> bool {
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
fn is_upgrade_header_web_socket(request: &hyper::Request<Body>) -> bool {
    request
        .headers()
        .get("Upgrade")
        .and_then(|h| h.to_str().ok())
        .map(|h| h.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}
fn is_web_socket_version_header_13(request: &hyper::Request<Body>) -> bool {
    request
        .headers()
        .get("Sec-WebSocket-Version")
        .map(|h| h == "13")
        .unwrap_or(false)
}
fn get_web_socket_key_header(request: &hyper::Request<Body>) -> Option<&HeaderValue> {
    request.headers().get("Sec-WebSocket-Key")
}

pub fn is_upgrade_request(request: &hyper::Request<hyper::Body>) -> bool {
    is_connection_header_upgrade(request) && is_upgrade_header_web_socket(request)
}

fn try_upgradable(
    http_request: &mut hyper::Request<Body>,
) -> Result<WebSocketUpgradable, ProtocolError> {
    if !is_get_method(http_request) {
        return Err(ProtocolError::WrongHttpMethod);
    }

    if !is_http_version_11_or_larger(http_request) {
        return Err(ProtocolError::WrongHttpVersion);
    }

    if !is_connection_header_upgrade(http_request) {
        return Err(ProtocolError::MissingConnectionUpgradeHeader);
    }

    if !is_upgrade_header_web_socket(http_request) {
        return Err(ProtocolError::MissingUpgradeWebSocketHeader);
    }

    if !is_web_socket_version_header_13(http_request) {
        return Err(ProtocolError::MissingSecWebSocketVersionHeader);
    }

    let key = derive_accept_key(
        get_web_socket_key_header(http_request)
            .ok_or(ProtocolError::MissingSecWebSocketKey)?
            .as_bytes(),
    );

    let on_upgrade = upgrade::on(http_request);

    Ok(WebSocketUpgradable { on_upgrade, key })
}

pub struct WebSocketConverter {
    config: Option<WebSocketConfig>,
}

impl WebSocketConverter {
    pub fn with_config(config: Option<WebSocketConfig>) -> Self {
        Self { config }
    }

    pub fn and_stream_converter<C>(self, stream_converter: C) -> WebSocketConverterFinal<C>
    where
        C: Sync + Send + 'static,
    {
        WebSocketConverterFinal {
            config: self.config,
            stream_converter: Arc::new(stream_converter),
        }
    }
}

pub struct WebSocketConverterFinal<C>
where
    C: Sync + Send + 'static,
{
    config: Option<WebSocketConfig>,
    stream_converter: Arc<C>,
}

#[async_trait]
impl<C, Content, Stream>
    RequestResponseConverter<WebSocketRequest<Content, Stream>, WebSocketResponse>
    for WebSocketConverterFinal<C>
where
    C: WebSocketStreamConverter<Stream> + Sync + Send + 'static,
    Content: WebSocketContent + Send + 'static,
    Stream: Send + Sync + 'static,
{
    type Request = Request;
    type Response = Response;
    async fn convert_request(
        &self,
        mut request: Self::Request,
    ) -> WebSocketRequest<Content, Stream> {
        let upgradable_result = try_upgradable(&mut request.http);
        let (http_parts, _) = request.http.into_parts();

        let request_content = Content::create(WebSocketOriginContent {
            http_parts,
            remote_addr: request.remote_addr,
            extensions: request.extensions,
        });

        let stream_converter = self.stream_converter.clone();
        let request_upgrade = WebSocketUpgrade {
            upgradable_result,
            convert_stream_fn: Box::new(move |generic_stream| {
                let stream_converter = stream_converter.clone();
                Box::pin(async move {
                    let stream = stream_converter.convert_stream(generic_stream).await;
                    stream
                })
            }),
        };

        WebSocketRequest {
            content: request_content,
            upgrade: request_upgrade,
        }
    }
    async fn convert_response(&self, response: WebSocketResponse) -> Self::Response {
        let http_response = match response.upgradable_result {
            Ok(upgradable) => {
                let config = self.config;

                let future = upgradable
                    .on_upgrade
                    .and_then(move |upgraded| {
                        WebSocketStream::from_raw_socket(upgraded, Role::Server, config).map(Ok)
                    })
                    .and_then(move |stream| (response.upgraded_fn)(stream).map(Ok));

                task::spawn(future);

                hyper::Response::builder()
                    .status(StatusCode::SWITCHING_PROTOCOLS)
                    .header("Connection", "Upgrade")
                    .header("Upgrade", "websocket")
                    .header("Sec-WebSocket-Accept", upgradable.key)
                    .body(Body::empty())
                    .unwrap()
            }
            Err(protocol_error) => match protocol_error {
                ProtocolError::WrongHttpMethod => {
                    panic!("incorrect method for WebSocket, should be GET")
                }
                _ => hyper::Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::empty())
                    .unwrap(),
            },
        };
        Response {
            http: http_response,
        }
    }
}

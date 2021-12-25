use crate::request::{WebSocketRequest, WebSocketUpgrade};
use crate::{
    WebSocketContent, WebSocketOriginContent, WebSocketResponse, WebSocketStreamConverter,
    WebSocketUpgradeInfo,
};
use async_trait::async_trait;
use futures_util::{FutureExt, TryFutureExt};
use hyper::header::HeaderValue;
use hyper::{upgrade, Body, Method, StatusCode, Version};
use screw_core::routing::router::RequestResponseConverter;
use screw_core::routing::{Request, Response};
use std::sync::Arc;
use tokio::task;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::protocol::{Role, WebSocketConfig};
use tokio_tungstenite::WebSocketStream;

pub(super) fn is_get_method(request: &hyper::Request<Body>) -> bool {
    request.method() == Method::GET
}
pub(super) fn is_http_version_11_or_larger(request: &hyper::Request<Body>) -> bool {
    request.version() >= Version::HTTP_11
}
pub(super) fn is_connection_header_upgrade(request: &hyper::Request<Body>) -> bool {
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
pub(super) fn is_upgrade_header_web_socket(request: &hyper::Request<Body>) -> bool {
    request
        .headers()
        .get("Upgrade")
        .and_then(|h| h.to_str().ok())
        .map(|h| h.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}
pub(super) fn is_web_socket_version_header_13(request: &hyper::Request<Body>) -> bool {
    request
        .headers()
        .get("Sec-WebSocket-Version")
        .map(|h| h == "13")
        .unwrap_or(false)
}
pub(super) fn get_web_socket_key_header(request: &hyper::Request<Body>) -> Option<&HeaderValue> {
    request.headers().get("Sec-WebSocket-Key")
}

pub struct WebSocketConverter<C>
where
    C: Sync + Send + 'static,
{
    pub stream_converter: Arc<C>,
    pub config: Option<WebSocketConfig>,
}

#[async_trait]
impl<C, Content, Stream>
    RequestResponseConverter<WebSocketRequest<Content, Stream>, WebSocketResponse>
    for WebSocketConverter<C>
where
    C: WebSocketStreamConverter<Stream> + Sync + Send + 'static,
    Content: WebSocketContent + Send + 'static,
    Stream: Send + Sync + 'static,
{
    async fn convert_request(&self, mut request: Request) -> WebSocketRequest<Content, Stream> {
        fn try_upgrade_info(
            request: &mut hyper::Request<Body>,
        ) -> Result<WebSocketUpgradeInfo, ProtocolError> {
            if !is_get_method(request) {
                panic!("WebSocket route should be with GET method!");
                //return Err(ProtocolError::WrongHttpMethod);
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

            Ok(WebSocketUpgradeInfo { on_upgrade, key })
        }

        let stream_converter = self.stream_converter.clone();
        let upgrade_info_result = try_upgrade_info(&mut request.http);
        let (http_parts, _) = request.http.into_parts();

        let request_content = Content::create(WebSocketOriginContent {
            http_parts,
            remote_addr: request.remote_addr,
            extensions: request.extensions,
        });

        WebSocketRequest {
            content: request_content,
            upgrade: WebSocketUpgrade {
                upgrade_info_result,
                stream_converter: Box::new(move |generic_stream| {
                    let stream_converter = stream_converter.clone();
                    Box::pin(async move {
                        let stream = stream_converter.convert_stream(generic_stream).await;
                        stream
                    })
                }),
            },
        }
    }
    async fn convert_response(&self, response: WebSocketResponse) -> Response {
        let response = match response.upgrade_info_result {
            Ok(upgrade_info) => {
                let config = self.config;

                let future = upgrade_info
                    .on_upgrade
                    .and_then(move |upgraded| {
                        WebSocketStream::from_raw_socket(upgraded, Role::Server, config).map(Ok)
                    })
                    .and_then(move |stream| (response.upgraded_handler)(stream).map(Ok))
                    .map(move |result| if let Err(_) = result {});

                task::spawn(future);

                hyper::Response::builder()
                    .status(StatusCode::SWITCHING_PROTOCOLS)
                    .header("Connection", "Upgrade")
                    .header("Upgrade", "websocket")
                    .header("Sec-WebSocket-Accept", upgrade_info.key)
                    .body(Body::empty())
                    .unwrap()
            }
            Err(_) => hyper::Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap(),
        };
        Response { http: response }
    }
}

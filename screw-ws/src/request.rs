use crate::WebSocketResponse;
use hyper::http::request::Parts;
use hyper::http::Extensions;
use hyper::upgrade::{OnUpgrade, Upgraded};
use screw_core::DFn;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::WebSocketStream;

pub struct WebSocketOriginContent {
    pub http_parts: Parts,
    pub remote_addr: SocketAddr,
    pub extensions: Arc<Extensions>,
}

pub trait WebSocketContent {
    fn create(origin_content: WebSocketOriginContent) -> Self;
}

pub(super) struct WebSocketUpgradeInfo { // TODO: Name?
    pub(super) on_upgrade: OnUpgrade,
    pub(super) key: String,
}

pub struct WebSocketUpgrade<Stream>
where
    Stream: Send + Sync + 'static,
{
    pub(super) upgrade_info_result: Result<WebSocketUpgradeInfo, ProtocolError>,
    pub(super) stream_converter: DFn<WebSocketStream<Upgraded>, Stream>,
}

impl<Stream> WebSocketUpgrade<Stream>
where
    Stream: Send + Sync + 'static,
{
    pub fn on<F, U>(self, upgraded_handler: F) -> WebSocketResponse
    where
        F: FnOnce(Stream) -> U + Send + Sync + 'static,
        U: Future<Output = ()> + Send + 'static,
    {
        let stream_converter = Arc::new(self.stream_converter);
        WebSocketResponse {
            upgrade_info_result: self.upgrade_info_result,
            upgraded_handler: Box::new(move |generic_stream| {
                let stream_converter = stream_converter.clone();
                Box::pin(async move {
                    let stream = stream_converter(generic_stream).await;
                    upgraded_handler(stream).await;
                })
            }),
        }
    }
}

pub struct WebSocketRequest<Content, Stream>
where
    Content: WebSocketContent + Send + 'static,
    Stream: Send + Sync + 'static,
{
    pub content: Content,
    pub upgrade: WebSocketUpgrade<Stream>,
}

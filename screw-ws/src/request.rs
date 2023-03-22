use super::*;
use hyper::http::request::Parts;
use hyper::http::Extensions;
use hyper::upgrade::{OnUpgrade, Upgraded};
use screw_components::dyn_fn::DFn;
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

impl WebSocketContent for () {
    fn create(_origin_content: WebSocketOriginContent) -> Self {}
}

pub(super) struct WebSocketUpgradable {
    pub(super) on_upgrade: OnUpgrade,
    pub(super) key: String,
}

pub struct WebSocketUpgrade<Stream>
where
    Stream: Send + Sync + 'static,
{
    pub(super) upgradable_result: Result<WebSocketUpgradable, ProtocolError>,
    pub(super) convert_stream_fn: DFn<WebSocketStream<Upgraded>, Stream>,
}

impl<Stream> WebSocketUpgrade<Stream>
where
    Stream: Send + Sync + 'static,
{
    pub fn on<F, U>(self, upgraded_fn: F) -> WebSocketResponse
    where
        F: FnOnce(Stream) -> U + Send + Sync + 'static,
        U: Future<Output = ()> + Send + 'static,
    {
        let convert_stream_fn = Arc::new(self.convert_stream_fn);
        WebSocketResponse {
            upgradable_result: self.upgradable_result,
            upgraded_fn: Box::new(move |generic_stream| {
                let convert_stream_fn = convert_stream_fn.clone();
                Box::pin(async move {
                    let stream = convert_stream_fn(generic_stream).await;
                    upgraded_fn(stream).await;
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
    pub(super) content: Content,
    pub(super) upgrade: WebSocketUpgrade<Stream>,
}

impl<Content, Stream> WebSocketRequest<Content, Stream>
where
    Content: WebSocketContent + Send + 'static,
    Stream: Send + Sync + 'static,
{
    pub fn split(self) -> (Content, WebSocketUpgrade<Stream>) {
        (self.content, self.upgrade)
    }
}

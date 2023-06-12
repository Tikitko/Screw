use super::*;
use hyper::http::request::Parts;
use hyper::upgrade::{OnUpgrade, Upgraded};
use screw_components::dyn_fn::DFn;
use screw_core::routing::Path;
use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::WebSocketStream;

pub struct WebSocketOriginContent<Extensions> {
    pub path: Path<String>,
    pub query: HashMap<String, String>,
    pub http_parts: Parts,
    pub remote_addr: SocketAddr,
    pub extensions: Arc<Extensions>,
}

pub trait WebSocketContent<Extensions> {
    fn create(origin_content: WebSocketOriginContent<Extensions>) -> Self;
}

impl<Extensions> WebSocketContent<Extensions> for () {
    fn create(_origin_content: WebSocketOriginContent<Extensions>) -> Self {}
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

pub struct WebSocketRequest<Content, Stream, Extensions>
where
    Content: WebSocketContent<Extensions> + Send + 'static,
    Stream: Send + Sync + 'static,
{
    pub(super) content: Content,
    pub(super) upgrade: WebSocketUpgrade<Stream>,
    pub(super) _p_e: PhantomData<Extensions>,
}

impl<Content, Stream, Extensions> WebSocketRequest<Content, Stream, Extensions>
where
    Content: WebSocketContent<Extensions> + Send + 'static,
    Stream: Send + Sync + 'static,
{
    pub fn split(self) -> (Content, WebSocketUpgrade<Stream>) {
        (self.content, self.upgrade)
    }
}

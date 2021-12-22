use hyper::upgrade::{OnUpgrade, Upgraded};
use screw_core::DFnOnce;
use std::future::Future;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::WebSocketStream;

pub struct WebSocketUpgrade {
    pub(super) on_upgrade: OnUpgrade,
    pub(super) key: String,
}

pub struct WebSocketRequest {
    pub(super) upgrade_result: Result<WebSocketUpgrade, ProtocolError>,
}

impl WebSocketRequest {
    pub fn on_upgrade<F, U>(self, upgraded_handler: F) -> WebSocketResponse
    where
        F: FnOnce(WebSocketStream<Upgraded>) -> U + Send + Sync + 'static,
        U: Future<Output = ()> + Send + 'static,
    {
        WebSocketResponse {
            upgrade_result: self.upgrade_result,
            upgraded_handler: Box::new(move |stream| {
                Box::pin(async move {
                    upgraded_handler(stream);
                })
            }),
        }
    }
}

pub struct WebSocketResponse {
    pub(super) upgrade_result: Result<WebSocketUpgrade, ProtocolError>,
    pub(super) upgraded_handler: DFnOnce<WebSocketStream<Upgraded>, ()>,
}

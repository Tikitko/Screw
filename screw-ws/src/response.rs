use super::*;
use hyper::upgrade::Upgraded;
use screw_components::dyn_fn::DFnOnce;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::WebSocketStream;

pub struct WebSocketResponse {
    pub(super) upgradable_result: Result<WebSocketUpgradable, ProtocolError>,
    pub(super) upgraded_fn: DFnOnce<WebSocketStream<Upgraded>, ()>,
}

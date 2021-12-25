use crate::WebSocketUpgradeInfo;
use hyper::upgrade::Upgraded;
use screw_core::DFnOnce;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::WebSocketStream;

pub struct WebSocketResponse {
    pub(super) upgrade_info_result: Result<WebSocketUpgradeInfo, ProtocolError>,
    pub(super) upgraded_handler: DFnOnce<WebSocketStream<Upgraded>, ()>,
}

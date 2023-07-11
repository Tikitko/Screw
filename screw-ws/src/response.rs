use hyper::upgrade::Upgraded;
use screw_components::dyn_fn::DFnOnce;
use tokio_tungstenite::WebSocketStream;

pub struct WebSocketResponse {
    pub(super) upgraded_fn: DFnOnce<WebSocketStream<Upgraded>, ()>,
}

mod builder;
mod convert_handler;
mod converter;
mod route;

pub use builder::*;
pub(in crate::routing::router) use convert_handler::*;
pub use converter::*;
pub use route::*;

pub struct StreamableRequest {
    pub stream: tokio_tungstenite::WebSocketStream<hyper::upgrade::Upgraded>,
    pub extensions: hyper::http::Extensions,
}

pub type WebSocketHandler = crate::routing::Handler<StreamableRequest, ()>;

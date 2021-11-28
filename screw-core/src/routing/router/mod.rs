mod builder;
mod default_types;
pub mod http;
mod router;
mod scoped_builder;
pub mod web_socket;

pub use builder::*;
pub use default_types::*;
pub use router::*;
pub use scoped_builder::*;

struct HandlersContainer {
    web_socket: std::collections::HashMap<String, std::sync::Arc<web_socket::WebSocketHandler>>,
    http: std::collections::HashMap<(hyper::Method, String), http::HttpHandler>,
}

use hyper::http::Extensions;
use hyper::Body;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct Request {
    pub remote_addr: SocketAddr,
    pub extensions: Arc<Extensions>,
    pub http: hyper::Request<Body>,
}

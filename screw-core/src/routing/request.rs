use hyper::http::Extensions;
use hyper::Body;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct Request {
    pub remote_addr: SocketAddr,
    pub extensions: Arc<Extensions>,
    pub http: hyper::Request<Body>,
}

impl AsRef<hyper::Request<Body>> for Request {
    fn as_ref(&self) -> &hyper::Request<Body> {
        &self.http
    }
}

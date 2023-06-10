use hyper::Body;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct Request<Extensions> {
    pub remote_addr: SocketAddr,
    pub extensions: Arc<Extensions>,
    pub http: hyper::Request<Body>,
}

impl<Extensions> AsRef<hyper::Request<Body>> for Request<Extensions> {
    fn as_ref(&self) -> &hyper::Request<Body> {
        &self.http
    }
}

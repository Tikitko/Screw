use super::{Responder, SessionService};
use hyper::server::conn::AddrStream;
use hyper::service::Service;
use std::convert::Infallible;
use std::future::{ready, Ready};
use std::net::SocketAddr;
use std::task::{Context, Poll};

pub struct ServerService<F, R>
where
    F: Fn(SocketAddr) -> R,
    R: Responder,
    R::ResponseFuture: 'static,
{
    make_responder_fn: F,
}

impl<F, R> ServerService<F, R>
where
    F: Fn(SocketAddr) -> R,
    R: Responder,
    R::ResponseFuture: 'static,
{
    pub fn with_make_responder_fn(make_responder_fn: F) -> Self {
        Self { make_responder_fn }
    }
}

impl<F, R> Service<&AddrStream> for ServerService<F, R>
where
    F: Fn(SocketAddr) -> R,
    R: Responder,
    R::ResponseFuture: 'static,
{
    type Response = SessionService<R>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, addr_stream: &AddrStream) -> Self::Future {
        let remote_addr = addr_stream.remote_addr();
        let responder = (self.make_responder_fn)(remote_addr);
        let session_service = SessionService { responder };

        ready(Ok(session_service))
    }
}

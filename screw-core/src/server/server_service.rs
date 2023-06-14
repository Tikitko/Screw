use super::*;
use hyper::server::conn::AddrStream;
use hyper::service::Service;
use std::convert::Infallible;
use std::future::{ready, Ready};
use std::task::{Context, Poll};

pub struct ServerService<F, R>
where
    F: ResponderFactory<Responder = R>,
    R: Responder,
    R::ResponseFuture: Send + 'static,
{
    responder_factory: F,
}

impl<F, R> ServerService<F, R>
where
    F: ResponderFactory<Responder = R>,
    R: Responder,
    R::ResponseFuture: Send + 'static,
{
    pub fn with_responder_factory(responder_factory: F) -> Self {
        Self { responder_factory }
    }
}

impl<F, R> Service<&AddrStream> for ServerService<F, R>
where
    F: ResponderFactory<Responder = R>,
    R: Responder,
    R::ResponseFuture: Send + 'static,
{
    type Response = SessionService<R>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, addr_stream: &AddrStream) -> Self::Future {
        let remote_addr = addr_stream.remote_addr();
        let responder = self.responder_factory.make_responder(remote_addr);
        let session_service = SessionService { responder };
        ready(Ok(session_service))
    }
}

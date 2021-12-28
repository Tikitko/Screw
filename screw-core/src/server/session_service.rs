use super::Responder;
use hyper::service::Service;
use hyper::{Body, Request, Response};
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct SessionService<R>
where
    R: Responder,
    R::ResponseFuture: 'static,
{
    pub(super) responder: R,
}

impl<R> Service<Request<Body>> for SessionService<R>
where
    R: Responder,
    R::ResponseFuture: 'static,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let response_future = self.responder.response(request);

        Box::pin(async move {
            let response = response_future.await;
            Ok(response)
        })
    }
}

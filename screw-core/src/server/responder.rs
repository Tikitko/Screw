use hyper::{Body, Request, Response};
use std::future::Future;

pub trait Responder: Send {
    type ResponseFuture: Future<Output = Response<Body>> + Send;
    fn response(&mut self, request: Request<Body>) -> Self::ResponseFuture;
}

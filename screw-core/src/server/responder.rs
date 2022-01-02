use hyper::{Body, Request, Response};
use std::future::Future;

pub trait Responder {
    type ResponseFuture: Future<Output = Response<Body>>;
    fn response(&mut self, request: Request<Body>) -> Self::ResponseFuture;
}

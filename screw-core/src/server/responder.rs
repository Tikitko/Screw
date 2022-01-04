use hyper::{Body, Request, Response};
use std::future::Future;

pub trait Responder {
    type ResponseFuture: Future<Output = Response<Body>>;
    fn response(&mut self, request: Request<Body>) -> Self::ResponseFuture;
}

impl<'a, T> Responder for &'a mut T
where
    T: Responder + 'a,
{
    type ResponseFuture = T::ResponseFuture;
    fn response(&mut self, request: Request<Body>) -> Self::ResponseFuture {
        (**self).response(request)
    }
}

impl<T> Responder for Box<T>
where
    T: Responder + ?Sized,
{
    type ResponseFuture = T::ResponseFuture;
    fn response(&mut self, request: Request<Body>) -> Self::ResponseFuture {
        (**self).response(request)
    }
}

use super::*;
use std::net::SocketAddr;

pub trait ResponderFactory {
    type Responder: Responder;
    fn make_responder(&self, remote_addr: SocketAddr) -> Self::Responder;
}

impl<'a, T> ResponderFactory for &'a T
where
    T: ResponderFactory + 'a,
{
    type Responder = T::Responder;
    fn make_responder(&self, remote_addr: SocketAddr) -> Self::Responder {
        (**self).make_responder(remote_addr)
    }
}

impl<T> ResponderFactory for Box<T>
where
    T: ResponderFactory + ?Sized,
{
    type Responder = T::Responder;
    fn make_responder(&self, remote_addr: SocketAddr) -> Self::Responder {
        (**self).make_responder(remote_addr)
    }
}

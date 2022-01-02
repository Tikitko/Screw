use super::Responder;
use std::net::SocketAddr;

pub trait ResponderFactory {
    type Responder: Responder;
    fn make_responder(&self, remote_addr: SocketAddr) -> Self::Responder;
}

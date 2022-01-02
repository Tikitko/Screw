use super::Responder;
use std::net::SocketAddr;

pub trait RespondersFactory {
    type Responder: Responder;
    fn make_responder(&self, remote_addr: SocketAddr) -> Self::Responder;
}

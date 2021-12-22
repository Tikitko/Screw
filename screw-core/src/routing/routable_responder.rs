use super::router::Router;
use crate::maps::SharedDataMap;
use crate::routing::Request;
use crate::server::Responder;
use hyper::Body;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

pub struct RoutableResponder {
    pub(crate) remote_addr: SocketAddr,
    pub(crate) data_map: SharedDataMap,
    pub(crate) router: Arc<Router>,
}

impl Responder for RoutableResponder {
    type ResponseFuture = Pin<Box<dyn Future<Output = hyper::Response<Body>> + Send>>;

    fn response(&mut self, request: hyper::Request<Body>) -> Self::ResponseFuture {
        let router = self.router.clone();

        let remote_addr = self.remote_addr;
        let data_map = self.data_map.clone();

        Box::pin(async move {
            let request = Request {
                remote_addr,
                data_map,
                http: request,
            };
            let response = router.process(request).await;
            response.http
        })
    }
}

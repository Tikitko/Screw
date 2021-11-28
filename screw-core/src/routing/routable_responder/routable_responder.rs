use crate::maps::DataMap;
use crate::routing::router::Router;
use crate::server::Responder;
use hyper::{Body, Request, Response};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

pub struct RoutableResponder {
    pub remote_addr: SocketAddr,
    pub data_map: Arc<DataMap>,
    pub router: Arc<Router>,
}

impl Responder for RoutableResponder {
    type ResponseFuture = Pin<Box<dyn Future<Output = Response<Body>> + Send>>;

    fn response(&mut self, mut request: Request<Body>) -> Self::ResponseFuture {
        let router = self.router.clone();

        let ext = request.extensions_mut();
        ext.insert(self.remote_addr);
        ext.insert(self.data_map.clone());

        let future = async move {
            let response = router.process(request).await;
            response
        };

        Box::pin(future)
    }
}

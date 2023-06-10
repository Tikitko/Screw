pub type ResponderFactory<Extensions> = first::ResponderFactory<Extensions>;
pub type FResponderFactory<Extensions> = second::ResponderFactory<Extensions>;

use super::*;
use hyper::Body;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

pub mod first {
    use super::*;
    use std::sync::Arc;

    pub struct ResponderFactory<Extensions>
    where
        Extensions: Sync + Send + 'static
    {
        router: Arc<routing::router::second::Router<Request<Extensions>, Response>>,
    }

    impl<Extensions> ResponderFactory<Extensions>
    where
        Extensions: Sync + Send + 'static
    {
        pub fn with_router(router: routing::router::second::Router<Request<Extensions>, Response>) -> Self {
            Self {
                router: Arc::new(router),
            }
        }

        pub fn and_extensions(self, extensions: Extensions) -> second::ResponderFactory<Extensions> {
            second::ResponderFactory {
                router: self.router,
                extensions: Arc::new(extensions),
            }
        }
    }
}

pub mod second {
    use super::*;
    use std::net::SocketAddr;
    use std::sync::Arc;

    pub struct ResponderFactory<Extensions>
    where
        Extensions: Sync + Send + 'static
    {
        pub(super) router: Arc<routing::router::second::Router<Request<Extensions>, Response>>,
        pub(super) extensions: Arc<Extensions>,
    }

    impl<Extensions> server::ResponderFactory for ResponderFactory<Extensions>
    where
        Extensions: Sync + Send + 'static
    {
        type Responder = Responder<Extensions>;
        fn make_responder(&self, remote_addr: SocketAddr) -> Self::Responder {
            Responder {
                remote_addr,
                router: self.router.clone(),
                extensions: self.extensions.clone(),
            }
        }
    }
}

pub struct Responder<Extensions>
where
    Extensions: Sync + Send + 'static
{
    remote_addr: SocketAddr,
    router: Arc<routing::router::second::Router<Request<Extensions>, Response>>,
    extensions: Arc<Extensions>,
}

impl<Extensions> server::Responder for Responder<Extensions>
where
    Extensions: Sync + Send + 'static
{
    type ResponseFuture = Pin<Box<dyn Future<Output = hyper::Response<Body>> + Send>>;

    fn response(&mut self, http_request: hyper::Request<Body>) -> Self::ResponseFuture {
        let remote_addr = self.remote_addr;
        let router = self.router.clone();
        let extensions = self.extensions.clone();
        Box::pin(async move {
            let request = Request {
                remote_addr,
                extensions,
                http: http_request,
            };
            let response = router.process(request).await;
            let http_response = response.http;
            http_response
        })
    }
}

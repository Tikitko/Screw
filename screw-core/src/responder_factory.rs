pub type ResponderFactory = first::ResponderFactory;
pub type FResponderFactory = second::ResponderFactory;

use super::*;
use hyper::http::Extensions;
use hyper::Body;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

pub mod first {
    use super::*;
    use hyper::http::Extensions;
    use std::sync::Arc;

    pub struct ResponderFactory {
        router: Arc<routing::router::second::Router<Request, Response>>,
    }

    impl ResponderFactory {
        pub fn with_router(router: routing::router::second::Router<Request, Response>) -> Self {
            Self {
                router: Arc::new(router),
            }
        }

        pub fn and_extensions(self, extensions: Extensions) -> second::ResponderFactory {
            second::ResponderFactory {
                router: self.router,
                extensions: Arc::new(extensions),
            }
        }
    }
}

pub mod second {
    use super::*;
    use hyper::http::Extensions;
    use std::net::SocketAddr;
    use std::sync::Arc;

    pub struct ResponderFactory {
        pub(super) router: Arc<routing::router::second::Router<Request, Response>>,
        pub(super) extensions: Arc<Extensions>,
    }

    impl server::ResponderFactory for ResponderFactory {
        type Responder = Responder;
        fn make_responder(&self, remote_addr: SocketAddr) -> Self::Responder {
            Responder {
                remote_addr,
                router: self.router.clone(),
                extensions: self.extensions.clone(),
            }
        }
    }
}

pub struct Responder {
    remote_addr: SocketAddr,
    router: Arc<routing::router::second::Router<Request, Response>>,
    extensions: Arc<Extensions>,
}

impl server::Responder for Responder {
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

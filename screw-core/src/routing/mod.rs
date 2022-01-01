mod convert_handler;
mod default_types;
mod request;
mod request_response_converter;
mod response;
mod routable_responder;
mod route;
mod router;
mod router_builder;
mod routes_collection;
mod routes_collection_builder;

use convert_handler::*;
pub use default_types::*;
pub use request::*;
pub use request_response_converter::*;
pub use response::*;
pub use routable_responder::*;
pub use route::*;
pub use router::*;
pub use router_builder::*;
pub use routes_collection::*;
pub use routes_collection_builder::*;

pub fn routable_server_service(
    extensions: hyper::http::Extensions,
    router: crate::routing::Router<crate::routing::Request, crate::routing::Response>,
) -> impl for<'a> hyper::service::Service<
    &'a hyper::server::conn::AddrStream,
    Error = std::convert::Infallible,
    Response = crate::server::SessionService<crate::routing::RoutableResponder>,
    Future = core::future::Ready<
        Result<
            crate::server::SessionService<crate::routing::RoutableResponder>,
            std::convert::Infallible,
        >,
    >,
> {
    let extensions = std::sync::Arc::new(extensions);
    let router = std::sync::Arc::new(router);
    crate::server::ServerService::with_make_responder_fn(move |remote_addr| {
        crate::routing::RoutableResponder {
            remote_addr,
            extensions: extensions.clone(),
            router: router.clone(),
        }
    })
}

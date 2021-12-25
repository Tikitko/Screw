pub mod routable_responder;
pub mod router;

pub struct Request {
    pub remote_addr: std::net::SocketAddr,
    pub extensions: std::sync::Arc<hyper::http::Extensions>,
    pub http: hyper::Request<hyper::Body>,
}
pub struct Response {
    pub http: hyper::Response<hyper::Body>,
}

pub type Handler = crate::DFn<Request, Response>;

pub fn query_params(uri: &hyper::Uri) -> std::collections::HashMap<String, String> {
    uri.query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(std::collections::HashMap::new)
}

pub fn routable_server_service(
    extensions: hyper::http::Extensions,
    router: crate::routing::router::Router,
) -> impl for<'a> hyper::service::Service<
    &'a hyper::server::conn::AddrStream,
    Error = std::convert::Infallible,
    Response = crate::server::SessionService<crate::routing::routable_responder::RoutableResponder>,
    Future = core::future::Ready<
        Result<
            crate::server::SessionService<crate::routing::routable_responder::RoutableResponder>,
            std::convert::Infallible,
        >,
    >,
> {
    let extensions = std::sync::Arc::new(extensions);
    let router = std::sync::Arc::new(router);
    crate::server::ServerService::new(move |remote_addr| {
        crate::routing::routable_responder::RoutableResponder {
            remote_addr,
            extensions: extensions.clone(),
            router: router.clone(),
        }
    })
}

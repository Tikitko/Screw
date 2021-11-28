use super::http::{
    convert_http_handler, HttpConverter, HttpHandler, HttpRoute, HttpRouterBuilder,
    HttpRouterBuilderNotFound,
};
use super::web_socket::{
    convert_web_socket_handler, WebSocketConverter, WebSocketRoute, WebSocketRouterBuilder,
};
use super::HandlersContainer;
use super::Router;
use super::ScopedRouterBuilder;
use derive_error::Error;
use std::future::Future;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

#[derive(Error, Debug)]
pub enum RouterBuilderError {
    NotFoundHandlerMissing,
}

pub struct RouterBuilder<Converter>
where
    Converter: Send + Sync + 'static,
{
    converter: Arc<Converter>,
    web_socket_config: Option<WebSocketConfig>,
    not_found_http_handler: Option<HttpHandler>,
    handlers_container: HandlersContainer,
}

impl<Converter> RouterBuilder<Converter>
where
    Converter: Send + Sync + 'static,
{
    pub fn new(converter: Converter) -> Self
    where
        Converter: Send + Sync + 'static,
    {
        Self {
            converter: Arc::new(converter),
            not_found_http_handler: None,
            web_socket_config: None,
            handlers_container: HandlersContainer {
                web_socket: Default::default(),
                http: Default::default(),
            },
        }
    }

    pub fn extend<ScopedConverter>(
        mut self,
        scoped_builder: ScopedRouterBuilder<ScopedConverter>,
    ) -> Self
    where
        ScopedConverter: Send + Sync + 'static,
    {
        let scope_handlers_container = scoped_builder.handlers_container();
        self.handlers_container
            .http
            .extend(scope_handlers_container.http);
        self.handlers_container
            .web_socket
            .extend(scope_handlers_container.web_socket);
        self
    }

    pub fn web_socket_config(mut self, config: WebSocketConfig) -> Self {
        self.web_socket_config = Some(config);
        self
    }

    pub fn build(self) -> Result<Router, RouterBuilderError> {
        Ok(Router {
            handlers_container: self.handlers_container,
            not_found_http_handler: self
                .not_found_http_handler
                .ok_or(RouterBuilderError::NotFoundHandlerMissing)?,
            web_socket_config: self.web_socket_config,
        })
    }
}

impl<Converter, Rq, Rs, HFn, HFut> HttpRouterBuilderNotFound<Converter, Rq, Rs, HFn, HFut>
    for RouterBuilder<Converter>
where
    Converter: HttpConverter<Rq, Rs> + Send + Sync + 'static,
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
{
    fn not_found_http_handler(mut self, not_found_http_handler: HFn) -> Self {
        self.not_found_http_handler = Some(convert_http_handler(
            self.converter.clone(),
            not_found_http_handler,
        ));
        self
    }
}

impl<Converter, Route> HttpRouterBuilder<Converter, Route> for RouterBuilder<Converter>
where
    Converter: HttpConverter<Route::Rq, Route::Rs> + Send + Sync + 'static,
    Route: HttpRoute + 'static,
{
    fn http_route(mut self, _route: Route) -> Self {
        let method = Route::method().clone();
        let path = Route::path().to_string();
        let handler = convert_http_handler(self.converter.clone(), Route::handler);

        self.handlers_container.http.insert((method, path), handler);
        self
    }
}

impl<Converter, Route> WebSocketRouterBuilder<Converter, Route> for RouterBuilder<Converter>
where
    Converter: WebSocketConverter<Route::SRq> + Send + Sync + 'static,
    Route: WebSocketRoute + 'static,
{
    fn web_socket_route(mut self, _route: Route) -> Self {
        let path = Route::path().to_string();
        let handler = Arc::new(convert_web_socket_handler(
            self.converter.clone(),
            Route::handler,
        ));

        self.handlers_container.web_socket.insert(path, handler);
        self
    }
}

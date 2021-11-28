use crate::routing::router::http::convert_http_handler;
use crate::routing::router::http::HttpConverter;
use crate::routing::router::http::HttpRoute;
use crate::routing::router::http::HttpRouterBuilder;
use crate::routing::router::web_socket::convert_web_socket_handler;
use crate::routing::router::web_socket::WebSocketConverter;
use crate::routing::router::web_socket::WebSocketRoute;
use crate::routing::router::web_socket::WebSocketRouterBuilder;
use crate::routing::router::HandlersContainer;
use std::sync::Arc;

pub struct ScopedRouterBuilder<Converter>
where
    Converter: Send + Sync + 'static,
{
    path: &'static str,
    converter: Arc<Converter>,
    handlers_container: HandlersContainer,
}

impl<Converter> ScopedRouterBuilder<Converter>
where
    Converter: Send + Sync + 'static,
{
    pub fn new(path: &'static str, converter: Converter) -> Self
    where
        Converter: Send + Sync + 'static,
    {
        Self {
            path,
            converter: Arc::new(converter),
            handlers_container: HandlersContainer {
                web_socket: Default::default(),
                http: Default::default(),
            },
        }
    }

    pub(super) fn handlers_container(self) -> HandlersContainer {
        self.handlers_container
    }
}

impl<Converter, Route> HttpRouterBuilder<Converter, Route> for ScopedRouterBuilder<Converter>
where
    Converter: HttpConverter<Route::Rq, Route::Rs> + Send + Sync + 'static,
    Route: HttpRoute + 'static,
{
    fn http_route(mut self, _route: Route) -> Self {
        let method = Route::method().clone();
        let path = format!("{}{}", self.path, Route::path());
        let handler = convert_http_handler(self.converter.clone(), Route::handler);

        self.handlers_container.http.insert((method, path), handler);
        self
    }
}

impl<Converter, Route> WebSocketRouterBuilder<Converter, Route> for ScopedRouterBuilder<Converter>
where
    Converter: WebSocketConverter<Route::SRq> + Send + Sync + 'static,
    Route: WebSocketRoute + 'static,
{
    fn web_socket_route(mut self, _route: Route) -> Self {
        let path = format!("{}{}", self.path, Route::path());
        let handler = Arc::new(convert_web_socket_handler(
            self.converter.clone(),
            Route::handler,
        ));

        self.handlers_container.web_socket.insert(path, handler);
        self
    }
}

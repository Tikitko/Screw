use super::{WebSocketConverter, WebSocketRoute};

pub trait WebSocketRouterBuilder<Converter, Route>
where
    Converter: WebSocketConverter<Route::SRq> + Send + Sync + 'static,
    Route: WebSocketRoute + 'static,
{
    fn web_socket_route(self, _route: Route) -> Self;
}

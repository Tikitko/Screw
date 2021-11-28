use super::{HttpConverter, HttpRoute};
use std::future::Future;

pub trait HttpRouterBuilderNotFound<Converter, Rq, Rs, HFn, HFut>
where
    Converter: HttpConverter<Rq, Rs> + Send + Sync + 'static,
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
{
    fn not_found_http_handler(self, not_found_http_handler: HFn) -> Self;
}

pub trait HttpRouterBuilder<Converter, Route>
where
    Converter: HttpConverter<Route::Rq, Route::Rs> + Send + Sync + 'static,
    Route: HttpRoute + 'static,
{
    fn http_route(self, _route: Route) -> Self;
}

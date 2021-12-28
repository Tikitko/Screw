use super::{
    convert_generic_handler, Handler, Request, Response, RouteFinal, Router, RoutesCollection,
};
use hyper::Method;
use std::collections::HashMap;
use std::future::Future;

pub struct RouterBuilder {
    handlers: HashMap<(Method, String), Handler>,
    fallback_handler: Handler,
}

impl RouterBuilder {
    pub fn with_fallback_handler<HFn, HFut>(fallback_handler: HFn) -> Self
    where
        HFn: Fn(Request) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = Response> + Send + 'static,
    {
        RouterBuilder {
            handlers: Default::default(),
            fallback_handler: convert_generic_handler(fallback_handler),
        }
    }

    pub fn route<HFn, HFut>(mut self, route: RouteFinal<Request, Response, HFn, HFut>) -> Self
    where
        HFn: Fn(Request) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = Response> + Send + 'static,
    {
        self.handlers.insert(
            (route.method.clone(), route.path.to_string()),
            convert_generic_handler(route.handler),
        );
        self
    }

    pub fn routes(mut self, routes: RoutesCollection) -> Self {
        self.handlers.extend(routes.handlers);
        self
    }

    pub fn build(self) -> Router {
        Router {
            handlers: self.handlers,
            fallback_handler: self.fallback_handler,
        }
    }
}

use super::{RouteFinal, Router, RoutesCollection};
use hyper::{Body, Method, Request};
use screw_components::dyn_fn::{AsDynFn, DFn};
use std::collections::HashMap;
use std::future::Future;

pub struct RouterBuilder<ORq, ORs>
where
    ORq: AsRef<Request<Body>> + Send + 'static,
    ORs: Send + 'static,
{
    handlers: HashMap<(Method, String), DFn<ORq, ORs>>,
    fallback_handler: DFn<ORq, ORs>,
}

impl<ORq, ORs> RouterBuilder<ORq, ORs>
where
    ORq: AsRef<Request<Body>> + Send + 'static,
    ORs: Send + 'static,
{
    pub fn with_fallback_handler<HFn, HFut>(fallback_handler: HFn) -> Self
    where
        HFn: Fn(ORq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = ORs> + Send + 'static,
    {
        RouterBuilder {
            handlers: Default::default(),
            fallback_handler: fallback_handler.to_dyn_fn(),
        }
    }

    pub fn route<HFn, HFut>(mut self, route: RouteFinal<ORq, ORs, HFn, HFut>) -> Self
    where
        HFn: Fn(ORq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = ORs> + Send + 'static,
    {
        self.handlers.insert(
            (route.method.clone(), route.path.to_string()),
            route.handler.to_dyn_fn(),
        );
        self
    }

    pub fn routes(mut self, routes: RoutesCollection<ORq, ORs>) -> Self {
        self.handlers.extend(routes.handlers);
        self
    }

    pub fn build(self) -> Router<ORq, ORs> {
        Router {
            handlers: self.handlers,
            fallback_handler: self.fallback_handler,
        }
    }
}

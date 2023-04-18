use super::*;
use hyper::Method;
use screw_components::dyn_fn::{AsDynFn, DFn};
use std::collections::HashMap;
use std::future::Future;

pub struct RouterBuilderParams<HFn> {
    pub fallback_handler: HFn,
}

pub struct RouterBuilder<ORq, ORs>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
{
    handlers: HashMap<(Method, String), DFn<ORq, ORs>>,
    fallback_handler: DFn<ORq, ORs>,
}

impl<ORq, ORs> RouterBuilder<ORq, ORs>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
{
    pub fn new<HFn, HFut>(params: RouterBuilderParams<HFn>) -> Self
    where
        HFn: Fn(ORq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = ORs> + Send + 'static,
    {
        RouterBuilder {
            handlers: Default::default(),
            fallback_handler: params.fallback_handler.to_dyn_fn(),
        }
    }

    pub fn route<HFn, HFut>(mut self, route: Route<ORq, ORs, HFn, HFut>) -> Self
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

    pub fn routes_collection(mut self, routes_collection: RoutesCollection<ORq, ORs>) -> Self {
        self.handlers.extend(routes_collection.handlers);
        self
    }

    pub fn build(self) -> Router<ORq, ORs> {
        Router {
            handlers: self.handlers,
            fallback_handler: self.fallback_handler,
        }
    }
}

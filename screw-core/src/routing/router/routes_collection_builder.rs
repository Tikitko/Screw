use super::{convert_typed_handler, HandlerRoute, RequestResponseConverter, RoutesCollection};
use crate::routing::Handler;
use hyper::Method;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

pub struct RoutesCollectionBuilderParams<C>
where
    C: Send + Sync + 'static,
{
    pub scope_path: &'static str,
    pub converter: C,
}

pub struct RoutesCollectionBuilder<C>
where
    C: Send + Sync + 'static,
{
    scope_path: &'static str,
    converter: Arc<C>,
    handlers: HashMap<(Method, String), Handler>,
}

impl<C> RoutesCollectionBuilder<C>
where
    C: Send + Sync + 'static,
{
    pub fn new(params: RoutesCollectionBuilderParams<C>) -> Self {
        Self {
            scope_path: params.scope_path,
            converter: Arc::new(params.converter),
            handlers: Default::default(),
        }
    }

    pub fn build(self) -> RoutesCollection {
        RoutesCollection {
            handlers: self.handlers,
        }
    }
}

pub trait RoutesCollectionBuild<Rq, Rs, HFn, HFut, C>
where
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
    C: RequestResponseConverter<Rq, Rs> + Send + Sync + 'static,
{
    fn route(self, route: HandlerRoute<Rq, Rs, HFn, HFut>) -> Self;
}

impl<Rq, Rs, HFn, HFut, C> RoutesCollectionBuild<Rq, Rs, HFn, HFut, C>
    for RoutesCollectionBuilder<C>
where
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
    C: RequestResponseConverter<Rq, Rs> + Send + Sync + 'static,
{
    fn route(mut self, route: HandlerRoute<Rq, Rs, HFn, HFut>) -> Self {
        self.handlers.insert(
            (
                route.method.clone(),
                format!("{}{}", self.scope_path, route.path),
            ),
            convert_typed_handler(self.converter.clone(), route.handler),
        );
        self
    }
}

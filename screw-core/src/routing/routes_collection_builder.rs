use super::{
    convert_typed_handler, Handler, RequestResponseConverter, RouteFinal, RoutesCollection,
};
use hyper::Method;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

pub struct RoutesCollectionBuilder {
    scope_path: &'static str,
}

impl RoutesCollectionBuilder {
    pub fn with_scope_path(scope_path: &'static str) -> Self {
        Self { scope_path }
    }

    pub fn and_converter<C>(self, converter: C) -> RoutesCollectionBuilderFinal<C>
    where
        C: Send + Sync + 'static,
    {
        RoutesCollectionBuilderFinal {
            scope_path: self.scope_path,
            converter: Arc::new(converter),
            handlers: Default::default(),
        }
    }
}

pub struct RoutesCollectionBuilderFinal<C>
where
    C: Send + Sync + 'static,
{
    scope_path: &'static str,
    converter: Arc<C>,
    handlers: HashMap<(Method, String), Handler>,
}

impl<C> RoutesCollectionBuilderFinal<C>
where
    C: Send + Sync + 'static,
{
    pub fn build(self) -> RoutesCollection {
        RoutesCollection {
            handlers: self.handlers,
        }
    }
}

pub trait RoutesCollectionBuild<Rq, Rs, HFn, HFut>
where
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
{
    fn route(self, route: RouteFinal<Rq, Rs, HFn, HFut>) -> Self;
}

impl<Rq, Rs, HFn, HFut, C> RoutesCollectionBuild<Rq, Rs, HFn, HFut>
    for RoutesCollectionBuilderFinal<C>
where
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
    C: RequestResponseConverter<Rq, Rs> + Send + Sync + 'static,
{
    fn route(mut self, route: RouteFinal<Rq, Rs, HFn, HFut>) -> Self {
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

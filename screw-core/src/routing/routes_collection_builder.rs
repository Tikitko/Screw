use super::{convert_typed_handler, RequestResponseConverter, RouteFinal, RoutesCollection};
use hyper::Method;
use screw_components::dyn_fn::DFn;
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

    pub fn and_converter<ORq, ORs, C>(
        self,
        converter: C,
    ) -> RoutesCollectionBuilderFinal<ORq, ORs, C>
    where
        ORq: Send + 'static,
        ORs: Send + 'static,
        C: Send + Sync + 'static,
    {
        RoutesCollectionBuilderFinal {
            scope_path: self.scope_path,
            converter: Arc::new(converter),
            handlers: Default::default(),
        }
    }
}

pub struct RoutesCollectionBuilderFinal<ORq, ORs, C>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    C: Send + Sync + 'static,
{
    scope_path: &'static str,
    converter: Arc<C>,
    handlers: HashMap<(Method, String), DFn<ORq, ORs>>,
}

impl<ORq, ORs, C> RoutesCollectionBuilderFinal<ORq, ORs, C>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    C: Send + Sync + 'static,
{
    pub fn build(self) -> RoutesCollection<ORq, ORs> {
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

impl<ORq, ORs, C, Rq, Rs, HFn, HFut> RoutesCollectionBuild<Rq, Rs, HFn, HFut>
    for RoutesCollectionBuilderFinal<ORq, ORs, C>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    C: RequestResponseConverter<Rq, Rs, Request = ORq, Response = ORs> + Send + Sync + 'static,
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
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

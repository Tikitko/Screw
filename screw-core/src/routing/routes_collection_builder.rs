use super::{RequestResponseConverter, RequestResponseConverterBase, RouteFinal, RoutesCollection};
use hyper::{Body, Method, Request};
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
        ORq: AsRef<Request<Body>> + Send + 'static,
        ORs: Send + 'static,
        C: RequestResponseConverterBase + Send + Sync + 'static,
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
    ORq: AsRef<Request<Body>> + Send + 'static,
    ORs: Send + 'static,
    C: RequestResponseConverterBase + Send + Sync + 'static,
{
    scope_path: &'static str,
    converter: Arc<C>,
    handlers: HashMap<(Method, String), DFn<ORq, ORs>>,
}

impl<ORq, ORs, C> RoutesCollectionBuilderFinal<ORq, ORs, C>
where
    ORq: AsRef<Request<Body>> + Send + 'static,
    ORs: Send + 'static,
    C: RequestResponseConverterBase + Send + Sync + 'static,
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
    ORq: AsRef<Request<Body>> + Send + 'static,
    ORs: Send + 'static,
    C: RequestResponseConverter<Rq, Rs, Request = ORq, Response = ORs> + Send + Sync + 'static,
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
{
    fn route(mut self, route: RouteFinal<Rq, Rs, HFn, HFut>) -> Self {
        let handler = Arc::new(route.handler);
        let converter = self.converter.clone();
        self.handlers.insert(
            (
                route.method.clone(),
                format!("{}{}", self.scope_path, route.path),
            ),
            Box::new(move |request| {
                let handler = handler.clone();
                let converter = converter.clone();
                Box::pin(async move {
                    let handler_request = converter.convert_request(request).await;
                    let handler_future = handler(handler_request);
                    let handler_response = handler_future.await;
                    let response = converter.convert_response(handler_response).await;
                    response
                })
            }),
        );
        self
    }
}

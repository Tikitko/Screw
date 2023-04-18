use super::*;
use hyper::Method;
use screw_components::dyn_fn::DFn;
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

pub struct RoutesCollectionBuilder<ORq, ORs, C>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    C: Send + Sync + 'static,
{
    scope_path: &'static str,
    converter: Arc<C>,
    handlers: HashMap<(Method, String), DFn<ORq, ORs>>,
}

impl<ORq, ORs, C> RoutesCollectionBuilder<ORq, ORs, C>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    C: Send + Sync + 'static,
{
    pub fn new(params: RoutesCollectionBuilderParams<C>) -> Self {
        Self {
            scope_path: params.scope_path,
            converter: Arc::new(params.converter),
            handlers: Default::default(),
        }
    }

    pub fn route<Rq, Rs, HFn, HFut>(mut self, route: Route<Rq, Rs, HFn, HFut>) -> Self
    where
        C: RequestResponseConverter<Rq, Rs, Request = ORq, Response = ORs>,
        Rq: Send + 'static,
        Rs: Send + 'static,
        HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = Rs> + Send + 'static,
    {
        let handler = Arc::new(route.handler);
        let converter = self.converter.clone();
        self.handlers.insert(
            (
                route.method.clone(),
                self.scope_path.to_string() + route.path,
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

    pub fn build(self) -> RoutesCollection<ORq, ORs> {
        RoutesCollection {
            handlers: self.handlers,
        }
    }
}

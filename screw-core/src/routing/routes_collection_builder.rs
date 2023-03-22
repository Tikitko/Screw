use super::*;
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
    ) -> RoutesCollectionBuilderSecondPart<ORq, ORs, C>
    where
        ORq: Send + 'static,
        ORs: Send + 'static,
        C: Send + Sync + 'static,
    {
        RoutesCollectionBuilderSecondPart {
            scope_path: self.scope_path,
            converter: Arc::new(converter),
            handlers: Default::default(),
        }
    }
}

pub struct RoutesCollectionBuilderSecondPart<ORq, ORs, C>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    C: Send + Sync + 'static,
{
    scope_path: &'static str,
    converter: Arc<C>,
    handlers: HashMap<(Method, String), DFn<ORq, ORs>>,
}

impl<ORq, ORs, C> RoutesCollectionBuilderSecondPart<ORq, ORs, C>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    C: Send + Sync + 'static,
{
    pub fn route<Rq, Rs, HFn, HFut>(mut self, route: RouteThirdPart<Rq, Rs, HFn, HFut>) -> Self
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

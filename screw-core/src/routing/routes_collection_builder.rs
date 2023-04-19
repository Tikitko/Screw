pub use first::*;

use super::*;

pub mod first {
    use super::*;
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
        ) -> second::RoutesCollectionBuilder<ORq, ORs, C>
        where
            ORq: Send + 'static,
            ORs: Send + 'static,
            C: Send + Sync + 'static,
        {
            second::RoutesCollectionBuilder {
                scope_path: self.scope_path,
                converter: Arc::new(converter),
                handlers: Default::default(),
            }
        }
    }
}

pub mod second {
    use super::*;
    use hyper::Method;
    use screw_components::dyn_fn::DFn;
    use std::collections::HashMap;
    use std::future::Future;
    use std::sync::Arc;

    pub struct RoutesCollectionBuilder<ORq, ORs, C>
    where
        ORq: Send + 'static,
        ORs: Send + 'static,
        C: Send + Sync + 'static,
    {
        pub(super) scope_path: &'static str,
        pub(super) converter: Arc<C>,
        pub(super) handlers: HashMap<(Method, String), DFn<ORq, ORs>>,
    }
    
    impl<ORq, ORs, C> RoutesCollectionBuilder<ORq, ORs, C>
    where
        ORq: Send + 'static,
        ORs: Send + 'static,
        C: Send + Sync + 'static,
    {
        pub fn route<Rq, Rs, HFn, HFut>(mut self, route: third::Route<Rq, Rs, HFn, HFut>) -> Self
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
}

pub use first::*;

use super::*;
use hyper::Method;
use screw_components::dyn_fn::DFn;
use std::collections::HashMap;

pub struct Routes<ORq, ORs>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
{
    handlers: HashMap<(&'static Method, String), DFn<ORq, ORs>>,
}

impl<ORq, ORs> Routes<ORq, ORs>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
{
    pub(super) fn handlers(self) -> HashMap<(&'static Method, String), DFn<ORq, ORs>> {
        self.handlers
    }
}

pub mod first {
    use super::*;
    use std::sync::Arc;

    pub struct RoutesBuilder {
        scope_path: &'static str,
    }

    impl RoutesBuilder {
        pub fn with_scope_path(scope_path: &'static str) -> Self {
            Self { scope_path }
        }

        pub fn and_request_converter<ORq, RqC>(
            self,
            request_converter: RqC,
        ) -> second::RoutesBuilder<ORq, RqC>
        where
            ORq: Send + 'static,
            RqC: Send + Sync + 'static,
        {
            second::RoutesBuilder {
                scope_path: self.scope_path,
                request_converter: Arc::new(request_converter),
                _p_orq: Default::default(),
            }
        }
    }
}

pub mod second {
    use super::*;
    use std::{marker::PhantomData, sync::Arc};

    pub struct RoutesBuilder<ORq, RqC>
    where
        ORq: Send + 'static,
        RqC: Send + Sync + 'static,
    {
        pub(super) scope_path: &'static str,
        pub(super) request_converter: Arc<RqC>,
        pub(super) _p_orq: PhantomData<ORq>,
    }

    impl<ORq, RqC> RoutesBuilder<ORq, RqC>
    where
        ORq: Send + 'static,
        RqC: Send + Sync + 'static,
    {
        pub fn and_response_converter<ORs, RsC>(
            self,
            response_converter: RsC,
        ) -> third::RoutesBuilder<ORq, ORs, RqC, RsC>
        where
            ORs: Send + 'static,
            RsC: Send + Sync + 'static,
        {
            third::RoutesBuilder {
                scope_path: self.scope_path,
                request_converter: self.request_converter,
                response_converter: Arc::new(response_converter),
                handlers: Default::default(),
            }
        }
    }
}

pub mod third {
    use super::*;
    use hyper::Method;
    use screw_components::dyn_fn::DFn;
    use std::collections::HashMap;
    use std::future::Future;
    use std::sync::Arc;

    pub struct RoutesBuilder<ORq, ORs, RqC, RsC>
    where
        ORq: Send + 'static,
        ORs: Send + 'static,
        RqC: Send + Sync + 'static,
        RsC: Send + Sync + 'static,
    {
        pub(super) scope_path: &'static str,
        pub(super) request_converter: Arc<RqC>,
        pub(super) response_converter: Arc<RsC>,
        pub(super) handlers: HashMap<(&'static Method, String), DFn<ORq, ORs>>,
    }

    impl<ORq, ORs, RqC, RsC> RoutesBuilder<ORq, ORs, RqC, RsC>
    where
        ORq: Send + 'static,
        ORs: Send + 'static,
        RqC: Send + Sync + 'static,
        RsC: Send + Sync + 'static,
    {
        pub fn route<Rq, Rs, HFn, HFut>(self, route: route::third::Route<Rq, Rs, HFn, HFut>) -> Self
        where
            RqC: RequestConverter<Rq, Request = ORq>,
            RsC: ResponseConverter<Rs, Response = ORs>,
            Rq: Send + 'static,
            Rs: Send + 'static,
            HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
            HFut: Future<Output = Rs> + Send + 'static,
        {
            let Self {
                scope_path,
                request_converter,
                response_converter,
                mut handlers,
            } = self;
            {
                let handler = Arc::new(route.handler);
                let request_converter = request_converter.clone();
                let response_converter = response_converter.clone();
                handlers.insert(
                    (route.method, self.scope_path.to_owned() + route.path),
                    Box::new(move |request| {
                        let handler = handler.clone();
                        let request_converter = request_converter.clone();
                        let response_converter = response_converter.clone();
                        Box::pin(async move {
                            let handler_request = request_converter.convert_request(request).await;
                            let handler_future = handler(handler_request);
                            let handler_response = handler_future.await;
                            let response =
                                response_converter.convert_response(handler_response).await;
                            response
                        })
                    }),
                );
            }
            Self {
                scope_path: scope_path,
                request_converter: request_converter,
                response_converter: response_converter,
                handlers,
            }
        }

        pub fn build(self) -> Routes<ORq, ORs> {
            Routes {
                handlers: self.handlers,
            }
        }
    }
}

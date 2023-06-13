use super::*;
use hyper::Method;
use screw_components::dyn_fn::DFn;
use std::{collections::HashMap, future::Future, sync::Arc};

pub struct Converters<RqC, RsC> {
    pub request_converter: RqC,
    pub response_converter: RsC,
}

pub struct Routes<ORq, ORs, RqC, RsC>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    RqC: Send + Sync + 'static,
    RsC: Send + Sync + 'static,
{
    pub(super) scope_path: String,
    pub(super) request_converter: Arc<RqC>,
    pub(super) response_converter: Arc<RsC>,
    pub(super) handlers: HashMap<String, HashMap<&'static Method, DFn<ORq, ORs>>>,
}

impl<ORq, ORs, RqC, RsC> Routes<ORq, ORs, RqC, RsC>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    RqC: Send + Sync + 'static,
    RsC: Send + Sync + 'static,
{
    pub fn scoped<F>(self, scope_path: &'static str, handler: F) -> Self
    where
        F: FnOnce(Routes<ORq, ORs, RqC, RsC>) -> Routes<ORq, ORs, RqC, RsC>,
    {
        let Self { handlers, .. } = handler(Self {
            scope_path: self.scope_path.clone() + scope_path,
            request_converter: self.request_converter.clone(),
            response_converter: self.response_converter.clone(),
            handlers: self.handlers,
        });
        Self {
            scope_path: self.scope_path,
            request_converter: self.request_converter,
            response_converter: self.response_converter,
            handlers,
        }
    }

    pub fn scoped_convertable<Rq, Rs, NRqC, NRsC, F>(
        self,
        scope_path: &'static str,
        converters: Converters<NRqC, NRsC>,
        handler: F,
    ) -> Self
    where
        RqC: converter::RequestConverter<Rq, Request = ORq>,
        RsC: converter::ResponseConverter<Rs, Response = ORs>,
        Rq: Send + 'static,
        Rs: Send + 'static,
        NRqC: Send + Sync + 'static,
        NRsC: Send + Sync + 'static,
        F: FnOnce(Routes<Rq, Rs, NRqC, NRsC>) -> Routes<Rq, Rs, NRqC, NRsC>,
    {
        let Routes {
            handlers: converted_handlers,
            ..
        } = handler(Routes {
            scope_path: self.scope_path.clone() + scope_path,
            request_converter: Arc::new(converters.request_converter),
            response_converter: Arc::new(converters.response_converter),
            handlers: HashMap::new(),
        });
        let handlers = {
            let mut handlers = self.handlers;
            for (path, converted_handlers) in converted_handlers {
                for (method, converted_handler) in converted_handlers {
                    Self::convert_route_to_handlers(
                        route::first::Route::with_method(method)
                            .and_path(path.clone())
                            .and_handler(converted_handler),
                        self.request_converter.clone(),
                        self.response_converter.clone(),
                        &mut handlers,
                    )
                }
            }
            handlers
        };
        Self {
            scope_path: self.scope_path,
            request_converter: self.request_converter,
            response_converter: self.response_converter,
            handlers,
        }
    }

    pub fn convertable<Rq, Rs, NRqC, NRsC, F>(
        self,
        converters: Converters<NRqC, NRsC>,
        handler: F,
    ) -> Self
    where
        RqC: converter::RequestConverter<Rq, Request = ORq>,
        RsC: converter::ResponseConverter<Rs, Response = ORs>,
        Rq: Send + 'static,
        Rs: Send + 'static,
        NRqC: Send + Sync + 'static,
        NRsC: Send + Sync + 'static,
        F: FnOnce(Routes<Rq, Rs, NRqC, NRsC>) -> Routes<Rq, Rs, NRqC, NRsC>,
    {
        self.scoped_convertable("", converters, handler)
    }

    pub fn route<Rq, Rs, HFn, HFut>(self, route: route::third::Route<Rq, Rs, HFn, HFut>) -> Self
    where
        RqC: converter::RequestConverter<Rq, Request = ORq>,
        RsC: converter::ResponseConverter<Rs, Response = ORs>,
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
            Self::convert_route_to_handlers(
                route::first::Route::with_method(route.method)
                    .and_path(scope_path.clone() + route.path.as_str())
                    .and_handler(route.handler),
                request_converter.clone(),
                response_converter.clone(),
                &mut handlers,
            )
        }
        Self {
            scope_path,
            request_converter,
            response_converter,
            handlers,
        }
    }

    fn convert_route_to_handlers<Rq, Rs, HFn, HFut>(
        route: route::third::Route<Rq, Rs, HFn, HFut>,
        request_converter: Arc<RqC>,
        response_converter: Arc<RsC>,
        handlers: &mut HashMap<String, HashMap<&'static Method, DFn<ORq, ORs>>>,
    ) where
        RqC: converter::RequestConverter<Rq, Request = ORq>,
        RsC: converter::ResponseConverter<Rs, Response = ORs>,
        Rq: Send + 'static,
        Rs: Send + 'static,
        HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = Rs> + Send + 'static,
    {
        let mut method_handlers = handlers.remove(&route.path).unwrap_or_default();
        let handler = Arc::new(route.handler);
        let request_converter = request_converter.clone();
        let response_converter = response_converter.clone();
        method_handlers.insert(
            route.method,
            Box::new(move |request| {
                let handler = handler.clone();
                let request_converter = request_converter.clone();
                let response_converter = response_converter.clone();
                Box::pin(async move {
                    let handler_request = request_converter.convert_request(request).await;
                    let handler_future = handler(handler_request);
                    let handler_response = handler_future.await;
                    let response = response_converter.convert_response(handler_response).await;
                    response
                })
            }),
        );
        handlers.insert(route.path, method_handlers);
    }
}

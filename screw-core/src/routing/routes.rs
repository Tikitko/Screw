use super::*;
use hyper::Method;
use screw_components::dyn_fn::DFn;
use std::{collections::HashMap, future::Future, sync::Arc};

pub struct Routes<ORq, ORs>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
{
    pub(super) scope_path: String,
    pub(super) handlers: HashMap<String, HashMap<&'static Method, DFn<ORq, ORs>>>,
}

impl<ORq, ORs> Routes<ORq, ORs>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
{
    pub fn scoped<F>(self, scope_path: &'static str, handler: F) -> Self
    where
        F: FnOnce(Routes<ORq, ORs>) -> Routes<ORq, ORs>,
    {
        let Self { handlers, .. } = handler(Self {
            scope_path: self.scope_path.clone() + scope_path,
            handlers: self.handlers,
        });
        Self {
            scope_path: self.scope_path,
            handlers,
        }
    }

    pub fn scoped_convertable<F, RqC, RsC>(
        self,
        scope_path: &'static str,
        converters: Converters<RqC, RsC>,
        handler: F,
    ) -> Self
    where
        F: FnOnce(ConvertableRoutes<ORq, ORs, RqC, RsC>) -> ConvertableRoutes<ORq, ORs, RqC, RsC>,
        RqC: Send + Sync + 'static,
        RsC: Send + Sync + 'static,
    {
        let ConvertableRoutes { handlers, .. } = handler(ConvertableRoutes {
            scope_path: self.scope_path.clone() + scope_path,
            request_converter: Arc::new(converters.request_converter),
            response_converter: Arc::new(converters.response_converter),
            handlers: self.handlers,
        });
        Self {
            scope_path: self.scope_path,
            handlers,
        }
    }

    pub fn convertable<F, RqC, RsC>(self, converters: Converters<RqC, RsC>, handler: F) -> Self
    where
        F: FnOnce(ConvertableRoutes<ORq, ORs, RqC, RsC>) -> ConvertableRoutes<ORq, ORs, RqC, RsC>,
        RqC: Send + Sync + 'static,
        RsC: Send + Sync + 'static,
    {
        self.scoped_convertable("", converters, handler)
    }
}

pub struct Converters<RqC, RsC>
where
    RqC: Send + Sync + 'static,
    RsC: Send + Sync + 'static,
{
    pub request_converter: RqC,
    pub response_converter: RsC,
}

pub struct ConvertableRoutes<ORq, ORs, RqC, RsC>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    RqC: Send + Sync + 'static,
    RsC: Send + Sync + 'static,
{
    scope_path: String,
    request_converter: Arc<RqC>,
    response_converter: Arc<RsC>,
    handlers: HashMap<String, HashMap<&'static Method, DFn<ORq, ORs>>>,
}

impl<ORq, ORs, RqC, RsC> ConvertableRoutes<ORq, ORs, RqC, RsC>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    RqC: Send + Sync + 'static,
    RsC: Send + Sync + 'static,
{
    pub fn scoped<F>(self, scope_path: &'static str, handler: F) -> Self
    where
        F: FnOnce(ConvertableRoutes<ORq, ORs, RqC, RsC>) -> ConvertableRoutes<ORq, ORs, RqC, RsC>,
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
            let handler = Arc::new(route.handler);
            let request_converter = request_converter.clone();
            let response_converter = response_converter.clone();
            let path = scope_path.to_owned() + route.path;
            let mut method_handlers = handlers.remove(&path).unwrap_or_default();
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
            handlers.insert(path, method_handlers);
        }
        Self {
            scope_path,
            request_converter,
            response_converter,
            handlers,
        }
    }
}

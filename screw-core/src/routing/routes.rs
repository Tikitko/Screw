use super::*;
use hyper::Method;
use screw_components::dyn_fn::{DFn, DFnOnce};
use std::future::Future;
use std::sync::Arc;

pub struct Routes<ORq, ORs, M>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    M: Send + Sync + 'static,
{
    scope_path: String,
    middleware: Arc<M>,
    handlers: Vec<(Vec<&'static Method>, String, DFn<ORq, ORs>)>,
}

impl<ORq, ORs> Routes<ORq, ORs, ()>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
{
    pub(super) fn new() -> Self {
        Self {
            scope_path: "".to_owned(),
            middleware: Arc::new(()),
            handlers: Vec::new(),
        }
    }
    pub(super) fn handlers(self) -> Vec<(Vec<&'static Method>, String, DFn<ORq, ORs>)> {
        self.handlers
    }
}

impl<ORq, ORs, M> Routes<ORq, ORs, M>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    M: Send + Sync + 'static,
{
    pub fn scoped<F>(self, scope_path: &'static str, handler: F) -> Self
    where
        F: FnOnce(Routes<ORq, ORs, M>) -> Routes<ORq, ORs, M>,
    {
        let Self { handlers, .. } = handler(Self {
            scope_path: self.scope_path.clone() + scope_path,
            middleware: self.middleware.clone(),
            handlers: self.handlers,
        });
        Self {
            scope_path: self.scope_path,
            middleware: self.middleware,
            handlers,
        }
    }

    pub fn scoped_middleware<Rq, Rs, NM, F>(
        self,
        scope_path: &'static str,
        middleware: NM,
        handler: F,
    ) -> Self
    where
        M: middleware::Middleware<Rq, Rs, Request = ORq, Response = ORs>,
        Rq: Send + 'static,
        Rs: Send + 'static,
        NM: Send + Sync + 'static,
        F: FnOnce(Routes<Rq, Rs, NM>) -> Routes<Rq, Rs, NM>,
    {
        let Routes {
            handlers: middleware_handlers,
            ..
        } = handler(Routes {
            scope_path: self.scope_path.clone() + scope_path,
            middleware: Arc::new(middleware),
            handlers: Vec::new(),
        });
        let handlers = {
            let mut handlers = self.handlers;
            for (methods, path, middleware_handler) in middleware_handlers {
                Self::add_route_to_handlers(
                    route::first::Route::with_methods(methods)
                        .and_path(path)
                        .and_handler(middleware_handler),
                    &mut handlers,
                    self.middleware.clone(),
                )
            }
            handlers
        };
        Self {
            scope_path: self.scope_path,
            middleware: self.middleware,
            handlers,
        }
    }

    pub fn middleware<Rq, Rs, NM, F>(self, middleware: NM, handler: F) -> Self
    where
        M: middleware::Middleware<Rq, Rs, Request = ORq, Response = ORs>,
        Rq: Send + 'static,
        Rs: Send + 'static,
        NM: Send + Sync + 'static,
        F: FnOnce(Routes<Rq, Rs, NM>) -> Routes<Rq, Rs, NM>,
    {
        self.scoped_middleware("", middleware, handler)
    }

    pub fn route<FRq, Rq, IRs, Rs, HFn, HFut>(
        self,
        route: route::third::Route<FRq, IRs, HFn, HFut>,
    ) -> Self
    where
        M: middleware::Middleware<Rq, Rs, Request = ORq, Response = ORs>,
        FRq: From<Rq> + Send + 'static,
        Rq: Send + 'static,
        IRs: Into<Rs> + Send + 'static,
        Rs: Send + 'static,
        HFn: Fn(FRq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = IRs> + Send + 'static,
    {
        let Self {
            scope_path,
            middleware,
            mut handlers,
        } = self;
        {
            Self::add_route_to_handlers(
                route::first::Route::with_methods(route.methods)
                    .and_path(scope_path.clone() + route.path.as_str())
                    .and_handler(route.handler),
                &mut handlers,
                middleware.clone(),
            )
        }
        Self {
            scope_path,
            middleware,
            handlers,
        }
    }

    fn add_route_to_handlers<FRq, Rq, IRs, Rs, HFn, HFut>(
        route: route::third::Route<FRq, IRs, HFn, HFut>,
        handlers: &mut Vec<(Vec<&'static Method>, String, DFn<ORq, ORs>)>,
        middleware: Arc<M>,
    ) where
        M: middleware::Middleware<Rq, Rs, Request = ORq, Response = ORs>,
        FRq: From<Rq> + Send + 'static,
        Rq: Send + 'static,
        IRs: Into<Rs> + Send + 'static,
        Rs: Send + 'static,
        HFn: Fn(FRq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = IRs> + Send + 'static,
    {
        let handler = Arc::new(route.handler);
        let middleware = middleware.clone();
        handlers.push((
            route.methods,
            route.path,
            Box::new(move |request| {
                let handler = handler.clone();
                let middleware = middleware.clone();
                Box::pin(async move {
                    let next: DFnOnce<Rq, Rs> = Box::new(move |rq| {
                        Box::pin(async move {
                            let rs = handler(From::from(rq)).await.into();
                            rs
                        })
                    });
                    middleware.respond(request, next).await
                })
            }),
        ));
    }
}

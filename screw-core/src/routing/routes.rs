use super::*;
use hyper::Method;
use screw_components::dyn_fn::DFn;
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

    pub fn scoped_convertable<Rq, Rs, NM, F>(
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
            handlers: converted_handlers,
            ..
        } = handler(Routes {
            scope_path: self.scope_path.clone() + scope_path,
            middleware: Arc::new(middleware),
            handlers: Vec::new(),
        });
        let handlers = {
            let mut handlers = self.handlers;
            for (methods, path, converted_handler) in converted_handlers {
                Self::add_route_to_handlers(
                    route::first::Route::with_methods(methods)
                        .and_path(path)
                        .and_handler(converted_handler),
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

    pub fn convertable<Rq, Rs, NM, F>(self, middleware: NM, handler: F) -> Self
    where
        M: middleware::Middleware<Rq, Rs, Request = ORq, Response = ORs>,
        Rq: Send + 'static,
        Rs: Send + 'static,
        NM: Send + Sync + 'static,
        F: FnOnce(Routes<Rq, Rs, NM>) -> Routes<Rq, Rs, NM>,
    {
        self.scoped_convertable("", middleware, handler)
    }

    pub fn route<Rq, Rs, HFn, HFut>(self, route: route::third::Route<Rq, Rs, HFn, HFut>) -> Self
    where
        M: middleware::Middleware<Rq, Rs, Request = ORq, Response = ORs>,
        Rq: Send + 'static,
        Rs: Send + 'static,
        HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = Rs> + Send + 'static,
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

    fn add_route_to_handlers<Rq, Rs, HFn, HFut>(
        route: route::third::Route<Rq, Rs, HFn, HFut>,
        handlers: &mut Vec<(Vec<&'static Method>, String, DFn<ORq, ORs>)>,
        middleware: Arc<M>,
    ) where
        M: middleware::Middleware<Rq, Rs, Request = ORq, Response = ORs>,
        Rq: Send + 'static,
        Rs: Send + 'static,
        HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = Rs> + Send + 'static,
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
                    middleware
                        .respond(
                            request,
                            Box::new(move |i| Box::pin(async move { handler(i).await })),
                        )
                        .await
                })
            }),
        ));
    }
}

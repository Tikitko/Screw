use super::{
    convert_handler, Handler, RequestConverter, ResponseConverter, Route, RouterBuilderRoute,
};
use hyper::Method;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ScopedRouterBuilder<C>
where
    C: Send + Sync + 'static,
{
    path: &'static str,
    converter: Arc<C>,
    handlers: HashMap<(Method, String), Handler>,
}

impl<C> ScopedRouterBuilder<C>
where
    C: Send + Sync + 'static,
{
    pub fn new(path: &'static str, converter: C) -> Self
    where
        C: Send + Sync + 'static,
    {
        Self {
            path,
            converter: Arc::new(converter),
            handlers: Default::default(),
        }
    }

    pub(super) fn handlers(self) -> HashMap<(Method, String), Handler> {
        self.handlers
    }
}

impl<C, R> RouterBuilderRoute<C, R> for ScopedRouterBuilder<C>
where
    C: RequestConverter<R::Rq> + ResponseConverter<R::Rs> + Send + Sync + 'static,
    R: Route + 'static,
{
    fn route(mut self, _route: R) -> Self {
        let method = R::method().clone();
        let path = format!("{}{}", self.path, R::path());
        let handler = convert_handler(self.converter.clone(), R::handler);

        self.handlers.insert((method, path), handler);
        self
    }
}

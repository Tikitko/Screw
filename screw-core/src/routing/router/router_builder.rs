use super::{
    convert_handler, Handler, RequestConverter, ResponseConverter, Route, Router,
    ScopedRouterBuilder,
};
use derive_error::Error;
use hyper::Method;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

pub trait RouterBuilderNotFound<C, Rq, Rs, HFn, HFut>
where
    C: RequestConverter<Rq> + ResponseConverter<Rs> + Send + Sync + 'static,
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
{
    fn not_found_handler(self, not_found_handler: HFn) -> Self;
}

pub trait RouterBuilderRoute<C, R>
where
    C: RequestConverter<R::Rq> + ResponseConverter<R::Rs> + Send + Sync + 'static,
    R: Route + 'static,
{
    fn route(self, _route: R) -> Self;
}

#[derive(Error, Debug)]
pub enum RouterBuilderError {
    NotFoundHandlerMissing,
}

pub struct RouterBuilder<C>
where
    C: Send + Sync + 'static,
{
    converter: Arc<C>,
    not_found_handler: Option<Handler>,
    handlers: HashMap<(Method, String), Handler>,
}

impl<C> RouterBuilder<C>
where
    C: Send + Sync + 'static,
{
    pub fn new(converter: C) -> Self
    where
        C: Send + Sync + 'static,
    {
        Self {
            converter: Arc::new(converter),
            not_found_handler: None,
            handlers: Default::default(),
        }
    }

    pub fn extend<SC>(mut self, scoped_builder: ScopedRouterBuilder<SC>) -> Self
    where
        SC: Send + Sync + 'static,
    {
        let scoped_handlers = scoped_builder.handlers();
        self.handlers.extend(scoped_handlers);
        self
    }

    pub fn build(self) -> Result<Router, RouterBuilderError> {
        Ok(Router {
            handlers: self.handlers,
            not_found_handler: self
                .not_found_handler
                .ok_or(RouterBuilderError::NotFoundHandlerMissing)?,
        })
    }
}

impl<C, Rq, Rs, HFn, HFut> RouterBuilderNotFound<C, Rq, Rs, HFn, HFut> for RouterBuilder<C>
where
    C: RequestConverter<Rq> + ResponseConverter<Rs> + Send + Sync + 'static,
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
{
    fn not_found_handler(mut self, not_found_handler: HFn) -> Self {
        self.not_found_handler = Some(convert_handler(self.converter.clone(), not_found_handler));
        self
    }
}

impl<C, R> RouterBuilderRoute<C, R> for RouterBuilder<C>
where
    C: RequestConverter<R::Rq> + ResponseConverter<R::Rs> + Send + Sync + 'static,
    R: Route + 'static,
{
    fn route(mut self, _route: R) -> Self {
        let method = R::method().clone();
        let path = R::path().to_string();
        let handler = convert_handler(self.converter.clone(), R::handler);

        self.handlers.insert((method, path), handler);
        self
    }
}

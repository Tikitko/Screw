use super::{convert_generic_handler, Router, RoutesCollection};
use crate::routing::router::HandlerRoute;
use crate::routing::{Handler, Request, Response};
use derive_error::Error;
use hyper::Method;
use std::collections::HashMap;
use std::future::Future;

#[derive(Error, Debug)]
pub enum RouterBuilderError {
    FallbackHandlerMissing,
}

pub struct RouterBuilder {
    handlers: HashMap<(Method, String), Handler>,
    fallback_handler: Option<Handler>,
}

impl Default for RouterBuilder {
    fn default() -> Self {
        Self {
            handlers: Default::default(),
            fallback_handler: None,
        }
    }
}

impl RouterBuilder {
    pub fn fallback_handler<HFn, HFut>(mut self, fallback_handler: HFn) -> Self
    where
        HFn: Fn(Request) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = Response> + Send + 'static,
    {
        self.fallback_handler = Some(convert_generic_handler(fallback_handler));
        self
    }

    pub fn route<HFn, HFut>(mut self, route: HandlerRoute<Request, Response, HFn, HFut>) -> Self
    where
        HFn: Fn(Request) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = Response> + Send + 'static,
    {
        self.handlers.insert(
            (route.method.clone(), route.path.to_string()),
            convert_generic_handler(route.handler),
        );
        self
    }

    pub fn routes(mut self, routes: RoutesCollection) -> Self {
        self.handlers.extend(routes.handlers);
        self
    }

    pub fn build(self) -> Result<Router, RouterBuilderError> {
        Ok(Router {
            handlers: self.handlers,
            fallback_handler: self
                .fallback_handler
                .ok_or(RouterBuilderError::FallbackHandlerMissing)?,
        })
    }
}

use super::*;
use hyper::{Body, Method, Request};
use screw_components::dyn_fn::{AsDynFn, DFn};
use std::collections::HashMap;
use std::future::Future;

pub struct Router<ORq, ORs> {
    pub(super) handlers: HashMap<(&'static Method, String), DFn<ORq, ORs>>,
    pub(super) fallback_handler: DFn<ORq, ORs>,
}

impl<ORq, ORs> Router<ORq, ORs>
where
    ORq: AsRef<Request<Body>>,
{
    pub async fn process(&self, request: ORq) -> ORs {
        let http_request_ref = request.as_ref();

        let method = http_request_ref.method().to_owned();
        let path = http_request_ref.uri().path().to_owned();

        let handler = self
            .handlers
            .get(&(&method, path))
            .unwrap_or(&self.fallback_handler);

        let response = handler(request).await;

        response
    }
}

pub struct RouterBuilder<ORq, ORs>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
{
    handlers: HashMap<(&'static Method, String), DFn<ORq, ORs>>,
    fallback_handler: DFn<ORq, ORs>,
}

impl<ORq, ORs> RouterBuilder<ORq, ORs>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
{
    pub fn with_fallback_handler<HFn, HFut>(fallback_handler: HFn) -> Self
    where
        HFn: Fn(ORq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = ORs> + Send + 'static,
    {
        RouterBuilder {
            handlers: Default::default(),
            fallback_handler: fallback_handler.to_dyn_fn(),
        }
    }

    pub fn routes(self, routes: Routes<ORq, ORs>) -> Self {
        let Self {
            mut handlers,
            fallback_handler,
        } = self;
        {
            handlers.extend(routes.handlers());
        }
        Self {
            handlers,
            fallback_handler,
        }
    }

    pub fn build(self) -> Router<ORq, ORs> {
        Router {
            handlers: self.handlers,
            fallback_handler: self.fallback_handler,
        }
    }
}

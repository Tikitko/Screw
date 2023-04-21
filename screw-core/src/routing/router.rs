use hyper::{Body, Method, Request};
use screw_components::dyn_fn::DFn;
use std::collections::HashMap;

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

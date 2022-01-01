use hyper::{Body, Method, Request};
use screw_components::dyn_fn::DFn;
use std::collections::HashMap;

pub struct Router<ORq, ORs>
where
    ORq: AsRef<Request<Body>>,
{
    pub(super) handlers: HashMap<(Method, String), DFn<ORq, ORs>>,
    pub(super) fallback_handler: DFn<ORq, ORs>,
}

impl<ORq, ORs> Router<ORq, ORs>
where
    ORq: AsRef<Request<Body>>,
{
    pub(super) async fn process(&self, request: ORq) -> ORs {
        let http_request_ref = request.as_ref();

        let method = http_request_ref.method().clone();
        let path = http_request_ref.uri().clone().path().to_string();

        let clean_path = format!(
            "/{}",
            path.split('/')
                .filter(|seg| !seg.is_empty())
                .collect::<Vec<&str>>()
                .join("/")
        );

        let handler = self
            .handlers
            .get(&(method, clean_path))
            .unwrap_or(&self.fallback_handler);

        let response = handler(request).await;

        response
    }
}

use crate::routing::{Handler, Request, Response};
use hyper::Method;
use std::collections::HashMap;

pub struct Router {
    pub(super) handlers: HashMap<(Method, String), Handler>,
    pub(super) fallback_handler: Handler,
}

impl Router {
    pub(crate) async fn process(&self, request: Request) -> Response {
        let method = request.http.method().clone();
        let path = {
            let mut path = request.http.uri().path().to_string();
            if path.ends_with('/') {
                path = (&path[..path.len() - 1]).to_string();
            }
            path
        };

        let handler = self
            .handlers
            .get(&(method, path))
            .unwrap_or(&self.fallback_handler);

        let response = handler(request).await;

        response
    }
}

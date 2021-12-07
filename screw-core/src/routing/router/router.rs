use super::{Handler, Request, Response};
use hyper::Method;
use std::collections::HashMap;

pub struct Router {
    pub(super) handlers: HashMap<(Method, String), Handler>,
    pub(super) not_found_handler: Handler,
}

impl Router {
    pub async fn process(&self, request: Request) -> Response {
        let method = request.method().clone();
        let path = {
            let mut path = request.uri().path().to_owned();
            if path.ends_with('/') {
                path = (&path[..path.len() - 1]).to_string();
            }
            path
        };

        let handler = match self.handlers.get(&(method, path)) {
            Some(handler) => handler,
            None => &self.not_found_handler,
        };

        let response = handler(request).await;

        response
    }
}

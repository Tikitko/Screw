use super::{Request, Response};
use hyper::{Body, StatusCode};

pub async fn not_found_fallback_handler(_request: Request) -> Response {
    Response {
        http: hyper::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}

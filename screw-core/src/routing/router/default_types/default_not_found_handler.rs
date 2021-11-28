use hyper::{Body, Request, Response, StatusCode};

pub async fn default_not_found_http_handler(_request: Request<Body>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
}

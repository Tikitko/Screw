use crate::routing::router::{Request, RequestConverter, Response, ResponseConverter};
use async_trait::async_trait;

pub struct DefaultConverter;

#[async_trait]
impl RequestConverter<Request> for DefaultConverter {
    async fn convert_request(&self, request: Request) -> Request {
        request
    }
}

#[async_trait]
impl ResponseConverter<Response> for DefaultConverter {
    async fn convert_response(&self, response: Response) -> Response {
        response
    }
}

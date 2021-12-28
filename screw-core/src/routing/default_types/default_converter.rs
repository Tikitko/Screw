use crate::routing::{Request, RequestResponseConverter, Response};
use async_trait::async_trait;

pub struct DefaultConverter;

#[async_trait]
impl RequestResponseConverter<Request, Response> for DefaultConverter {
    async fn convert_request(&self, request: Request) -> Request {
        request
    }
    async fn convert_response(&self, response: Response) -> Response {
        response
    }
}

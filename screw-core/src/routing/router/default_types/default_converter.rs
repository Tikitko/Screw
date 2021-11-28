use crate::routing::router::http::HttpConverter;
use crate::routing::router::http::{Request, Response};
use crate::routing::router::web_socket::StreamableRequest;
use crate::routing::router::web_socket::WebSocketConverter;
use async_trait::async_trait;

pub struct DefaultConverter;

#[async_trait]
impl HttpConverter<Request, Response> for DefaultConverter {
    async fn convert_request(&self, request: Request) -> Request {
        request
    }

    async fn convert_response(&self, response: Response) -> Response {
        response
    }
}

#[async_trait]
impl WebSocketConverter<StreamableRequest> for DefaultConverter {
    async fn convert_streamable_request(
        &self,
        streamable_request: StreamableRequest,
    ) -> StreamableRequest {
        streamable_request
    }
}

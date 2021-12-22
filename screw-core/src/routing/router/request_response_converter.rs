use crate::routing::{Request, Response};
use async_trait::async_trait;

#[async_trait]
pub trait RequestResponseConverter<Rq, Rs> {
    async fn convert_request(&self, request: Request) -> Rq;
    async fn convert_response(&self, response: Rs) -> Response;
}

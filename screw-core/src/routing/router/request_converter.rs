use super::Request;
use async_trait::async_trait;

#[async_trait]
pub trait RequestConverter<Rq> {
    async fn convert_request(&self, request: Request) -> Rq;
}

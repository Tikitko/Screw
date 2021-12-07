use super::Response;
use async_trait::async_trait;

#[async_trait]
pub trait ResponseConverter<Rs> {
    async fn convert_response(&self, response: Rs) -> Response;
}

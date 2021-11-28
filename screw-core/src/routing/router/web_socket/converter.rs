use super::StreamableRequest;
use async_trait::async_trait;

#[async_trait]
pub trait WebSocketConverter<SRq> {
    async fn convert_streamable_request(&self, streamable_request: StreamableRequest) -> SRq;
}

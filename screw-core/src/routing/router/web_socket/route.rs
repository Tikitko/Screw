use async_trait::async_trait;

#[async_trait]
pub trait WebSocketRoute {
    type SRq: Send + 'static;
    fn path() -> &'static str;
    async fn handler(streamable_request: Self::SRq);
}

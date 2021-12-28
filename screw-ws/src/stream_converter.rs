use async_trait::async_trait;
use hyper::upgrade::Upgraded;
use tokio_tungstenite::WebSocketStream;

#[async_trait]
pub trait WebSocketStreamConverter<Stream> {
    async fn convert_stream(&self, stream: WebSocketStream<Upgraded>) -> Stream;
}

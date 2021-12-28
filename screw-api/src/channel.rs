use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use hyper::http::request::Parts;
use hyper::http::Extensions;
use hyper::upgrade::Upgraded;
use screw_components::dyn_fn::DFn;
use screw_components::dyn_result::{DError, DResult};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::WebSocketStream;

pub struct ApiChannelOriginContent {
    pub http_parts: Parts,
    pub remote_addr: SocketAddr,
    pub extensions: Arc<Extensions>,
}

pub trait ApiChannelContent {
    fn create(origin_content: ApiChannelOriginContent) -> Self;
}

pub struct ApiChannel<Send, Receive>
where
    Send: Serialize + std::marker::Send + 'static,
    Receive: for<'de> Deserialize<'de> + std::marker::Send + 'static,
{
    pub sender: ApiChannelSenderFinal<Send>,
    pub receiver: ApiChannelReceiverFinal<Receive>,
}

pub enum ApiChannelSenderError {
    Convert(DError),
    Tungstenite(Error),
}

pub struct ApiChannelSender {
    sink: SplitSink<WebSocketStream<Upgraded>, Message>,
}

impl ApiChannelSender {
    pub fn with_sink(sink: SplitSink<WebSocketStream<Upgraded>, Message>) -> Self {
        Self { sink }
    }

    pub fn and_converter<Send, HFn, HFut>(self, converter: HFn) -> ApiChannelSenderFinal<Send>
    where
        Send: Serialize + std::marker::Send + 'static,
        HFn: Fn(Send) -> HFut + std::marker::Send + Sync + 'static,
        HFut: Future<Output = DResult<String>> + std::marker::Send + 'static,
    {
        let converter = Arc::new(converter);
        ApiChannelSenderFinal {
            converter: Box::new(move |message| {
                let converter = converter.clone();
                Box::pin(async move { converter(message).await })
            }),
            sink: self.sink,
        }
    }
}

pub struct ApiChannelSenderFinal<Send>
where
    Send: Serialize + std::marker::Send + 'static,
{
    converter: DFn<Send, DResult<String>>,
    sink: SplitSink<WebSocketStream<Upgraded>, Message>,
}

impl<Send> ApiChannelSenderFinal<Send>
where
    Send: Serialize + std::marker::Send + 'static,
{
    pub async fn send(&mut self, message: Send) -> Result<(), ApiChannelSenderError> {
        let converter = &self.converter;

        let message_string = converter(message)
            .await
            .map_err(ApiChannelSenderError::Convert)?;
        self.sink
            .send(Message::Text(message_string))
            .await
            .map_err(ApiChannelSenderError::Tungstenite)
    }

    pub async fn close(&mut self) -> Result<(), ApiChannelSenderError> {
        self.sink
            .send(Message::Close(None))
            .await
            .map_err(ApiChannelSenderError::Tungstenite)
    }
}

pub enum ApiChannelReceiverError {
    Convert(DError),
    Tungstenite(Error),
    NoMessage,
    UnsupportedMessage,
    Closed,
}

pub struct ApiChannelReceiver {
    stream: SplitStream<WebSocketStream<Upgraded>>,
}

impl ApiChannelReceiver {
    pub fn with_stream(stream: SplitStream<WebSocketStream<Upgraded>>) -> Self {
        Self { stream }
    }

    pub fn and_converter<Receive, HFn, HFut>(
        self,
        converter: HFn,
    ) -> ApiChannelReceiverFinal<Receive>
    where
        for<'de> Receive: Deserialize<'de> + std::marker::Send + 'static,
        HFn: Fn(String) -> HFut + std::marker::Send + Sync + 'static,
        HFut: Future<Output = DResult<Receive>> + std::marker::Send + 'static,
    {
        let converter = Arc::new(converter);
        ApiChannelReceiverFinal {
            converter: Box::new(move |message| {
                let converter = converter.clone();
                Box::pin(async move { converter(message).await })
            }),
            stream: self.stream,
        }
    }
}

pub struct ApiChannelReceiverFinal<Receive>
where
    for<'de> Receive: Deserialize<'de> + std::marker::Send + 'static,
{
    stream: SplitStream<WebSocketStream<Upgraded>>,
    converter: DFn<String, DResult<Receive>>,
}

impl<Receive> ApiChannelReceiverFinal<Receive>
where
    for<'de> Receive: Deserialize<'de> + std::marker::Send + 'static,
{
    pub async fn next_message(&mut self) -> Result<Receive, ApiChannelReceiverError> {
        let converter = &self.converter;

        match self.stream.next().await {
            None => Err(ApiChannelReceiverError::NoMessage),
            Some(message_result) => match message_result {
                Ok(message) => match message {
                    Message::Text(test) => converter(test)
                        .await
                        .map_err(ApiChannelReceiverError::Convert),
                    Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => {
                        Err(ApiChannelReceiverError::UnsupportedMessage)
                    }
                    Message::Close(_) => Err(ApiChannelReceiverError::Closed),
                },
                Err(e) => Err(ApiChannelReceiverError::Tungstenite(e)),
            },
        }
    }
}

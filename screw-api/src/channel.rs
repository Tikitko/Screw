use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use hyper::http::request::Parts;
use hyper::http::Extensions;
use hyper::upgrade::Upgraded;
use screw_core::DFn;
use screw_core::{DError, DResult};
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
    sender: ApiChannelSender<Send>,
    receiver: ApiChannelReceiver<Receive>,
}

impl<Send, Receive> ApiChannel<Send, Receive>
where
    Send: Serialize + std::marker::Send + 'static,
    Receive: for<'de> Deserialize<'de> + std::marker::Send + 'static,
{
    pub fn new(sender: ApiChannelSender<Send>, receiver: ApiChannelReceiver<Receive>) -> Self {
        Self { sender, receiver }
    }

    pub fn split(self) -> (ApiChannelSender<Send>, ApiChannelReceiver<Receive>) {
        (self.sender, self.receiver)
    }

    pub fn sender_mut_ref(&mut self) -> &mut ApiChannelSender<Send> {
        &mut self.sender
    }

    pub fn receiver_mut_ref(&mut self) -> &mut ApiChannelReceiver<Receive> {
        &mut self.receiver
    }
}

pub enum ApiChannelSenderError {
    Convert(DError),
    Tungstenite(Error),
}

pub struct ApiChannelSender<Send>
where
    Send: Serialize + std::marker::Send + 'static,
{
    converter: DFn<Send, DResult<String>>,
    sink: SplitSink<WebSocketStream<Upgraded>, Message>,
}

impl<Send> ApiChannelSender<Send>
where
    Send: Serialize + std::marker::Send + 'static,
{
    pub fn new<HFn, HFut>(
        converter: HFn,
        sink: SplitSink<WebSocketStream<Upgraded>, Message>,
    ) -> Self
    where
        HFn: Fn(Send) -> HFut + std::marker::Send + Sync + 'static,
        HFut: Future<Output = DResult<String>> + std::marker::Send + 'static,
    {
        let converter = Arc::new(converter);
        ApiChannelSender {
            converter: Box::new(move |message| {
                let converter = converter.clone();
                Box::pin(async move { converter(message).await })
            }),
            sink,
        }
    }

    pub async fn send(&mut self, message: Send) -> Result<(), ApiChannelSenderError> {
        let converter = &self.converter;

        let message_string = converter(message)
            .await
            .map_err(|e| ApiChannelSenderError::Convert(e.into()))?;
        self.sink
            .send(Message::Text(message_string))
            .await
            .map_err(|e| ApiChannelSenderError::Tungstenite(e))
    }

    pub async fn close(&mut self) -> Result<(), ApiChannelSenderError> {
        self.sink
            .send(Message::Close(None))
            .await
            .map_err(|e| ApiChannelSenderError::Tungstenite(e))
    }
}

pub enum ApiChannelReceiverError {
    Convert(DError),
    Tungstenite(Error),
    NoMessage,
    UnsupportedMessage,
    Closed,
}

pub struct ApiChannelReceiver<Receive>
where
    for<'de> Receive: Deserialize<'de> + std::marker::Send + 'static,
{
    converter: DFn<String, DResult<Receive>>,
    stream: SplitStream<WebSocketStream<Upgraded>>,
}

impl<Receive> ApiChannelReceiver<Receive>
where
    for<'de> Receive: Deserialize<'de> + std::marker::Send + 'static,
{
    pub fn new<HFn, HFut>(converter: HFn, stream: SplitStream<WebSocketStream<Upgraded>>) -> Self
    where
        HFn: Fn(String) -> HFut + std::marker::Send + Sync + 'static,
        HFut: Future<Output = DResult<Receive>> + std::marker::Send + 'static,
    {
        let converter = Arc::new(converter);
        ApiChannelReceiver {
            converter: Box::new(move |message| {
                let converter = converter.clone();
                Box::pin(async move { converter(message).await })
            }),
            stream,
        }
    }

    pub async fn next_message(&mut self) -> Result<Receive, ApiChannelReceiverError> {
        let converter = &self.converter;

        match self.stream.next().await {
            None => Err(ApiChannelReceiverError::NoMessage),
            Some(message_result) => match message_result {
                Ok(message) => match message {
                    Message::Text(test) => converter(test)
                        .await
                        .map_err(|e| ApiChannelReceiverError::Convert(e.into())),
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

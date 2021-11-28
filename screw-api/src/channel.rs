use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use hyper::http::Extensions;
use hyper::upgrade::Upgraded;
use screw_core::routing::Handler;
use screw_core::{DError, DResult};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::WebSocketStream;

pub trait ApiChannelExtensions {
    fn create(extensions: Extensions) -> Self;
}

pub struct ApiChannel<Extensions, Send, Receive>
where
    Extensions: ApiChannelExtensions,
    Send: Serialize + std::marker::Send + 'static,
    Receive: for<'de> Deserialize<'de> + std::marker::Send + 'static,
{
    extensions: Extensions,
    sender: ApiChannelSender<Send>,
    receiver: ApiChannelReceiver<Receive>,
}

impl<Extensions, Send, Receive> ApiChannel<Extensions, Send, Receive>
where
    Extensions: ApiChannelExtensions,
    Send: Serialize + std::marker::Send + 'static,
    Receive: for<'de> Deserialize<'de> + std::marker::Send + 'static,
{
    pub fn new(
        extensions: Extensions,
        sender: ApiChannelSender<Send>,
        receiver: ApiChannelReceiver<Receive>,
    ) -> Self {
        Self {
            extensions,
            sender,
            receiver,
        }
    }

    pub fn split(
        self,
    ) -> (
        Extensions,
        ApiChannelSender<Send>,
        ApiChannelReceiver<Receive>,
    ) {
        (self.extensions, self.sender, self.receiver)
    }

    pub fn extensions_ref(&self) -> &Extensions {
        &self.extensions
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
    converter: Handler<Send, DResult<String>>,
    sink: SplitSink<WebSocketStream<Upgraded>, Message>,
}

impl<Send> ApiChannelSender<Send>
where
    Send: Serialize + std::marker::Send + 'static,
{
    pub fn new(
        converter: Handler<Send, DResult<String>>,
        sink: SplitSink<WebSocketStream<Upgraded>, Message>,
    ) -> Self {
        ApiChannelSender { converter, sink }
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
    converter: Handler<String, DResult<Receive>>,
    stream: SplitStream<WebSocketStream<Upgraded>>,
}

impl<Receive> ApiChannelReceiver<Receive>
where
    for<'de> Receive: Deserialize<'de> + std::marker::Send + 'static,
{
    pub fn new(
        converter: Handler<String, DResult<Receive>>,
        stream: SplitStream<WebSocketStream<Upgraded>>,
    ) -> Self {
        ApiChannelReceiver { converter, stream }
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

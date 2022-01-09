use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use hyper::http::request::Parts;
use hyper::http::Extensions;
use hyper::upgrade::Upgraded;
use screw_components::dyn_fn::{AsDynFn, DFn};
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
    pub sender: ApiChannelSenderSecondPart<Send>,
    pub receiver: ApiChannelReceiverSecondPart<Receive>,
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

    pub fn and_convert_typed_message_fn<Send, HFn, HFut>(
        self,
        convert_typed_message_fn: HFn,
    ) -> ApiChannelSenderSecondPart<Send>
    where
        Send: Serialize + std::marker::Send + 'static,
        HFn: Fn(Send) -> HFut + std::marker::Send + Sync + 'static,
        HFut: Future<Output = DResult<String>> + std::marker::Send + 'static,
    {
        ApiChannelSenderSecondPart {
            sink: self.sink,
            convert_typed_message_fn: convert_typed_message_fn.to_dyn_fn(),
        }
    }
}

pub struct ApiChannelSenderSecondPart<Send>
where
    Send: Serialize + std::marker::Send + 'static,
{
    sink: SplitSink<WebSocketStream<Upgraded>, Message>,
    convert_typed_message_fn: DFn<Send, DResult<String>>,
}

impl<Send> ApiChannelSenderSecondPart<Send>
where
    Send: Serialize + std::marker::Send + 'static,
{
    pub async fn send(&mut self, typed_message: Send) -> Result<(), ApiChannelSenderError> {
        let convert_typed_message_fn = &self.convert_typed_message_fn;

        let generic_message = convert_typed_message_fn(typed_message)
            .await
            .map_err(ApiChannelSenderError::Convert)?;
        self.sink
            .send(Message::Text(generic_message))
            .await
            .map_err(ApiChannelSenderError::Tungstenite)?;
        Ok(())
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

    pub fn and_convert_generic_message_fn<Receive, HFn, HFut>(
        self,
        convert_generic_message_fn: HFn,
    ) -> ApiChannelReceiverSecondPart<Receive>
    where
        for<'de> Receive: Deserialize<'de> + std::marker::Send + 'static,
        HFn: Fn(String) -> HFut + std::marker::Send + Sync + 'static,
        HFut: Future<Output = DResult<Receive>> + std::marker::Send + 'static,
    {
        ApiChannelReceiverSecondPart {
            stream: self.stream,
            convert_generic_message_fn: convert_generic_message_fn.to_dyn_fn(),
        }
    }
}

pub struct ApiChannelReceiverSecondPart<Receive>
where
    for<'de> Receive: Deserialize<'de> + std::marker::Send + 'static,
{
    stream: SplitStream<WebSocketStream<Upgraded>>,
    convert_generic_message_fn: DFn<String, DResult<Receive>>,
}

impl<Receive> ApiChannelReceiverSecondPart<Receive>
where
    for<'de> Receive: Deserialize<'de> + std::marker::Send + 'static,
{
    pub async fn receive(&mut self) -> Result<Receive, ApiChannelReceiverError> {
        let convert_generic_message_fn = &self.convert_generic_message_fn;

        let message_type_result = self
            .stream
            .next()
            .await
            .ok_or(ApiChannelReceiverError::NoMessage)?;
        let message_type = message_type_result.map_err(ApiChannelReceiverError::Tungstenite)?;
        let generic_message = match message_type {
            Message::Text(generic_message) => Ok(generic_message),
            Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => {
                Err(ApiChannelReceiverError::UnsupportedMessage)
            }
            Message::Close(_) => Err(ApiChannelReceiverError::Closed),
        }?;
        let typed_message = convert_generic_message_fn(generic_message)
            .await
            .map_err(ApiChannelReceiverError::Convert)?;
        Ok(typed_message)
    }
}

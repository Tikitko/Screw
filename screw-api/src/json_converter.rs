use super::*;
use hyper::http::request::Parts;
use hyper::{header, Body, StatusCode};
use response::ApiResponseContentBase;
use screw_components::dyn_result::DResult;
use screw_core::routing::converter::{RequestConverter, ResponseConverter};
use screw_core::routing::router::RoutedRequest;
use screw_core::{Request, Response};
use serde::Deserialize;

#[derive(Clone, Copy, Debug)]
pub struct JsonApiRequestConverter;

#[async_trait]
impl<RqContent, Extensions> RequestConverter<request::ApiRequest<RqContent, Extensions>>
    for JsonApiRequestConverter
where
    RqContent: request::ApiRequestContent<Extensions> + Send + 'static,
    Extensions: Sync + Send + 'static,
{
    type Request = RoutedRequest<Request<Extensions>>;
    async fn convert_request(
        &self,
        request: Self::Request,
    ) -> request::ApiRequest<RqContent, Extensions> {
        async fn convert<Data>(parts: &Parts, body: Body) -> DResult<Data>
        where
            for<'de> Data: Deserialize<'de>,
        {
            let content_type = match parts.headers.get(header::CONTENT_TYPE) {
                Some(header_value) => Some(header_value.to_str()?),
                None => None,
            };
            match content_type {
                Some("application/json") => Ok(()),
                Some("") | None => Err(ApiRequestContentTypeError::Missed),
                Some(_) => Err(ApiRequestContentTypeError::Incorrect),
            }?;
            let json_bytes = hyper::body::to_bytes(body).await?;
            let data = serde_json::from_slice(&json_bytes)?;
            Ok(data)
        }

        let (http_parts, http_body) = request.origin.http.into_parts();
        let data_result = convert(&http_parts, http_body).await;

        let request_content = RqContent::create(request::ApiRequestOriginContent {
            path: request.path,
            query: request.query,
            http_parts,
            remote_addr: request.origin.remote_addr,
            extensions: request.origin.extensions,
            data_result,
        });

        request::ApiRequest {
            content: request_content,
            _p_e: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct JsonApiResponseConverter {
    pub pretty_printed: bool,
}

#[async_trait]
impl<RsContentSuccess, RsContentFailure>
    ResponseConverter<response::ApiResponse<RsContentSuccess, RsContentFailure>>
    for JsonApiResponseConverter
where
    RsContentSuccess: response::ApiResponseContentSuccess + Send + 'static,
    RsContentFailure: response::ApiResponseContentFailure + Send + 'static,
{
    type Response = Response;
    async fn convert_response(
        &self,
        api_response: response::ApiResponse<RsContentSuccess, RsContentFailure>,
    ) -> Self::Response {
        let http_response_result: DResult<hyper::Response<Body>> = (|| {
            let content = api_response.content;

            let status_code = content.status_code();
            let json_bytes = if self.pretty_printed {
                serde_json::to_vec_pretty(&content)
            } else {
                serde_json::to_vec(&content)
            }?;

            let response = hyper::Response::builder()
                .status(status_code)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json_bytes))?;

            Ok(response)
        })();

        let http_response = http_response_result.unwrap_or_else(|_| {
            hyper::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()
        });

        Response {
            http: http_response,
        }
    }
}

#[cfg(feature = "ws")]
pub mod ws {
    use super::*;
    use futures::{future, StreamExt};
    use hyper::upgrade::Upgraded;
    use screw_ws::WebSocketStreamConverter;
    use serde::Serialize;
    use tokio_tungstenite::WebSocketStream;

    #[derive(Clone, Copy, Debug)]
    pub struct JsonApiWebSocketConverter {
        pub pretty_printed: bool,
    }

    #[async_trait]
    impl<Send, Receive> WebSocketStreamConverter<channel::ApiChannel<Send, Receive>>
        for JsonApiWebSocketConverter
    where
        Send: Serialize + std::marker::Send + 'static,
        Receive: for<'de> Deserialize<'de> + std::marker::Send + 'static,
    {
        async fn convert_stream(
            &self,
            stream: WebSocketStream<Upgraded>,
        ) -> channel::ApiChannel<Send, Receive> {
            let (sink, stream) = stream.split();
            let pretty_printed = self.pretty_printed;

            let sender = channel::first::ApiChannelSender::with_sink(sink)
                .and_convert_typed_message_fn(move |typed_message| {
                    let generic_message_result = if pretty_printed {
                        serde_json::to_string_pretty(&typed_message)
                    } else {
                        serde_json::to_string(&typed_message)
                    };
                    future::ready(generic_message_result.map_err(|e| e.into()))
                });

            let receiver = channel::first::ApiChannelReceiver::with_stream(stream)
                .and_convert_generic_message_fn(|generic_message| {
                    let typed_message_result = serde_json::from_str(generic_message.as_str());
                    future::ready(typed_message_result.map_err(|e| e.into()))
                });

            channel::ApiChannel { sender, receiver }
        }
    }
}

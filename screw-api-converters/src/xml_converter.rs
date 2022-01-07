use super::ApiRequestContentTypeError;
use async_trait::async_trait;
use hyper::http::request::Parts;
use hyper::{body, header, Body, StatusCode};
use screw_api::{
    ApiRequest, ApiRequestContent, ApiRequestOriginContent, ApiResponse, ApiResponseContentBase,
    ApiResponseContentFailure, ApiResponseContentSuccess,
};
use screw_components::dyn_result::DResult;
use screw_core::routing::{RequestResponseConverter, RequestResponseConverterBase};
use screw_core::{Request, Response};
use serde::Deserialize;

#[derive(Clone, Copy, Debug)]
pub struct XmlApiConverter;

impl RequestResponseConverterBase for XmlApiConverter {}

#[async_trait]
impl<RqContent, RsContentSuccess, RsContentFailure>
    RequestResponseConverter<ApiRequest<RqContent>, ApiResponse<RsContentSuccess, RsContentFailure>>
    for XmlApiConverter
where
    RqContent: ApiRequestContent + Send + 'static,
    RsContentSuccess: ApiResponseContentSuccess + Send + 'static,
    RsContentFailure: ApiResponseContentFailure + Send + 'static,
{
    type Request = Request;
    type Response = Response;

    async fn convert_request(&self, request: Self::Request) -> ApiRequest<RqContent> {
        async fn convert<Data>(parts: &Parts, body: Body) -> DResult<Data>
        where
            for<'de> Data: Deserialize<'de>,
        {
            let content_type = match parts.headers.get(header::CONTENT_TYPE) {
                Some(header_value) => Some(header_value.to_str()?),
                None => None,
            };
            match content_type {
                Some("application/xml") => Ok(()),
                Some("") | None => Err(ApiRequestContentTypeError::Missed),
                Some(_) => Err(ApiRequestContentTypeError::Incorrect),
            }?;
            let bytes = body::to_bytes(body).await?;
            let xml_string = String::from_utf8(bytes.to_vec())?;
            let data = quick_xml::de::from_str(xml_string.as_str())?;
            Ok(data)
        }

        let (http_parts, http_body) = request.http.into_parts();
        let data_result = convert(&http_parts, http_body).await;

        let request_content = RqContent::create(ApiRequestOriginContent {
            http_parts,
            remote_addr: request.remote_addr,
            extensions: request.extensions,
            data_result,
        });

        ApiRequest {
            content: request_content,
        }
    }
    async fn convert_response(
        &self,
        api_response: ApiResponse<RsContentSuccess, RsContentFailure>,
    ) -> Self::Response {
        let http_response_result: DResult<hyper::Response<Body>> = (|| {
            let content = api_response.content;

            let status_code = content.status_code();
            let xml_string = quick_xml::se::to_string(&content)?;

            let response = hyper::Response::builder()
                .status(status_code)
                .header(header::CONTENT_TYPE, "application/xml")
                .body(Body::from(xml_string))?;

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
    use screw_api::{ApiChannel, ApiChannelReceiver, ApiChannelSender};
    use screw_ws::{WebSocketStreamConverter, WebSocketStreamConverterBase};
    use serde::Serialize;
    use tokio_tungstenite::WebSocketStream;

    impl WebSocketStreamConverterBase for XmlApiConverter {}

    #[async_trait]
    impl<Send, Receive> WebSocketStreamConverter<ApiChannel<Send, Receive>> for XmlApiConverter
    where
        Send: Serialize + std::marker::Send + 'static,
        Receive: for<'de> Deserialize<'de> + std::marker::Send + 'static,
    {
        async fn convert_stream(
            &self,
            stream: WebSocketStream<Upgraded>,
        ) -> ApiChannel<Send, Receive> {
            let (sink, stream) = stream.split();

            let sender = ApiChannelSender::with_sink(sink).and_convert_typed_message_fn(
                move |typed_message| {
                    let generic_message_result = quick_xml::se::to_string(&typed_message);
                    future::ready(generic_message_result.map_err(|e| e.into()))
                },
            );

            let receiver = ApiChannelReceiver::with_stream(stream).and_convert_generic_message_fn(
                |generic_message| {
                    let typed_message_result = quick_xml::de::from_str(generic_message.as_str());
                    future::ready(typed_message_result.map_err(|e| e.into()))
                },
            );

            ApiChannel { sender, receiver }
        }
    }
}

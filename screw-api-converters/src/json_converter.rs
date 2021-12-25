use async_trait::async_trait;
use derive_error::Error;
use futures::StreamExt;
use hyper::header::ToStrError;
use hyper::http::request::Parts;
use hyper::upgrade::Upgraded;
use hyper::{header, Body, StatusCode};
use screw_api::{
    ApiChannel, ApiChannelReceiver, ApiChannelSender, ApiRequest, ApiRequestContent,
    ApiRequestOriginContent, ApiResponse, ApiResponseContentBase, ApiResponseContentFailure,
    ApiResponseContentSuccess,
};
use screw_core::routing::router::RequestResponseConverter;
use screw_core::routing::{Request, Response};
use screw_core::DResult;
use serde::{Deserialize, Serialize};

pub struct JsonApiConverter {
    pub pretty_printed: bool,
}

#[derive(Error, Debug)]
pub enum JsonApiRequestConvertError {
    ContentTypeMissed,
    ContentTypeIncorrect,
    ToStr(ToStrError),
    Hyper(hyper::Error),
    SerdeJson(serde_json::Error),
}

#[async_trait]
impl<RqContent, RsContentSuccess, RsContentFailure>
    RequestResponseConverter<ApiRequest<RqContent>, ApiResponse<RsContentSuccess, RsContentFailure>>
    for JsonApiConverter
where
    RqContent: ApiRequestContent + Send + 'static,
    RsContentSuccess: ApiResponseContentSuccess + Send + 'static,
    RsContentFailure: ApiResponseContentFailure + Send + 'static,
{
    async fn convert_request(&self, request: Request) -> ApiRequest<RqContent> {
        async fn convert<Data>(
            parts: &Parts,
            body: Body,
        ) -> Result<Data, JsonApiRequestConvertError>
        where
            for<'de> Data: Deserialize<'de>,
        {
            let content_type = match parts.headers.get(header::CONTENT_TYPE) {
                Some(content_type) => Some(
                    content_type
                        .to_str()
                        .map_err(|e| JsonApiRequestConvertError::ToStr(e))?,
                ),
                None => None,
            };
            match content_type {
                Some("application/json") => {
                    let bytes = hyper::body::to_bytes(body)
                        .await
                        .map_err(|e| JsonApiRequestConvertError::Hyper(e))?;
                    let data = serde_json::from_slice(&bytes)
                        .map_err(|e| JsonApiRequestConvertError::SerdeJson(e))?;
                    Ok(data)
                }
                Some("") | None => Err(JsonApiRequestConvertError::ContentTypeMissed),
                Some(_) => Err(JsonApiRequestConvertError::ContentTypeIncorrect),
            }
        }

        let (http_parts, http_body) = request.http.into_parts();
        let data_result = convert(&http_parts, http_body).await;

        let request_content = RqContent::create(ApiRequestOriginContent {
            http_parts,
            remote_addr: request.remote_addr,
            extensions: request.extensions,
            data_result: data_result.map_err(|e| e.into()),
        });

        ApiRequest {
            content: request_content,
        }
    }
    async fn convert_response(
        &self,
        api_response: ApiResponse<RsContentSuccess, RsContentFailure>,
    ) -> Response {
        let http_response_result: DResult<hyper::Response<Body>> = (|| {
            let content = api_response.content;

            let status_code = content.status_code();
            let json_bytes_vec = if self.pretty_printed {
                serde_json::to_vec_pretty(&content)?
            } else {
                serde_json::to_vec(&content)?
            };

            let response = hyper::Response::builder()
                .status(status_code)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json_bytes_vec))?;

            Ok(response)
        })();

        let http_response = match http_response_result {
            Ok(http_response) => http_response,
            Err(_) => hyper::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap(),
        };
        Response {
            http: http_response,
        }
    }
}

#[cfg(feature = "ws")]
#[async_trait]
impl<Send, Receive> screw_ws::WebSocketStreamConverter<ApiChannel<Send, Receive>>
    for JsonApiConverter
where
    Send: Serialize + std::marker::Send + 'static,
    Receive: for<'de> Deserialize<'de> + std::marker::Send + 'static,
{
    async fn convert_stream(
        &self,
        stream: tokio_tungstenite::WebSocketStream<Upgraded>,
    ) -> ApiChannel<Send, Receive> {
        let (sink, stream) = stream.split();
        let pretty_printed = self.pretty_printed;

        let sender = ApiChannelSender::new(
            Box::new(move |message| {
                Box::pin(async move {
                    let serde_result = if pretty_printed {
                        serde_json::to_string_pretty(&message)
                    } else {
                        serde_json::to_string(&message)
                    };
                    serde_result.map_err(|e| e.into())
                })
            }),
            sink,
        );

        let receiver = ApiChannelReceiver::new(
            Box::new(|message| {
                Box::pin(async move {
                    let serde_result = serde_json::from_str(message.as_str());
                    serde_result.map_err(|e| e.into())
                })
            }),
            stream,
        );

        ApiChannel::new(sender, receiver)
    }
}

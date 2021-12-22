use async_trait::async_trait;
use derive_error::Error;
use hyper::header::ToStrError;
use hyper::http::request::Parts;
use hyper::{header, Body, StatusCode};
use hyper::{Error as HyperError, Response as HyperResponse};
use screw_api::{
    ApiRequest, ApiRequestContent, ApiResponse, ApiResponseContentBase, ApiResponseContentFailure,
    ApiResponseContentSuccess,
};
use screw_core::routing::router::RequestResponseConverter;
use screw_core::routing::{Request, Response};
use screw_core::DResult;
use serde::Deserialize;

pub struct JsonApiConverter {
    pub pretty_printed: bool,
}

#[derive(Error, Debug)]
pub enum JsonApiRequestConvertError {
    ContentTypeMissed,
    ContentTypeIncorrect,
    ToStr(ToStrError),
    Hyper(HyperError),
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

        let (parts, body) = request.http.into_parts();
        let data_result = convert(&parts, body).await;

        ApiRequest::new(RqContent::create(
            parts,
            request.remote_addr,
            request.data_map,
            data_result.map_err(|e| e.into()),
        ))
    }
    async fn convert_response(
        &self,
        request: ApiResponse<RsContentSuccess, RsContentFailure>,
    ) -> Response {
        let response_result: DResult<hyper::Response<Body>> = (|| {
            let content = request.content();

            let status_code = content.status_code();
            let json_bytes_vec = if self.pretty_printed {
                serde_json::to_vec_pretty(&content)?
            } else {
                serde_json::to_vec(&content)?
            };

            let response = HyperResponse::builder()
                .status(status_code)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json_bytes_vec))?;

            Ok(response)
        })();

        let response = match response_result {
            Ok(response) => response,
            Err(_) => HyperResponse::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap(),
        };
        Response { http: response }
    }
}

use hyper::http::request::Parts;
use hyper::http::Extensions;
use screw_core::DResult;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct ApiRequestOriginContent<Data>
where
    Data: for<'de> Deserialize<'de>,
{
    pub http_parts: Parts,
    pub remote_addr: SocketAddr,
    pub extensions: Arc<Extensions>,
    pub data_result: DResult<Data>,
}

pub trait ApiRequestContent {
    type Data: for<'de> Deserialize<'de>;
    fn create(origin_content: ApiRequestOriginContent<Self::Data>) -> Self;
}

pub struct ApiRequest<Content>
where
    Content: ApiRequestContent,
{
    pub content: Content,
}

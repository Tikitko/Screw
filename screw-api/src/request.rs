use hyper::http::request::Parts;
use screw_core::maps::SharedDataMap;
use screw_core::DResult;
use serde::Deserialize;
use std::net::SocketAddr;

pub trait ApiRequestContent {
    type Data: for<'de> Deserialize<'de>;
    fn create(
        parts: Parts,
        remote_addr: SocketAddr,
        data_map: SharedDataMap,
        data_result: DResult<Self::Data>,
    ) -> Self;
}

pub struct ApiRequest<Content>
where
    Content: ApiRequestContent,
{
    content: Content,
}

impl<Content> ApiRequest<Content>
where
    Content: ApiRequestContent,
{
    pub fn new(content: Content) -> Self {
        Self { content }
    }

    pub fn content(self) -> Content {
        self.content
    }

    pub fn content_ref(&self) -> &Content {
        &self.content
    }
}

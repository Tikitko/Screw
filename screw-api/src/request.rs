use hyper::http::request::Parts;
use screw_components::dyn_result::DResult;
use screw_core::routing::actix::Path;
use serde::Deserialize;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct ApiRequestOriginContent<Data, Extensions>
where
    Data: for<'de> Deserialize<'de>,
{
    pub path: Path<String>,
    pub query: HashMap<String, String>,
    pub http_parts: Parts,
    pub remote_addr: SocketAddr,
    pub extensions: Arc<Extensions>,
    pub data_result: DResult<Data>,
}

pub trait ApiRequestContent<Extensions> {
    type Data: for<'de> Deserialize<'de>;
    fn create(origin_content: ApiRequestOriginContent<Self::Data, Extensions>) -> Self;
}

impl<Extensions> ApiRequestContent<Extensions> for () {
    type Data = ();
    fn create(_origin_content: ApiRequestOriginContent<Self::Data, Extensions>) -> Self {}
}

pub struct ApiRequest<Content, Extensions>
where
    Content: ApiRequestContent<Extensions>,
{
    pub content: Content,
    pub _p_e: PhantomData<Extensions>,
}

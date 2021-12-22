use crate::ApiRequestContent;
use hyper::http::request::Parts;
use screw_core::maps::SharedDataMap;
use screw_core::DResult;
use std::net::SocketAddr;

pub struct EmptyApiRequestContent;

impl ApiRequestContent for EmptyApiRequestContent {
    type Data = ();

    fn create(
        _parts: Parts,
        _remote_addr: SocketAddr,
        _data_map: SharedDataMap,
        _data_result: DResult<Self::Data>,
    ) -> Self {
        EmptyApiRequestContent
    }
}

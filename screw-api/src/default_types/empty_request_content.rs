use crate::ApiRequestContent;
use hyper::http::request::Parts;
use screw_core::DResult;

pub struct EmptyApiRequestContent;

impl ApiRequestContent for EmptyApiRequestContent {
    type Data = ();

    fn create(_parts: Parts, _data_result: DResult<Self::Data>) -> Self {
        EmptyApiRequestContent
    }
}

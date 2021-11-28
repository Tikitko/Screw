use crate::{ApiResponseContentBase, ApiResponseContentFailure, ApiResponseContentSuccess};
use hyper::StatusCode;

pub enum UnreachableApiResponseContent {}

impl ApiResponseContentBase for UnreachableApiResponseContent {
    fn status_code(&self) -> &'static StatusCode {
        unreachable!()
    }
}

impl ApiResponseContentSuccess for UnreachableApiResponseContent {
    type Data = ();

    fn identifier(&self) -> &'static str {
        unreachable!()
    }

    fn description(&self) -> Option<String> {
        unreachable!()
    }

    fn data(&self) -> &Self::Data {
        unreachable!()
    }
}

impl ApiResponseContentFailure for UnreachableApiResponseContent {
    fn identifier(&self) -> &'static str {
        unreachable!()
    }

    fn reason(&self) -> Option<String> {
        unreachable!()
    }
}

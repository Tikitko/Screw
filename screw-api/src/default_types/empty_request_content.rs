use crate::{ApiRequestContent, ApiRequestOriginContent};

pub struct EmptyApiRequestContent;

impl ApiRequestContent for EmptyApiRequestContent {
    type Data = ();

    fn create(_origin_content: ApiRequestOriginContent<Self::Data>) -> Self {
        Self
    }
}

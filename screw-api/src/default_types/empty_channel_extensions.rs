use crate::ApiChannelExtensions;
use hyper::http::Extensions;

pub struct EmptyApiChannelExtensions;

impl ApiChannelExtensions for EmptyApiChannelExtensions {
    fn create(_extensions: Extensions) -> Self {
        EmptyApiChannelExtensions
    }
}

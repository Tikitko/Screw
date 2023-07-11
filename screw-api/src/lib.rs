#[cfg(feature = "ws")]
pub mod channel;
pub mod request;
pub mod response;

#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "xml")]
pub mod xml;
#[cfg(any(feature = "json", feature = "xml"))]
#[derive(derive_error::Error, Debug)]
enum ApiRequestContentTypeError {
    Missed,
    Incorrect,
}

#[cfg(any(feature = "json", feature = "xml"))]
#[macro_use]
extern crate async_trait;

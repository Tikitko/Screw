#[cfg(feature = "ws")]
pub mod channel;
pub mod request;
pub mod response;

#[cfg(feature = "json_converter")]
pub mod json_converter;
#[cfg(feature = "xml_converter")]
pub mod xml_converter;

#[cfg(any(feature = "json_converter", feature = "xml_converter"))]
#[derive(derive_error::Error, Debug)]
enum ApiRequestContentTypeError {
    Missed,
    Incorrect,
}

#[cfg(any(feature = "json_converter", feature = "xml_converter"))]
#[macro_use]
extern crate async_trait;

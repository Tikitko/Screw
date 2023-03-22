#[cfg(feature = "json_converter")]
mod json_converter;
#[cfg(feature = "xml_converter")]
mod xml_converter;

#[cfg(feature = "json_converter")]
pub use json_converter::*;
#[cfg(feature = "xml_converter")]
pub use xml_converter::*;

#[cfg(any(feature = "json_converter", feature = "xml_converter"))]
#[derive(derive_error::Error, Debug)]
enum ApiRequestContentTypeError {
    Missed,
    Incorrect,
}

#[cfg(any(feature = "json_converter", feature = "xml_converter"))]
#[macro_use]
extern crate async_trait;

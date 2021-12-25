mod converter;
mod request;
mod response;
mod stream_converter;

pub use converter::*;
pub use request::*;
pub use response::*;
pub use stream_converter::*;

pub fn is_upgrade_request(request: &hyper::Request<hyper::Body>) -> bool {
    is_connection_header_upgrade(request) && is_upgrade_header_web_socket(request)
}

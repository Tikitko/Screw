use super::routing::RequestResponseConverter;
use async_trait::async_trait;
use hyper::{Body, Request};

pub struct ThroughConverter;

#[async_trait]
impl<Rq, Rs> RequestResponseConverter<Rq, Rs> for ThroughConverter
where
    Rq: AsRef<Request<Body>> + Send + 'static,
    Rs: Send + 'static,
{
    type Request = Rq;
    type Response = Rs;
    async fn convert_request(&self, request: Self::Request) -> Rq {
        request
    }
    async fn convert_response(&self, response: Rs) -> Self::Response {
        response
    }
}

use async_trait::async_trait;

#[async_trait]
pub trait RequestResponseConverter<Rq, Rs> {
    type Request;
    type Response;
    async fn convert_request(&self, request: Self::Request) -> Rq;
    async fn convert_response(&self, response: Rs) -> Self::Response;
}

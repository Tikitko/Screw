#[async_trait]
pub trait RequestResponseConverter<Rq, Rs> {
    type Request;
    type Response;
    async fn convert_request(&self, request: Self::Request) -> Rq;
    async fn convert_response(&self, response: Rs) -> Self::Response;
}

#[async_trait]
impl<Rq, Rs> RequestResponseConverter<Rq, Rs> for ()
where
    Rq: Send + 'static,
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

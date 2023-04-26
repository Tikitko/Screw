use std::future::Future;

#[async_trait]
pub trait RequestConverter<Rq> {
    type Request;
    async fn convert_request(&self, request: Self::Request) -> Rq;
}

#[async_trait]
pub trait ResponseConverter<Rs> {
    type Response;
    async fn convert_response(&self, response: Rs) -> Self::Response;
}

#[async_trait]
impl<Rq> RequestConverter<Rq> for ()
where
    Rq: Send + 'static,
{
    type Request = Rq;
    async fn convert_request(&self, request: Self::Request) -> Rq {
        request
    }
}

#[async_trait]
impl<Rs> ResponseConverter<Rs> for ()
where
    Rs: Send + 'static,
{
    type Response = Rs;
    async fn convert_response(&self, response: Rs) -> Self::Response {
        response
    }
}

pub struct MiddlewareConverter<C, HFn> {
    pub converter: C,
    pub handler: HFn,
}

#[async_trait]
impl<Rq, C, HFn, HFut> RequestConverter<Rq> for MiddlewareConverter<C, HFn>
where
    Rq: Send + 'static,
    C: RequestConverter<Rq> + Send + Sync + 'static,
    C::Request: Send + 'static,
    HFn: Fn(C::Request) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = C::Request> + Send + 'static,
{
    type Request = C::Request;
    async fn convert_request(&self, request: Self::Request) -> Rq {
        self.converter
            .convert_request((self.handler)(request).await)
            .await
    }
}

#[async_trait]
impl<Rs, C, HFn, HFut> ResponseConverter<Rs> for MiddlewareConverter<C, HFn>
where
    Rs: Send + 'static,
    C: ResponseConverter<Rs> + Send + Sync + 'static,
    C::Response: Send + 'static,
    HFn: Fn(C::Response) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = C::Response> + Send + 'static,
{
    type Response = C::Response;
    async fn convert_response(&self, response: Rs) -> Self::Response {
        (self.handler)(self.converter.convert_response(response).await).await
    }
}

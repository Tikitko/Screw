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

#[async_trait]
impl<Rq, HFn, HFut> RequestConverter<Rq> for HFn
where
    Rq: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rq> + Send + 'static,
{
    type Request = Rq;
    async fn convert_request(&self, request: Self::Request) -> Rq {
        self(request).await
    }
}

#[async_trait]
impl<Rs, HFn, HFut> ResponseConverter<Rs> for HFn
where
    Rs: Send + 'static,
    HFn: Fn(Rs) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
{
    type Response = Rs;
    async fn convert_response(&self, response: Rs) -> Self::Response {
        self(response).await
    }
}

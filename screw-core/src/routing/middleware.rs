use screw_components::dyn_fn::DFnOnce;
use std::future::Future;

#[async_trait]
pub trait Middleware<Rq, Rs> {
    type Request;
    type Response;
    async fn respond(&self, request: Self::Request, next: DFnOnce<Rq, Rs>) -> Self::Response;
}

#[async_trait]
impl<Rq, Rs> Middleware<Rq, Rs> for ()
where
    Rq: Send + 'static,
    Rs: Send + 'static,
{
    type Request = Rq;
    type Response = Rs;
    async fn respond(&self, request: Self::Request, next: DFnOnce<Rq, Rs>) -> Self::Response {
        next(request).await
    }
}

#[async_trait]
impl<Rq, Rs, HFn, HFut> Middleware<Rq, Rs> for HFn
where
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq, DFnOnce<Rq, Rs>) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
{
    type Request = Rq;
    type Response = Rs;
    async fn respond(&self, request: Self::Request, next: DFnOnce<Rq, Rs>) -> Self::Response {
        self(request, next).await
    }
}

use async_trait::async_trait;
use hyper::Method;

#[async_trait]
pub trait Route {
    type Rq: Send + 'static;
    type Rs: Send + 'static;
    fn method() -> &'static Method;
    fn path() -> &'static str;
    async fn handler(request: Self::Rq) -> Self::Rs;
}

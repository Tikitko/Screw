use super::HttpConverter;
use super::HttpHandler;
use std::future::Future;
use std::sync::Arc;

pub(in crate::routing::router) fn convert_http_handler<C, Rq, Rs, HFn, HFut>(
    converter: Arc<C>,
    http_handler: HFn,
) -> HttpHandler
where
    C: HttpConverter<Rq, Rs> + Send + Sync + 'static,
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
{
    let handler = Arc::new(http_handler);
    Box::new(move |request| {
        let handler = handler.clone();
        let converter = converter.clone();
        Box::pin(async move {
            let handler_request = converter.convert_request(request).await;
            let handler_future = handler(handler_request);
            let handler_response = handler_future.await;
            let response = converter.convert_response(handler_response).await;
            response
        })
    })
}

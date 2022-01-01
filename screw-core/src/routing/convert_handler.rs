use super::RequestResponseConverter;
use screw_components::dyn_fn::DFn;
use std::future::Future;
use std::sync::Arc;

pub(super) fn convert_generic_handler<ORq, ORs, HFn, HFut>(handler: HFn) -> DFn<ORq, ORs>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    HFn: Fn(ORq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = ORs> + Send + 'static,
{
    let handler = Arc::new(handler);
    Box::new(move |request| {
        let handler = handler.clone();
        Box::pin(async move {
            let response = handler(request).await;
            response
        })
    })
}

pub(super) fn convert_typed_handler<ORq, ORs, C, Rq, Rs, HFn, HFut>(
    converter: Arc<C>,
    handler: HFn,
) -> DFn<ORq, ORs>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
    C: RequestResponseConverter<Rq, Rs, Request = ORq, Response = ORs> + Send + Sync + 'static,
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
{
    let handler = Arc::new(handler);
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

use super::WebSocketConverter;
use super::WebSocketHandler;
use std::future::Future;
use std::sync::Arc;

pub(in crate::routing::router) fn convert_web_socket_handler<C, SRq, HFn, HFut>(
    converter: Arc<C>,
    web_socket_handler: HFn,
) -> WebSocketHandler
where
    C: WebSocketConverter<SRq> + Send + Sync + 'static,
    SRq: Send + 'static,
    HFn: Fn(SRq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = ()> + Send + 'static,
{
    let handler = Arc::new(web_socket_handler);
    Box::new(move |streamable_request| {
        let handler = handler.clone();
        let converter = converter.clone();
        Box::pin(async move {
            let handler_streamable_request = converter
                .convert_streamable_request(streamable_request)
                .await;
            let handler_future = handler(handler_streamable_request);
            handler_future.await;
        })
    })
}

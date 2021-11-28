use super::http::{HttpHandler, Request, Response};
use super::web_socket::StreamableRequest;
use super::HandlersContainer;
use crate::protocols::web_socket as web_socket_protocol;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

pub struct Router {
    pub(super) handlers_container: HandlersContainer,
    pub(super) not_found_http_handler: HttpHandler,
    pub(super) web_socket_config: Option<WebSocketConfig>,
}

impl Router {
    pub(in crate::routing) async fn process(&self, request: Request) -> Response {
        let method = request.method().clone();
        let path = {
            let mut path = request.uri().path().to_owned();
            if path.ends_with('/') {
                path = (&path[..path.len() - 1]).to_string();
            }
            path
        };

        let response = if let (true, Some(web_socket_handler)) = (
            web_socket_protocol::is_upgrade_request(&request),
            self.handlers_container.web_socket.get(path.as_str()),
        ) {
            let web_socket_handler = web_socket_handler.clone();

            web_socket_protocol::upgrade(
                request,
                self.web_socket_config,
                |upgrade_result| async move {
                    if let Ok((stream, extensions)) = upgrade_result {
                        let streamable_request = StreamableRequest { stream, extensions };
                        web_socket_handler(streamable_request).await;
                    }
                },
            )
        } else {
            let http_handler = match self.handlers_container.http.get(&(method, path)) {
                Some(http_handler) => http_handler,
                None => &self.not_found_http_handler,
            };

            http_handler(request).await
        };

        response
    }
}

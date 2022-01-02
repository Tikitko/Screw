use super::{WebSocketContent, WebSocketOriginContent};

struct EmptyWebSocketContent;

impl WebSocketContent for EmptyWebSocketContent {
    fn create(_origin_content: WebSocketOriginContent) -> Self {
        Self
    }
}

pub mod first {
    use super::super::*;
    use screw_components::dyn_fn::{AsDynFn, DFn};
    use std::collections::HashMap;
    use std::future::Future;

    pub struct Router<ORq, ORs>
    where
        ORq: Send + 'static,
        ORs: Send + 'static,
    {
        fallback_handler: DFn<ORq, ORs>,
    }

    impl<ORq, ORs> Router<ORq, ORs>
    where
        ORq: Send + 'static,
        ORs: Send + 'static,
    {
        pub fn with_fallback_handler<HFn, HFut>(fallback_handler: HFn) -> Self
        where
            HFn: Fn(ORq) -> HFut + Send + Sync + 'static,
            HFut: Future<Output = ORs> + Send + 'static,
        {
            Router {
                fallback_handler: fallback_handler.to_dyn_fn(),
            }
        }

        pub fn and_routes<F>(self, handler: F) -> router::second::Router<ORq, ORs>
        where
            F: FnOnce(routes::Routes<ORq, ORs>) -> routes::Routes<ORq, ORs>,
        {
            let routes::Routes { handlers, .. } = handler(routes::Routes {
                scope_path: "".to_owned(),
                handlers: HashMap::default(),
            });
            router::second::Router {
                handlers,
                fallback_handler: self.fallback_handler,
            }
        }
    }
}

pub mod second {
    use hyper::{Body, Method, Request};
    use screw_components::dyn_fn::DFn;
    use std::collections::HashMap;

    pub struct Router<ORq, ORs>
    where
        ORq: Send + 'static,
        ORs: Send + 'static,
    {
        pub(super) handlers: HashMap<(&'static Method, String), DFn<ORq, ORs>>,
        pub(super) fallback_handler: DFn<ORq, ORs>,
    }

    impl<ORq, ORs> Router<ORq, ORs>
    where
        ORq: AsRef<Request<Body>> + Send + 'static,
        ORs: Send + 'static,
    {
        pub async fn process(&self, request: ORq) -> ORs {
            let http_request_ref = request.as_ref();

            let method = http_request_ref.method().to_owned();
            let path = http_request_ref.uri().path().to_owned();

            let handler = self
                .handlers
                .get(&(&method, path))
                .unwrap_or(&self.fallback_handler);

            let response = handler(request).await;

            response
        }
    }
}

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
        fallback_handler: DFn<request::DirectedRequest<ORq>, ORs>,
    }

    impl<ORq, ORs> Router<ORq, ORs>
    where
        ORq: Send + 'static,
        ORs: Send + 'static,
    {
        pub fn with_fallback_handler<HFn, HFut>(fallback_handler: HFn) -> Self
        where
            HFn: Fn(request::DirectedRequest<ORq>) -> HFut + Send + Sync + 'static,
            HFut: Future<Output = ORs> + Send + 'static,
        {
            Router {
                fallback_handler: fallback_handler.to_dyn_fn(),
            }
        }

        pub fn and_routes<F>(self, handler: F) -> router::second::Router<ORq, ORs>
        where
            F: FnOnce(
                routes::Routes<request::DirectedRequest<ORq>, ORs>,
            ) -> routes::Routes<request::DirectedRequest<ORq>, ORs>,
        {
            let routes::Routes { handlers, .. } = handler(routes::Routes {
                scope_path: "".to_owned(),
                handlers: HashMap::default(),
            });
            router::second::Router {
                inner: {
                    let mut inner_router = actix_router::Router::build();
                    for (path, handler) in handlers {
                        inner_router.path(path, handler);
                    }
                    inner_router.finish()
                },
                fallback_handler: self.fallback_handler,
            }
        }
    }
}

pub mod second {
    use super::super::*;
    use hyper::{Body, Method, Request};
    use screw_components::dyn_fn::DFn;
    use std::collections::HashMap;

    pub struct Router<ORq, ORs>
    where
        ORq: Send + 'static,
        ORs: Send + 'static,
    {
        pub(super) inner:
            actix_router::Router<HashMap<&'static Method, DFn<request::DirectedRequest<ORq>, ORs>>>,
        pub(super) fallback_handler: DFn<request::DirectedRequest<ORq>, ORs>,
    }

    impl<ORq, ORs> Router<ORq, ORs>
    where
        ORq: AsRef<Request<Body>> + Send + 'static,
        ORs: Send + 'static,
    {
        pub async fn process(&self, request: ORq) -> ORs {
            let http_request_ref = request.as_ref();

            let method = http_request_ref.method();
            let mut path = actix_router::Path::new(http_request_ref.uri().path().to_owned());
            let query = http_request_ref
                .uri()
                .query()
                .map(|v| {
                    url::form_urlencoded::parse(v.as_bytes())
                        .into_owned()
                        .collect()
                })
                .unwrap_or_else(HashMap::new);

            let handler = self
                .inner
                .recognize(&mut path)
                .map(|(h, _)| h.get(method))
                .flatten()
                .unwrap_or(&self.fallback_handler);

            let request = request::DirectedRequest {
                path,
                query,
                origin: request,
            };
            let response = handler(request).await;
            response
        }
    }
}

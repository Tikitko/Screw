use hyper::{Body, Method, Request};
use screw_components::dyn_fn::DFn;
use std::collections::HashMap;

pub struct Router<ORq, ORs>
where
    ORq: AsRef<Request<Body>>,
{
    pub(super) handlers: HashMap<(Method, String), DFn<ORq, ORs>>,
    pub(super) fallback_handler: DFn<ORq, ORs>,
}

impl<ORq, ORs> Router<ORq, ORs>
where
    ORq: AsRef<Request<Body>>,
{
    pub async fn process(&self, request: ORq) -> ORs {
        let http_request_ref = request.as_ref();

        let method = http_request_ref.method().clone();
        let path = http_request_ref.uri().clone().path().to_string();

        let clean_path = format!(
            "/{}",
            path.split('/')
                .filter(|seg| !seg.is_empty())
                .collect::<Vec<&str>>()
                .join("/")
        );

        let handler = self
            .handlers
            .get(&(method, clean_path))
            .unwrap_or(&self.fallback_handler);

        let response = handler(request).await;

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Body, Response, StatusCode};

    struct Rq(Request<Body>);
    impl Rq {
        fn with_path(path: &str) -> Rq {
            let http_request = Request::get(path).body(Body::empty()).unwrap();
            Rq(http_request)
        }
    }
    impl AsRef<Request<Body>> for Rq {
        fn as_ref(&self) -> &Request<Body> {
            &self.0
        }
    }

    struct Rs(Response<Body>);
    impl Rs {
        fn with_status(status: StatusCode) -> Rs {
            let http_response = Response::builder()
                .status(status)
                .body(Body::empty())
                .unwrap();
            Rs(http_response)
        }
        fn status(&self) -> StatusCode {
            self.0.status()
        }
    }

    fn create_handler_with_status(status: StatusCode) -> DFn<Rq, Rs> {
        Box::new(move |_| Box::pin(async move { Rs::with_status(status) }))
    }

    #[tokio::test]
    async fn test_router_process() {
        let router = Router {
            handlers: {
                let mut handlers = HashMap::new();
                handlers.insert(
                    (Method::GET, "/".to_string()),
                    create_handler_with_status(StatusCode::GONE),
                );
                handlers.insert(
                    (Method::GET, "/r1".to_string()),
                    create_handler_with_status(StatusCode::ACCEPTED),
                );
                handlers.insert(
                    (Method::GET, "/r2/t1".to_string()),
                    create_handler_with_status(StatusCode::OK),
                );
                handlers
            },
            fallback_handler: create_handler_with_status(StatusCode::INTERNAL_SERVER_ERROR),
        };

        assert_eq!(
            router.process(Rq::with_path("/")).await.status(),
            StatusCode::GONE
        );
        assert_eq!(
            router.process(Rq::with_path("///")).await.status(),
            StatusCode::GONE
        );
        assert_eq!(
            router.process(Rq::with_path("/r1")).await.status(),
            StatusCode::ACCEPTED
        );
        assert_eq!(
            router.process(Rq::with_path("/r1/")).await.status(),
            StatusCode::ACCEPTED
        );
        assert_eq!(
            router.process(Rq::with_path("/r1//")).await.status(),
            StatusCode::ACCEPTED
        );
        assert_eq!(
            router.process(Rq::with_path("/r2/t1")).await.status(),
            StatusCode::OK
        );
        assert_eq!(
            router.process(Rq::with_path("/r2/t1/")).await.status(),
            StatusCode::OK
        );
        assert_eq!(
            router.process(Rq::with_path("/some")).await.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            router.process(Rq::with_path("/some//")).await.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
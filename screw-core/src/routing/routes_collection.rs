use hyper::{Body, Method, Request};
use screw_components::dyn_fn::DFn;
use std::collections::HashMap;

pub struct RoutesCollection<ORq, ORs>
where
    ORq: AsRef<Request<Body>> + Send + 'static,
    ORs: Send + 'static,
{
    pub(super) handlers: HashMap<(Method, String), DFn<ORq, ORs>>,
}

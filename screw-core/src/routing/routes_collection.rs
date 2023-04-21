use hyper::Method;
use screw_components::dyn_fn::DFn;
use std::collections::HashMap;

pub struct RoutesCollection<ORq, ORs>
where
    ORq: Send + 'static,
    ORs: Send + 'static,
{
    pub(super) handlers: HashMap<(&'static Method, String), DFn<ORq, ORs>>,
}

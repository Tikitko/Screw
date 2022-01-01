use hyper::Method;
use screw_components::dyn_fn::DFn;
use std::collections::HashMap;

pub struct RoutesCollection<ORq, ORs> {
    pub(super) handlers: HashMap<(Method, String), DFn<ORq, ORs>>,
}

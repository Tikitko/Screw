use super::Handler;
use hyper::Method;
use std::collections::HashMap;

pub struct RoutesCollection {
    pub(super) handlers: HashMap<(Method, String), Handler>,
}

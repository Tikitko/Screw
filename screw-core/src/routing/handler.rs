use super::{Request, Response};
use screw_components::dyn_fn::DFn;

pub type Handler = DFn<Request, Response>;

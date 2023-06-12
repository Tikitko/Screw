use std::collections::HashMap;

pub struct DirectedRequest<ORq> {
    pub path: actix_router::Path<String>,
    pub query: HashMap<String, String>,
    pub origin: ORq,
}
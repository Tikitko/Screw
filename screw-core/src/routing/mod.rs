pub mod middleware;
pub mod route;
pub mod router;
pub mod routes;

pub mod actix {
    pub use actix_router::*;
}

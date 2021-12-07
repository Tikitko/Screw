pub mod maps;
pub mod routing;
pub mod server;

pub type DError = Box<dyn std::error::Error + Send + Sync>;
pub type DResult<T> = std::result::Result<T, DError>;

pub type DFn<P, R> = Box<dyn Fn(P) -> DFuture<R> + Send + Sync + 'static>;
pub type DFnOnce<P, R> = Box<dyn FnOnce(P) -> DFuture<R> + Send + Sync + 'static>;
pub type DFuture<R> = std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'static>>;

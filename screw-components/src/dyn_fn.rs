pub type DFn<P, R> = Box<dyn Fn(P) -> DFuture<R> + Send + Sync + 'static>;
pub type DFnOnce<P, R> = Box<dyn FnOnce(P) -> DFuture<R> + Send + Sync + 'static>;
pub type DFuture<R> = std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'static>>;

pub trait AsDynFn<F, R> {
    fn to_dyn_fn(self) -> DFn<F, R>;
}

impl<F, R, I, O> AsDynFn<I, O> for F
where
    F: Fn(I) -> R + Send + Sync + 'static,
    R: std::future::Future<Output = O> + Send + 'static,
    I: Send + 'static,
    O: Send + 'static,
{
    fn to_dyn_fn(self) -> DFn<I, O> {
        let fn_ref = std::sync::Arc::new(self);
        Box::new(move |i| {
            let fn_ref = fn_ref.clone();
            Box::pin(async move { fn_ref(i).await })
        })
    }
}

pub trait AsDynFnOnce<F, R> {
    fn to_dyn_fn_once(self) -> DFnOnce<F, R>;
}

impl<F, R, I, O> AsDynFnOnce<I, O> for F
where
    F: Fn(I) -> R + Send + Sync + 'static,
    R: std::future::Future<Output = O> + Send + 'static,
    I: Send + 'static,
    O: Send + 'static,
{
    fn to_dyn_fn_once(self) -> DFnOnce<I, O> {
        let fn_obj = self;
        Box::new(move |i| Box::pin(async move { fn_obj(i).await }))
    }
}

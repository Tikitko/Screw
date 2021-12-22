use hyper::Method;
use std::future::Future;
use std::marker::PhantomData;

pub struct Route {
    pub method: &'static Method,
    pub path: &'static str,
}

impl Route {
    pub fn with_handler<Rq, Rs, HFn, HFut>(self, handler: HFn) -> HandlerRoute<Rq, Rs, HFn, HFut>
    where
        Rq: Send + 'static,
        Rs: Send + 'static,
        HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = Rs> + Send + 'static,
    {
        HandlerRoute {
            method: self.method,
            path: self.path,
            handler,
            _p_rq: Default::default(),
            _p_rs: Default::default(),
            _p_h_fut: Default::default(),
        }
    }
}

pub struct HandlerRoute<Rq, Rs, HFn, HFut>
where
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
{
    pub(super) method: &'static Method,
    pub(super) path: &'static str,
    pub(super) handler: HFn,
    _p_rq: PhantomData<Rq>,
    _p_rs: PhantomData<Rq>,
    _p_h_fut: PhantomData<HFut>,
}

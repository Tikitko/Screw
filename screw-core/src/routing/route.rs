use hyper::Method;
use std::future::Future;
use std::marker::PhantomData;

pub struct RouteParams<HFn> {
    pub method: &'static Method,
    pub path: &'static str,
    pub handler: HFn,
}

pub struct Route<Rq, Rs, HFn, HFut>
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

impl<Rq, Rs, HFn, HFut> Route<Rq, Rs, HFn, HFut>
where
    Rq: Send + 'static,
    Rs: Send + 'static,
    HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
    HFut: Future<Output = Rs> + Send + 'static,
{
    pub fn new(params: RouteParams<HFn>) -> Self {
        Self {
            method: params.method,
            path: params.path,
            handler: params.handler,
            _p_rq: Default::default(),
            _p_rs: Default::default(),
            _p_h_fut: Default::default(),
        }
    }
}

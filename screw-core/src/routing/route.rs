use hyper::Method;
use std::future::Future;
use std::marker::PhantomData;

pub struct Route {
    method: &'static Method,
}

impl Route {
    pub fn with_method(method: &'static Method) -> Self {
        Self { method }
    }

    pub fn and_path(self, path: &'static str) -> RouteSecondPart {
        RouteSecondPart {
            method: self.method,
            path,
        }
    }
}

pub struct RouteSecondPart {
    method: &'static Method,
    path: &'static str,
}

impl RouteSecondPart {
    pub fn and_handler<Rq, Rs, HFn, HFut>(self, handler: HFn) -> RouteThirdPart<Rq, Rs, HFn, HFut>
    where
        Rq: Send + 'static,
        Rs: Send + 'static,
        HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = Rs> + Send + 'static,
    {
        RouteThirdPart {
            method: self.method,
            path: self.path,
            handler,
            _p_rq: Default::default(),
            _p_rs: Default::default(),
            _p_h_fut: Default::default(),
        }
    }
}

pub struct RouteThirdPart<Rq, Rs, HFn, HFut>
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

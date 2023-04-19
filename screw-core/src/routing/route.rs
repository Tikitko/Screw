pub use first::*;

pub mod first {
    use super::*;
    use hyper::Method;

    pub struct Route {
        method: &'static Method,
    }

    impl Route {
        pub fn with_method(method: &'static Method) -> Self {
            Self { method }
        }

        pub fn and_path(self, path: &'static str) -> second::Route {
            second::Route {
                method: self.method,
                path,
            }
        }
    }
}

pub mod second {
    use super::*;
    use hyper::Method;
    use std::future::Future;

    pub struct Route {
        pub(super) method: &'static Method,
        pub(super) path: &'static str,
    }

    impl Route {
        pub fn and_handler<Rq, Rs, HFn, HFut>(self, handler: HFn) -> third::Route<Rq, Rs, HFn, HFut>
        where
            Rq: Send + 'static,
            Rs: Send + 'static,
            HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
            HFut: Future<Output = Rs> + Send + 'static,
        {
            third::Route {
                method: self.method,
                path: self.path,
                handler,
                _p_rq: Default::default(),
                _p_rs: Default::default(),
                _p_h_fut: Default::default(),
            }
        }
    }
}

pub mod third {
    use hyper::Method;
    use std::future::Future;
    use std::marker::PhantomData;

    pub struct Route<Rq, Rs, HFn, HFut>
    where
        Rq: Send + 'static,
        Rs: Send + 'static,
        HFn: Fn(Rq) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = Rs> + Send + 'static,
    {
        pub(in super::super) method: &'static Method,
        pub(in super::super) path: &'static str,
        pub(in super::super) handler: HFn,
        pub(super) _p_rq: PhantomData<Rq>,
        pub(super) _p_rs: PhantomData<Rq>,
        pub(super) _p_h_fut: PhantomData<HFut>,
    }
}

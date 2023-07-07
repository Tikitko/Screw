pub mod first {
    use super::*;
    use hyper::Method;

    pub struct Route {
        methods: Vec<&'static Method>,
    }

    impl Route {
        pub fn with_method<M: Into<&'static Method>>(method: M) -> Self {
            Self {
                methods: vec![method.into()],
            }
        }

        pub fn with_methods<M: Into<Vec<&'static Method>>>(methods: M) -> Self {
            Self {
                methods: methods.into(),
            }
        }

        pub fn with_any_method() -> Self {
            Self { methods: vec![] }
        }

        pub fn and_path<P: Into<String>>(self, path: P) -> second::Route {
            second::Route {
                methods: self.methods,
                path: path.into(),
            }
        }
    }
}

pub mod second {
    use super::*;
    use hyper::Method;
    use std::future::Future;

    pub struct Route {
        pub(super) methods: Vec<&'static Method>,
        pub(super) path: String,
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
                methods: self.methods,
                path: self.path,
                handler,
                _p_rq: Default::default(),
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
        pub(in super::super) methods: Vec<&'static Method>,
        pub(in super::super) path: String,
        pub(in super::super) handler: HFn,
        pub(super) _p_rq: PhantomData<Rq>,
        pub(super) _p_h_fut: PhantomData<HFut>,
    }
}

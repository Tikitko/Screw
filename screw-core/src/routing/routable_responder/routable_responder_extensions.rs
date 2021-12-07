use crate::maps::DataMap;
use hyper::http::request::Parts;
use hyper::http::Extensions;
use hyper::{Body, Request};
use std::net::SocketAddr;
use std::sync::Arc;

pub trait RoutableResponderExtensions {
    fn remote_addr(&self) -> &SocketAddr;
    fn extract<T: Send + Sync + 'static>(&self) -> Option<&T>;
}

impl RoutableResponderExtensions for Extensions {
    fn remote_addr(&self) -> &SocketAddr {
        let remote_addr = self
            .get::<SocketAddr>()
            .expect("Must be paired with `RoutableResponder`!");

        remote_addr
    }

    fn extract<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        let data_map = self
            .get::<Arc<DataMap>>()
            .expect("Must be paired with `RoutableResponder`!");
        let object = data_map.get::<T>();

        object
    }
}

impl RoutableResponderExtensions for Parts {
    fn remote_addr(&self) -> &SocketAddr {
        self.extensions.remote_addr()
    }

    fn extract<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.extensions.extract()
    }
}

impl RoutableResponderExtensions for Request<Body> {
    fn remote_addr(&self) -> &SocketAddr {
        self.extensions().remote_addr()
    }

    fn extract<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.extensions().extract()
    }
}

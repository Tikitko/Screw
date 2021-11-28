use hyper::http::Extensions;

#[derive(Debug)]
pub struct DataMap {
    inner: Extensions,
}

impl DataMap {
    pub fn new() -> Self {
        Self {
            inner: Extensions::new(),
        }
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) {
        self.inner.insert(val);
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.inner.get::<T>()
    }
}

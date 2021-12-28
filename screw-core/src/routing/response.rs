use hyper::Body;

pub struct Response {
    pub http: hyper::Response<Body>,
}

pub mod routable_responder;
pub mod router;

pub fn query_params(uri: &hyper::Uri) -> std::collections::HashMap<String, String> {
    uri.query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(std::collections::HashMap::new)
}

pub use routable_responder::RoutableResponder;
pub use router::Router;

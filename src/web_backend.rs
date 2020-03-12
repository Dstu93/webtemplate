use crate::web::{WebServer, ApplicationError, HttpController, Middleware, RequestProcessor, HttpRequest, HttpResponse, HttpMethod};
use hyper::{Request, Response, Server, Body, Error, Method};
use std::collections::HashMap;
use hyper::header::HeaderName;


impl Into<HttpRequest> for Request<Body> {
    fn into(self) -> HttpRequest {
        let params = self.uri()
            .query()
            .map(|v| {
                url::form_urlencoded::parse(v.as_bytes())
                    .into_owned()
                    .collect()
            })
            .unwrap_or_else(HashMap::new);
        let headers = self.headers()
            .iter()
            .map(|v|
                (v.0.as_str().into(),String::from_utf8_lossy(v.1.as_bytes()).into_owned()))
            .collect();
        let cookies = self.headers().get("Cookies".into());
        HttpRequest{
            method: self.method().into(),
            path: self.uri().path().into(),
            params,
            headers,
            cookies,
            body: vec![]
        }
    }
}

impl From<&Method> for HttpMethod {
    fn from(m: &Method) -> Self {
        match m {
            &Method::GET => HttpMethod::Get,
            &Method::POST => HttpMethod::Post,
            &Method::PUT => HttpMethod::Put,
            &Method::DELETE => HttpMethod::Delete,
            _ => HttpMethod::Unsupported,
        }
    }
}

impl From<HttpResponse> for Response<Body> {
    fn from(_: HttpResponse) -> Self {
        unimplemented!()
    }
}
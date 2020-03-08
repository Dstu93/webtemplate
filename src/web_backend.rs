use crate::web::{WebServer, ApplicationError, HttpController, Middleware, RequestProcessor, HttpRequest, HttpResponse, HttpMethod};
use hyper::{Request, Response, Server, Body, Error};


impl Into<HttpRequest> for Request<Body> {
    fn into(self) -> HttpRequest {
        unimplemented!()
    }
}

impl From<HttpResponse> for Response<Body> {
    fn from(_: HttpResponse) -> Self {
        unimplemented!()
    }
}
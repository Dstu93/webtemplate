use crate::web::{WebServer, ApplicationError, HttpController, Middleware, RequestProcessor, HttpRequest, HttpResponse, HttpMethod};
use std::sync::mpsc::{Receiver, Sender};
use hyper::{Request, Response};

pub struct WebServerImpl;

impl WebServer<Request<Vec<u8>>, Response<Vec<u8>>> for WebServerImpl {
    fn start(self, ip: String, port: u16, processor: Box<dyn RequestProcessor<Request<Vec<u8>>, Response<Vec<u8>>>>) -> Result<(), ApplicationError> {
        //TODO
        unimplemented!()
    }
}

impl Into<HttpRequest> for Request<Vec<u8>> {
    fn into(self) -> HttpRequest {
        unimplemented!()
    }
}

impl From<HttpResponse> for Response<Vec<u8>> {
    fn from(_: HttpResponse) -> Self {
        unimplemented!()
    }
}
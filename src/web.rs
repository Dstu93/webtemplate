use std::collections::HashMap;
use std::io::Error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::borrow::BorrowMut;
use std::ops::Deref;

#[derive(Debug,Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub params: HashMap<String,String>,
    pub headers: HashMap<String,String>,
    pub cookies: HashMap<String,String>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// Parsed den Payload der Request als JSON zum Objekt T
    pub fn as_json<'a,T: Deserialize<'a>>(&'a self) -> Result<T,JsonError> {
        match serde_json::from_slice(self.body.as_slice()) {
            Ok(t) => {Ok(t)},
            Err(_) => { Err(JsonError::CouldNotParse) },
        }
    }
}


#[derive(Copy, Clone,Ord, PartialOrd, Eq, PartialEq,Hash,Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Delete,
    Put,
}

#[derive(Debug,Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String,String>,
    pub body: Vec<u8>,
}

impl HttpResponse {

    pub fn new(body: Vec<u8>,content_type: &str,status: u16) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Connection".into(),"close".into());
        headers.insert("Content-Type".into(),content_type.into());
        HttpResponse{
            status,
            headers: headers,
            body,
        }
    }

    pub fn not_found() -> Self {
        let mut headers = HashMap::new();
        headers.insert("Connection".into(),"close".into());
        HttpResponse{
            status: 404,
            headers,
            body: vec![]
        }
    }

    pub fn with_json<T: Serialize>(payload: &T) -> Self {
        let body = serde_json::to_string(payload).unwrap();
        let mut headers = HashMap::new();
        headers.insert("Connection".into(),"close".into());
        headers.insert("Content-Type".into(),"application/json".into());
        HttpResponse{
            status: 200,
            headers: headers,
            body: body.into_bytes(),
        }
    }
}

/// WebController implementiert die Logik zum verarbeiten einer Request.
pub trait HttpController: Sync + Send {

    /// URL für die der WebController die Requests bearbeitet.
    fn url(&self) -> String;

    fn on_post(&mut self,_req: &HttpRequest) -> HttpResponse {
        HttpResponse::not_found()
    }

    fn on_get(&mut self,_req: &HttpRequest) -> HttpResponse {
        HttpResponse::not_found()
    }

    fn on_put(&mut self, _req: &HttpRequest) -> HttpResponse {
        HttpResponse::not_found()
    }

    fn on_delete(&mut self, _req: &HttpRequest) -> HttpResponse {
        HttpResponse::not_found()
    }

}

/// Middleware nimmt Requests noch vor der Bearbeitung des entsprechenden WebControllers entgegen.
/// Die Middleware kann die Request bearbeiten, weiterleiten oder ablehnen.
/// Über Middlewares lässt sich z.B. Logging oder Authorisierung realisieren.
pub trait Middleware: Send + Sync{
    /// Verarbeitet die einzelne Request und kann über den Rückgabetyp entscheiden
    /// ob die Request an die nächste Middleware oder WebController weitergereicht,abgelehnt oder
    /// vorher noch bearbeitet wird.
    fn process(&mut self,req: &mut HttpRequest) -> ProcessResult;
}

/// Result einer Verarbeitung durch eine Middleware
pub enum ProcessResult {
    /// Request wird an die nächste Middleware oder an den
    /// entsprechenden WebController weitergeleitet.
    Done,
    /// Request wird direkt mit folgender Response beantwortet
    Response(HttpResponse),
}

pub trait WebServer<I: Into<HttpRequest>, O: Into<HttpResponse>> {
    fn start(self,ip: String, port: u16, processor: dyn RequestProcessor<I,O>) -> Result<(),ApplicationError>;
}

pub trait RequestProcessor<I: Into<HttpRequest>, O: From<HttpResponse>>: Sync {
    fn dispatch(&mut self, controller: Vec<Arc<dyn HttpController>>, middlewares: Vec<Arc<dyn Middleware>>);
    fn process(&mut self, req: I) -> O;
}

pub struct StandardRequestProcessor {
    middlewares: Vec<Arc<dyn Middleware>>,
    controller: Vec<Arc<dyn HttpController>>,
}

impl <I,O>RequestProcessor<I,O> for StandardRequestProcessor where I: Into<HttpRequest>,O: From<HttpResponse> {
    fn dispatch(&mut self, controller: Vec<Arc<dyn HttpController>>, middlewares: Vec<Arc<dyn Middleware>>) {
        self.middlewares = middlewares;
        self.controller = controller;
    }

    fn process(&mut self, req: I) -> O {
        let mut req = req.into();
        for middleware in &mut self.middlewares {
            match middleware.process(&mut req) {
                ProcessResult::Done => {continue},
                ProcessResult::Response(resp) => {return resp.clone().into()}
            }
        }
        HttpResponse::not_found().into()
    }
}

pub enum ApplicationError {
    IoError(Error),
}

#[derive(Copy, Clone,Ord, PartialOrd, Eq, PartialEq,Debug,Hash)]
pub enum JsonError {
    CouldNotParse,
}

impl Into<HttpResponse> for JsonError {
    fn into(self) -> HttpResponse {
        HttpResponse::new(Vec::new(),"application/json",400)
    }
}
use std::collections::HashMap;
use std::io::Error;
use serde::{Deserialize, Serialize};
use crossbeam_channel::{Sender, Receiver};
use std::thread::Builder;
use std::thread::JoinHandle;

#[derive(Debug,Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
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
            headers,
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
        let body = serde_json::to_vec(payload)
            .expect("could not deserialize payload");
        let mut headers = HashMap::new();
        headers.insert("Connection".into(),"close".into());
        headers.insert("Content-Type".into(),"application/json".into());
        HttpResponse{
            status: 200,
            headers,
            body,
        }
    }
}

/// WebController implementiert die Logik zum verarbeiten einer Request.
pub trait HttpController: Sync + Send {

    /// URL für die der WebController die Requests bearbeitet.
    fn url(&self) -> &'static str;

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
pub trait Middleware: Send + Sync {
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

pub trait WebServer {
    fn start(self, processor: Box<dyn RequestProcessor>) -> Result<(),ApplicationError>;
}

pub trait RequestProcessor: Sync + Send{
    fn process(&mut self, req: HttpRequest) -> HttpResponse;
}

pub struct StandardRequestProcessor {
    middlewares: Vec<Box<dyn Middleware>>,
    controller: Vec<Box<dyn HttpController>>,
}

impl StandardRequestProcessor {
    pub fn new(middlewares: Vec<Box<dyn Middleware>>, controller: Vec<Box<dyn HttpController>>) -> Self {
        StandardRequestProcessor{ middlewares, controller }
    }
}

impl RequestProcessor for StandardRequestProcessor {

    fn process(&mut self, req: HttpRequest) -> HttpResponse {
        let mut req = req.into();
        for m in self.middlewares.iter_mut() {
            match m.process(&mut req) {
                ProcessResult::Done => continue,
                ProcessResult::Response(resp) => return resp.into(),
            }
        }
        //Middlewares sind durchlaufen, nun koennen wir den entsprechenden HttpController arbeiten lassen.
        let controller_option = self.controller
            .iter_mut()
            .find(|http_controller| http_controller.url().eq(&req.path));
        match controller_option {
            None => {HttpResponse::not_found()},
            Some(c) => match req.method {
                HttpMethod::Get => c.on_get(&req),
                HttpMethod::Post => c.on_post(&req),
                HttpMethod::Delete => c.on_delete(&req),
                HttpMethod::Put => c.on_put(&req),
            },
        }
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
        let mut response = HttpResponse::new(Vec::new(),
                                         "application/json",400);
        response.headers.insert("Accept".into(),"application/json".into());
        response
    }
}

#[derive(Clone)]
/// Channel for bidirectional Communication with 2 different Structs
pub struct BidirectionalChannel<I,O> where I: Send,O: Send {
    sender: Sender<I>,
    recv: Receiver<O>,
}

impl<I, O> BidirectionalChannel<I, O> where I: Send, O: Send {
    pub fn new() ->  (BidirectionalChannel<I,O>,BidirectionalChannel<O,I>) {
        let (sender1,recv1) = crossbeam_channel::unbounded();
        let (sender2,recv2) = crossbeam_channel::unbounded();
        let first_channel = BidirectionalChannel{
            sender: sender1,
            recv: recv2,
        };
        let second_channel = BidirectionalChannel {
            sender: sender2,
            recv: recv1,
        };

        (first_channel,second_channel)
    }

    pub async fn send_n_receive(&self, i: I) -> O {
        self.sender.send(i).expect("could not send");
        self.recv.recv().expect("could not receive from channel")
    }

    pub fn send_n_receive_sync(&self,i: I) -> O {
        self.sender.send(i).expect("could not send");
        self.recv.recv().expect("could not receive from channel")
    }

    pub fn send(&self, input: I) {
        self.sender.send(input).expect("could not send");
    }

    pub fn recv(&self) -> O {
        self.recv.recv().expect("could not listen on Channel")
    }
}


pub struct RequestDispatcher {
    workers: Vec<JoinHandle<()>>,
}

impl RequestDispatcher {

    pub fn new() -> Self {
        RequestDispatcher{ workers: Vec::new() }
    }

    pub fn register(&mut self) -> BidirectionalChannel<HttpRequest,HttpResponse> {
        //TODO StandardRequestProcessor factory
        let mut request_processor = StandardRequestProcessor {
            middlewares: vec![],
            controller: vec![]
        };
        let (channel1,channel2) = BidirectionalChannel::new();
        let handle = Builder::new()
            .name(format!("Http-Worker-{}",self.workers.len() + 1 ))
            .spawn(move || {
                loop {
                    let req = channel1.recv();
                    println!("Processing Request {:#?}",req); //TODO remove
                    let response = request_processor.process(req);
                    channel1.send(response);
                }
            }).expect("could not spawn worker thread");
        self.workers.push(handle);
        channel2
    }
}


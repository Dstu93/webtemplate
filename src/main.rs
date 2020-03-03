use crate::example::CountController;
use crate::web::{StandardRequestProcessor, HttpController, WebServer};

mod web;
mod web_backend;
mod example;

fn main() {
    let web_controller: Vec<Box<dyn HttpController>> = vec![Box::new(CountController::new())];
    let middleware = vec![];
    let request_processor = StandardRequestProcessor {
        middlewares: middleware,
        controller: web_controller,
    };
    let webserver = CanteenWebserver;
    webserver.start("127.0.0.1".into(),8080u16,Box::new(request_processor));
}

use std::sync::{RwLock};
use crate::web::{HttpController, HttpRequest, HttpResponse};
use std::ops::{Add, AddAssign};

pub struct CountController {
    counter: RwLock<u64>,
}

impl CountController {
    pub fn new() -> Self {
        CountController{
            counter: RwLock::new(0)
        }
    }
}


impl HttpController for CountController {

    fn url(&self) -> &'static str {
        "/api/counter"
    }

    fn on_get(&mut self, _req: &HttpRequest) -> HttpResponse {
        let counter = match self.counter.get_mut() {
            Ok(c) => {c.add_assign(1); c},
            Err(_) => {
                return HttpResponse::new(Vec::new(),"text/html",500);
            }
        };
        HttpResponse::new(format!("Site called {} times",counter).into_bytes(),"text/html",200)
    }
}
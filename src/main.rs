use crate::example::CountController;
use crate::web::{StandardRequestProcessor, HttpController, WebServer};

mod web;
mod web_backend;
mod example;


use std::cell::Cell;
use std::rc::Rc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Response, Server, Request};
use std::sync::mpsc::channel;

fn main() {

    // Configure a runtime that runs everything on the current thread
    let mut rt = tokio::runtime::Builder::new()
        .enable_all()
        .basic_scheduler()
        .build()
        .expect("build runtime");

    // Combine it with a `LocalSet,  which means it can spawn !Send futures...
    let local = tokio::task::LocalSet::new();
    local.block_on(&mut rt, run());
}

async fn run() {
    let addr = ([127, 0, 0, 1], 3000).into();

    let (sender,recv) = channel::<String>();
    // Using a !Send request counter is fine on 1 thread...
    let counter = Rc::new(Cell::new(0));
    let (sender,recv) = crossbeam_channel::unbounded();
    let (rsp_sender,rsp_recv) = crossbeam_channel::unbounded::<Response<Body>>();

    let make_service = make_service_fn(move |_| {
        // For each connection, clone the counter to use in our service...
        let cnt = counter.clone();
        let sender = sender.clone();
        let response_recv = rsp_recv.clone();
        async move {
            Ok::<_, Error>(service_fn(move |req: Request<Body>| {
                let prev = cnt.get();
                cnt.set(prev + 1);
                let value = cnt.get();
                let req = sender.send(req);
                let response_recv = response_recv.clone();
                async move {
                    Ok::<_, Error>(response_recv.recv().expect("http response recv crashed"))
                }
            }))
        }
    });

    let server = Server::bind(&addr).executor(LocalExec).serve(make_service);

    println!("Listening on http://{}", addr);

    // The server would block on current thread to await !Send futures.
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

// Since the Server needs to spawn some background tasks, we needed
// to configure an Executor that can spawn !Send futures...
#[derive(Clone, Copy, Debug)]
struct LocalExec;

impl<F> hyper::rt::Executor<F> for LocalExec
    where
        F: std::future::Future + 'static, // not requiring `Send`
{
    fn execute(&self, fut: F) {
        // This will spawn into the currently running `LocalSet`.
        tokio::task::spawn_local(fut);
    }
}
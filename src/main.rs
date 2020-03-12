use crate::example::CountController;
use crate::web::{StandardRequestProcessor, HttpController, WebServer, RequestDispatcher};

mod web;
mod web_backend;
mod example;
mod factory;


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
    let mut dispatcher = RequestDispatcher::new();
    let make_service = make_service_fn(move |_| {
        // For each connection, clone the counter to use in our service...
        let req_channel = dispatcher.register();
        async move {
            let req_channel = req_channel.clone();
            Ok::<_, Error>(service_fn(move |req: Request<Body>| {
                let chnl = req_channel.clone();
                async move {
                    Ok::<_, Error>(chnl.send_n_receive(req.into()).await.into())
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
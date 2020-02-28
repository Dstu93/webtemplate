use crate::web::{WebServer, ApplicationError, HttpController, Middleware, RequestProcessor};
use canteen::Canteen;

pub struct CanteenWebserver;

// impl WebServer for CanteenWebserver {
//     fn start(self, ip: String, port: u16, processor: RequestProcessor<_, _>) -> Result<(), ApplicationError> {
//         unimplemented!()
//     }
//     fn start(self, ip: String, port: u16, controller: Vec<Box<dyn WebController>>, middlewares: Vec<Box<dyn Middleware>>) -> Result<(), ApplicationError> {
//         let mut canteen = Canteen::new();
//         canteen.bind(format!("{}:{}",ip,port));
//         //canteen.set_default() //TODO
//         Ok(())
//     }
// }

//TODO From für die Requests und Responses
// RequestProcessor implementieren... RequestProcessor muss kein Trait sein sondern kann eine Struct sein...
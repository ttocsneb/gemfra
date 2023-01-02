//! Simple Application
//!
//! Run the application with a single handler. You will need to implement the
//! [Handler] trait and supply it to a [SimpleApp].
//!
//! ### Example
//!
//! ```no_run
//! use async_trait::async_trait;
//! use std::error::Error;
//! use gemfra::{
//!     simple::{Handler, SimpleApp},
//!     request::Request,
//!     response::Response
//! };
//!
//! struct MyHandler;
//!
//! #[async_trait]
//! impl Handler for MyHandler {
//!     async fn handle(&self, request: Request) -> Result<Response, Box<dyn Error>> {
//!         Ok(Response::success("text/gemini", "# Hello World!
//!
//! Welcome to my simple app! This could have been just a simple gmi file, but I
//! guess it makes sense as an example :).
//! "))
//!     }
//! }
//!
//!
//! let my_app = SimpleApp::new(&MyHandler);
//!
//! todo!("Run my app");
//! ```
//!
//!
use std::{error::Error, io};

use async_trait::async_trait;

use crate::{
    error::GemError,
    protocol::Cgi,
    request::{cgi_request, Request},
    response::Response,
};

/// A handler for a SimpleApp
#[async_trait]
pub trait Handler {
    /// handle any request
    ///
    /// Take a gemini request and return a gemini response. It is possible to
    /// return an error, but that should be avoided. Errors will be logged and the generated
    /// response will be a CGI Error.
    async fn handle(&self, request: Request) -> Result<Response, Box<dyn Error>>;
}

/// A single page application
///
/// This application will send all requests to the provided handler.
///
/// The only setup required is initialization. Once the app is created, you may
/// start the app.
pub struct SimpleApp<'a> {
    handler: &'a (dyn Handler + Send + Sync),
}

impl<'a> SimpleApp<'a> {
    /// Create a new simple app
    #[inline]
    pub fn new(handler: &'a (dyn Handler + Send + Sync)) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl<'a> Cgi for SimpleApp<'a> {
    async fn run_cgi(self) {
        async fn send_response(response: Response) {
            if let Err(err) = response.send_sync(&mut io::stdout()).await {
                eprintln!("{err}");
            }
        }

        let request = match cgi_request() {
            Ok(request) => request,
            Err(err) => {
                eprintln!("{err}");
                send_response(err.into()).await;
                return;
            }
        };

        let response = match self.handler.handle(request).await {
            Ok(result) => result,
            Err(err) => {
                // There was an error
                eprintln!("{err}");
                // If the error was a GemError, it can be converted into a response
                match err.downcast::<GemError>() {
                    Ok(err) => Response::from(*err),
                    Err(err) => Response::error_cgi(err.to_string()),
                }
            }
        };

        send_response(response).await;
    }
}

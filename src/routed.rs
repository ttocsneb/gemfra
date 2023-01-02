//! Routed Application
//!
//! Use paths to serve multiple pages. You can create a [RoutedApp] then
//! register [Route]s that your application will handle.
//!
//! You can create a route in the following way:
//!
//! ```
//! use async_trait::async_trait;
//! use std::error::Error;
//! use gemfra::{routed::Route, request::Request, response::Response};
//! use route_recognizer::Params;
//!
//! struct MyRoute;
//!
//! #[async_trait]
//! impl Route for MyRoute {
//!     fn endpoint(&self) -> &str {
//!         "/myroute/:var"
//!     }
//!
//!     async fn handle(&self, params: &Params, request: Request) -> Result<Response, Box<dyn Error>> {
//!         let var = params.find("var").unwrap();
//!
//!         Ok(Response::success("text/gemini", format!("You've found {var}")))
//!     }
//! }
//! ```
//!
//! > Currently there is a lot of boilerplate invovled in creating a route, but I
//! > hope to reduce this by creating a directive that can convert a function into
//! > a [Route] struct.
//!
//!
//! You can then register your route with the app:
//!
//! ```
//! # use async_trait::async_trait;
//! # use std::error::Error;
//! # use gemfra::{routed::Route, request::Request, response::Response};
//! # use route_recognizer::Params;
//! # struct MyRoute;
//! # #[async_trait]
//! # impl Route for MyRoute {
//! #     fn endpoint(&self) -> &str {
//! #         "/myroute/:var"
//! #     }
//! #    async fn handle(&self, params: &Params, request: Request) -> Result<Response, Box<dyn Error>> {
//! #        let var = params.find("var").unwrap();
//! #        Ok(Response::success("text/gemini", format!("You've found {var}")))
//! #   }
//! # }
//! use gemfra::routed::RoutedApp;
//!
//! let mut my_app = RoutedApp::new();
//!
//! my_app.register(&MyRoute);
//! ```
//!

use async_trait::async_trait;
use std::error::Error;
use std::io;

use crate::error::GemError;
use crate::protocol::Cgi;
use crate::request::{cgi_request, Request};
use crate::response::Response;

use route_recognizer::{Params, Router};

/// A handler to an endpoint
///
/// ## Example
///
/// ```
/// use async_trait::async_trait;
/// use std::error::Error;
/// use gemfra::{routed::Route, request::Request, response::Response};
/// use route_recognizer::Params;
///
/// struct MyRoute;
///
/// #[async_trait]
/// impl Route for MyRoute {
///     fn endpoint(&self) -> &str {
///         "/myroute/:var"
///     }
///
///     async fn handle(&self, params: &Params, request: Request) -> Result<Response, Box<dyn Error>> {
///         let var = params.find("var").unwrap();
///
///         Ok(Response::success("text/gemini", format!("You've found {var}")))
///     }
/// }
/// ```
#[async_trait]
pub trait Route {
    /// The endpoint that this route handles
    ///
    /// The endpoint can have four kinds of route segments:
    /// - __segments__: these are of the format `/a/b`.
    /// - __params__: these are of the format `/a/:b`.
    /// - __named wildcards__: these are of the format `/a/*b`.
    /// - __unnamed wildcards__: these are of the format `/a/*`.
    ///
    /// The values for params and named wildcards will be provided in the params
    /// variable of [handle](Route::handle).
    fn endpoint(&self) -> &str;

    /// Handle a request for the route
    ///
    /// Take a gemini request and return a gemini response. It is possible to
    /// return an error, but that should be avoided. Errors will be logged and the generated
    /// response will be a CGI Error.
    ///
    /// params are the path parameters that were requested when registering the route
    async fn handle(&self, params: &Params, request: Request) -> Result<Response, Box<dyn Error>>;
}

/// An application that can have multiple endpoints
///
/// Endpoints are registered using [register](RoutedApp::register) where each
/// endpoint refers to a different [Route].
///
/// Once the app is setup, you can start it using the following commands:
///
/// * [run_cgi](RoutedApp::run_cgi): Run a cgi application
/// * __todo__: More application types need to be developped
pub struct RoutedApp<'a> {
    router: Router<&'a (dyn Route + Send + Sync)>,
}

impl<'a> RoutedApp<'a> {
    /// Create a new routed capsule
    #[inline]
    pub fn new() -> Self {
        Self {
            router: Router::new(),
        }
    }

    /// Register a route to the app.
    #[inline]
    pub fn register(&mut self, route: &'a (dyn Route + Send + Sync)) {
        self.router.add(route.endpoint(), route)
    }
}

#[async_trait]
impl<'a> Cgi for RoutedApp<'a> {
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

        let route = match self.router.recognize(&request.path) {
            Ok(val) => val,
            Err(_) => {
                // There was no route found for the endpoint
                send_response(Response::not_found("Path not found")).await;
                return;
            }
        };

        let params = route.params();
        let handler = **route.handler();
        let response = match handler.handle(params, request).await {
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

//! Routed Application
//!
//! Use paths to serve multiple pages. You can create a [RoutedApp] then
//! register [Route]s that your application will handle.
//!
//! You can create a route, then register it to a RoutedApp.
//!
//! ```
//! use gemfra::{
//!     routed::RoutedApp,
//!     request::Request,
//!     response::Response,
//!     error::AnyError,
//! };
//! use gemfra_codegen::route;
//!
//! #[route("/myroute/:var")]
//! async fn my_route(request: Request, var: &str) -> Result<Response, AnyError> {
//!     Ok(Response::success("text/gemini", format!("You've found {var}")))
//! }
//!
//! let mut my_app = RoutedApp::new();
//!
//! my_app.register(&my_route);
//! ```
//!
//! > In order to use the route macro, you will need to
//! > include gemfra-codegen in your Cargo.toml file
//!

use async_trait::async_trait;

use crate::request::Request;
use crate::response::Response;
use crate::{application::Application, error::AnyError};

pub use route_recognizer::{Params, Router};

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
///     async fn handle(&self, params: &Params, request: Request) -> Result<Response, Box<dyn Error + Send + Sync>> {
///         let var = params.find("var").unwrap();
///
///         Ok(Response::success("text/gemini", format!("You've found {var}")))
///     }
/// }
/// ```
///
/// You can instead use the route macro for a more simple route implementation
///
/// ```
/// use gemfra::{
///     routed::Route,
///     request::Request,
///     response::Response,
///     error::AnyError
/// };
/// use gemfra_codegen::route;
///
/// #[route("/myroute/:var")]
/// async fn my_route(request: Request, var: &str) -> Result<Response, AnyError> {
///     Ok(Response::success("text/gemini", format!("You've found {var}")))
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
    async fn handle(&self, params: &Params, request: Request) -> Result<Response, AnyError>;
}

/// An application that can have multiple endpoints
///
/// Endpoints are registered using [register](RoutedApp::register) where each
/// endpoint refers to a different [Route].
///
/// Once the app is setup, you can start it with a protocol command, see
/// [protocol](crate::protocol).
pub struct RoutedApp {
    router: Router<&'static (dyn Route + Send + Sync)>,
}

impl RoutedApp {
    /// Create a new routed capsule
    #[inline]
    pub fn new() -> Self {
        Self {
            router: Router::new(),
        }
    }

    /// Register a route to the app.
    #[inline]
    pub fn register(&mut self, route: &'static (dyn Route + Send + Sync)) {
        self.router.add(route.endpoint(), route)
    }
}

#[async_trait]
impl Application for RoutedApp {
    async fn handle_request(&self, request: Request) -> Result<Response, AnyError> {
        let route = match self.router.recognize(&request.path) {
            Ok(val) => val,
            Err(_) => {
                return Ok(Response::not_found("Path not found"));
            }
        };

        let params = route.params();
        let handler = **route.handler();

        handler.handle(params, request).await
    }
}

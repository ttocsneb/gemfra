//! Base Application
//!
//! All applications impl from [Application]. If a struct impls Application,
//! then all protocols will implicitly impl the struct. This makes it very easy
//! to make custom applications.
//!
//! ### Example
//!
//! If you wanted to create a single page application that ran on CGI, your
//! application might look something like this.
//!
//! ```no_run
//! use async_trait::async_trait;
//! use gemfra::{
//!     application::Application,
//!     error::AnyError,
//!     request::Request,
//!     response::Response,
//!     protocol::Cgi,
//! };
//!
//! struct MyApp;
//!
//! #[async_trait]
//! impl Application for MyApp {
//!     async fn handle_request(&self, request: Request) -> Result<Response, AnyError> {
//!         Ok(Response::success("text/gemini", "# Hello World!
//!
//! Welcome to my simple app! This could have been just a simple gmi file, but I
//! guess it makes sense as an example :).
//! "))
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     MyApp.run_cgi().await;
//! }
//! ```
//!

use async_trait::async_trait;

use crate::{error::AnyError, request::Request, response::Response};

/// Base Application
///
/// All applications impl from this trait. If a struct impls Application,
/// then all protocols will implicitly impl the struct. This makes it very easy
/// to make custom applications.
#[async_trait]
pub trait Application {
    /// Process an incoming request.
    ///
    /// A response should be returned. If an error occurs, there are several
    /// options that can be done to deal with the error:
    ///
    /// 1. Raise the error. This will result in an `42 Internal Server Error` response.
    /// 2. Convert the error into a [GemError](crate::error::GemError) using
    ///    [ToGemError](crate::error::ToGemError). GemErrors will be converted into an
    ///    apropriate response.
    /// 3. Return the response that you would like the client to see.
    async fn handle_request(&self, request: Request) -> Result<Response, AnyError>;
}

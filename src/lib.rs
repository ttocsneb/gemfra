//! # Gemfra: A Rust Gemini Framework
//!
//! With gemfra, you will be able to more easily create dynamic content on gemini
//! using rust. This framework abstracts out most of the boilerplate involved in
//! a cgi application.
//!
//! Gemfra is split into two sections: The application and the protocol. The
//! application is the way in which you will handle incoming requests. The
//! protocol determines how you will install your application.
//!
//! This framework is setup so that if you change your protocol, your code will
//! remain the same.
//!
//! ## Quickstart
//!
//! You will need to choose which kind of application you want to run, and what
//! protocol you want to use. Below is a list of each available where more details
//! will be provided on how to implement your chosen setup.
//!
//! There are two kinds of applications that are available:
//!
//! * [Routed App](routed): Build an app with multiple pages
//! * [Simple App](simple): Build an app with only one page
//!
//! Once you have setup your app, you can start it using your preferred protocol.
//!
//! * [run_cgi](protocol::Cgi::run_cgi): Run a CGI application
//!
//! ## Limitations
//!
//! Currently this software has only been tested with [stargazer](https://git.sr.ht/~zethra/stargazer)
//! While protocol implementations should work across different softwares, it is
//! possible if not likely that this software will break with different servers.
//! If that is the case, please submit an issue or pull request. I would like
//! this framework to be compatible with as many servers as possible.
//!
//! ## Roadmap
//!
//! This is the beginning of the project and there are several things that need
//! to be implemented. Below is a list of what I would like to implement before
//! considering a 0.1.0 release.
//!
//! * [X] Create multi-page app
//! * [X] Create single-page app
//! * [X] Implement CGI
//! * [ ] Implement SCGI
//! * [ ] Ability to raise error responses
//! * [ ] Convert any error into a [GemError](error::GemError)
//! * [ ] Reduce boilerplate with attribute macros
//! * [ ] Implement FastCGI (If there are any servers that support it)
//! * [ ] Implement any other commonly used CGI-like protocols in geminispace
//! * [ ] Add support for handling authentication
//! * [ ] Divide sections into features
//!
pub mod error;
pub mod protocol;
pub mod request;
pub mod response;
pub mod routed;
pub mod simple;

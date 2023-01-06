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
//! * [Application](application): Build an app with only one page
//!
//! Once you have setup your app, you can start it using your preferred protocol.
//!
//! * [run_cgi](protocol::Cgi::run_cgi): Run a CGI application
//! * [run_scgi](protocol::Scgi::run_scgi): Run a SCGI application
#![cfg_attr(doc, feature(doc_auto_cfg))]

pub mod application;
pub mod error;
pub mod protocol;
pub mod request;
pub mod response;
#[cfg(feature = "routed")]
pub mod routed;

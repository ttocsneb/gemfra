//! Available Protocols
//!
//! Every protocol is a trait that is implemented by an application. This allows
//! for each protocol to be executed in the same way regardless of the application
//! that is used.

use async_trait::async_trait;

/// Common Gateway Interface
///
/// Run the application using the CGI protocol. This is a one-shot program that
/// gets request information from environment variables and sends the response
/// to stdout.
#[async_trait]
pub trait Cgi {
    /// Run the application using the CGI protocol. This is a one-shot program that
    /// gets run as a new process for every request made. Request information is
    /// taken from environment variables and sends the response to stdout.
    ///
    /// It is important that stdout is not used for logging, stderr should be
    /// used instead for logging e.g. [`eprintln!()`](std::eprintln)
    ///
    /// Because a new process is created for every request, any time used to
    /// setup the application is re-ran for every request. If there is a
    /// considerable setup time required for your application, it might be better
    /// to use an alternative protocol.
    ///
    /// after the function finishes, the application should exit as soon as
    /// possible because the client connection will not be closed until the
    /// application has stopped.
    async fn run_cgi(self);
}

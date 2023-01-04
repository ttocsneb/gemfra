//! Available Protocols
//!
//! Every protocol is a trait that is implemented by an application. This allows
//! for each protocol to be executed in the same way regardless of the application
//! that is used.

use std::{collections::HashMap, env, error::Error, io, sync::Arc};

use async_trait::async_trait;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

use crate::{
    application::Application,
    error::{GemError, ToGemError},
    request::Request,
    response::Response,
};

async fn send_cgi_response(response: Response) {
    if let Err(err) = response.send_sync(&mut io::stdout()).await {
        eprintln!("Could not send response: {err}");
    };
}

fn get_cgi_header(key: &str) -> Result<String, GemError> {
    env::var(key).into_gem()
}

/// Common Gateway Interface
///
/// Run the application using the CGI protocol. This is a one-shot program that
/// gets request information from environment variables and sends the response
/// to stdout.
#[async_trait]
pub trait Cgi: Application + Sized + Send + Sync + 'static {
    /// Run the application using the CGI protocol. This is a one-shot program that
    /// gets run as a new process for every request made. Request information is
    /// taken from environment variables and the response is sent to stdout.
    ///
    /// It is important that stdout is not used for logging as this will interfere
    /// with the response, stderr should be used instead for logging e.g.
    /// [`eprintln!()`](std::eprintln).
    ///
    /// Because a new process is created for every request, any time used to
    /// setup the application is re-run for every request. If there is a
    /// considerable amount of setup/cleanup time required for your application,
    /// it might be better to use an alternative protocol.
    ///
    /// After the function finishes, the application should exit as soon as
    /// possible because the client connection will not be closed until the
    /// application has stopped. This is important because some clients will not
    /// display a response until the connection is fully closed.
    ///
    /// ### Example
    ///
    /// ```no_run
    /// use gemfra::{
    ///     protocol::Cgi,
    ///     application::Application,
    ///     request::Request,
    ///     response::Response,
    ///     error::AnyError,
    /// };
    /// use async_trait::async_trait;
    ///
    /// struct MyApp;
    /// #[async_trait]
    /// impl Application for MyApp {
    ///     async fn handle_request(&self, request: Request) -> Result<Response, AnyError> {
    ///         todo!("Handle the request")
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     MyApp.run_cgi().await;
    /// }
    /// ```
    async fn run_cgi(self) {
        let request = match Request::parse_request(get_cgi_header) {
            Ok(request) => request,
            Err(err) => {
                eprintln!("Invalid CGI header: {err}");
                send_cgi_response(Response::error_cgi("Invalid CGI header")).await;
                return;
            }
        };

        let response = match self.handle_request(request).await {
            Ok(response) => response,
            Err(err) => {
                eprintln!("Error while handling request: {err}");
                match err.downcast::<GemError>() {
                    Ok(err) => Response::from(*err),
                    Err(_) => Response::error_cgi("Internal Server Error"),
                }
            }
        };

        send_cgi_response(response).await;
    }
}

impl<A> Cgi for A where A: Application + Send + Sync + 'static {}

async fn read_scgi_request(conn: &mut TcpStream) -> Result<Request, Box<dyn Error + Send + Sync>> {
    // Read the length of the headers
    let mut buf = Vec::new();
    loop {
        let chr = conn.read_u8().await?;
        if chr == b':' {
            break;
        }
        buf.push(chr);
    }
    let size: usize = String::from_utf8(buf)?.parse()?;

    // Read the headers
    let mut buffer = BytesMut::zeroed(size);
    conn.read_exact(buffer.as_mut()).await?;

    // Parse the headers
    let mut headers = HashMap::new();
    let mut values = buffer.as_ref().split(|c| *c == b'\0');
    loop {
        if let Some(key) = values.next() {
            if let Some(val) = values.next() {
                let key = std::str::from_utf8(key)?;
                let val = std::str::from_utf8(val)?;
                headers.insert(key, val);
            } else {
                if !key.is_empty() {
                    return Err(Box::new(GemError::runtime_error("Missing header value")));
                }
                break;
            }
        } else {
            break;
        }
    }

    // Create a Request from the headers
    Ok(Request::parse_request(|k| {
        headers
            .get(k)
            .map(|v| (*v).to_owned())
            .ok_or(GemError::runtime_error(format!("Missing header {k}")))
    })?)
}

async fn send_scgi_response(mut conn: TcpStream, response: Response) {
    println!("{}\t{}", response.code, response.meta);
    if let Err(e) = response.send_async(&mut conn).await {
        eprintln!("Could not send body: {e}");
    }
    if let Err(e) = conn.shutdown().await {
        eprintln!("Could not shutdown connection: {e}");
    };
}

/// Simple Common Gateway Interface
///
/// SCGI is a simplification of the FastCGI protocol. It runs a tcp server where
/// each connection to the server is a single CGI request. This allows for the
/// reduction of time spent on setup/cleanup.
#[async_trait]
pub trait Scgi: Application + Sized + Send + Sync + 'static {
    /// SCGI is a simplification of the FastCGI protocol. It runs a tcp server where
    /// each connection to the server is a single CGI request. This allows for the
    /// reduction of time spent on setup/cleanup.
    ///
    /// addr is the address that the server should listen on. Because this is a
    /// CGI application, it is recommended that this server is not available publicly.
    ///
    /// This is a long running command that generally should not return. If it
    /// does return, the server could not be created.
    ///
    /// ### Example
    ///
    /// ```no_run
    /// use gemfra::{
    ///     protocol::Scgi,
    ///     application::Application,
    ///     request::Request,
    ///     response::Response,
    ///     error::AnyError,
    /// };
    /// use async_trait::async_trait;
    ///
    /// struct MyApp;
    /// #[async_trait]
    /// impl Application for MyApp {
    ///     async fn handle_request(&self, request: Request) -> Result<Response, AnyError> {
    ///         todo!("Handle the request")
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     MyApp.run_scgi("127.0.0.1:8000").await;
    /// }
    /// ```
    async fn run_scgi<A>(self, addr: A) -> io::Result<()>
    where
        A: ToSocketAddrs + Send + Sync,
    {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        println!("Listening to {:?}", listener.local_addr()?);

        let self_arc = Arc::new(self);

        loop {
            let (mut conn, _) = listener.accept().await?;

            let self_ref = self_arc.clone();
            tokio::spawn(async move {
                let response = match read_scgi_request(&mut conn).await {
                    Ok(request) => match self_ref.handle_request(request).await {
                        Ok(response) => response,
                        Err(err) => {
                            eprintln!("Error while handling request: {err}");
                            match err.downcast::<GemError>() {
                                Ok(err) => Response::from(*err),
                                Err(_) => Response::error_cgi("Internal Server Error"),
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Invalid SCGI header: {e}");
                        Response::error_cgi("Invalid CGI header")
                    }
                };

                send_scgi_response(conn, response).await;
            });
        }
    }
}

impl<A> Scgi for A where A: Application + Sized + Send + Sync + 'static {}

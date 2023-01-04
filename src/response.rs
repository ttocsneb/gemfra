//! Gemini Response
//!
//! Send a response using [Response].
//!
//! ### Example
//!
//! ```
//! use gemfra::response::Response;
//!
//! let response = Response::success("text/gemini", "Hello World!");
//! ```
use std::{
    io::{self, Read, Write},
    pin::Pin,
};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use bytes::{Buf, Bytes};

enum ResponseBody {
    Async(Pin<Box<dyn AsyncRead + Send + Sync>>),
    Sync(Box<dyn Read + Send + Sync>),
}

/// Gemini Response
///
/// The gemini response has two parts: A header and a body. The header is made
/// up of a 2 digit response code and a meta string. The body is only allowed if
/// the response code is 2X (success). There are helper functions for each
/// response type:
///
/// * __10__ [input][Response::input] Request for input
/// * __11__ [input_sensitive][Response::input_sensitive] Request for sensitive input
/// * __20__ [success][Response::success] Success with a string buffer body
/// * __20__ [success_sync][Response::success_sync] Success with a synchronous stream body
/// * __20__ [success_async][Response::success_async] Success with an asynchronous stream body
/// * __30__ [redirect][Response::redirect] Redirect to another page
/// * __31__ [redirect_perm][Response::redirect] Redirect to another page
/// * __40__ [error_temp][Response::error_temp] Temporary error
/// * __41__ [unavailable][Response::unavailable] Server unavailable
/// * __42__ [error_cgi][Response::error_cgi] CGI error
/// * __43__ [error_proxy][Response::error_proxy] Unable to fetch proxy
/// * __44__ [slow_down][Response::slow_down] Too many requests
/// * __50__ [error_perm][Response::error_perm] Permanent error
/// * __51__ [not_found][Response::not_found] file not found
/// * __52__ [gone][Response::gone] file no longer exists
/// * __53__ [proxy_refused][Response::proxy_refused] proxies are not allowed
/// * __59__ [bad_request][Response::bad_request] Unable to parse request
/// * __60__ [cert_required][Response::cert_required] Certificate is required
/// * __61__ [cert_not_authorised][Response::cert_not_authorised] Certificate not authorised
/// * __62__ [cert_not_valid][Response::cert_not_valid] Certificate is invalid
pub struct Response {
    pub code: u32,
    pub meta: String,
    body: Option<ResponseBody>,
}

impl Response {
    /// Create a new resposne
    pub fn new(code: u32, meta: impl Into<String>) -> Self {
        Self {
            code,
            meta: meta.into(),
            body: None,
        }
    }

    /// Set the body of the response with a string
    pub fn body(self, body: impl Into<Bytes>) -> Self {
        self.body_sync(Bytes::from(body.into()).reader())
    }

    /// Set the body of the response with a synchronous reader
    pub fn body_sync<R>(mut self, body: R) -> Self
    where
        R: Read + Send + Sync + 'static,
    {
        self.body = Some(ResponseBody::Sync(Box::new(body)));
        self
    }

    /// Set the body of the response with an asynchronous reader
    pub fn body_async<R>(mut self, body: R) -> Self
    where
        R: AsyncRead + Send + Sync + 'static,
    {
        self.body = Some(ResponseBody::Async(Box::pin(body)));
        self
    }

    /// Request for a query input (__10__)
    ///
    /// > The requested resource accepts a line of textual user input. The <META>
    /// > line is a prompt which should be displayed to the user. The same
    /// > resource should then be requested again with the user's input included
    /// > as a query component. Queries are included in requests as per the usual
    /// > generic URL definition in RFC3986, i.e. separated from the path by a ?.
    /// > Reserved characters used in the user's input must be "percent-encoded"
    /// > as per RFC3986, and space characters should also be percent-encoded.
    ///
    #[inline]
    pub fn input(request: impl Into<String>) -> Self {
        Self::new(10, request)
    }
    /// Request for a sensitive query input (__11__)
    ///
    /// > As per status code 10, but for use with sensitive input such as
    /// > passwords. Clients should present the prompt as per status code 10, but
    /// > the user's input should not be echoed to the screen to prevent it being
    /// > read by "shoulder surfers".
    #[inline]
    pub fn input_sensitive(request: impl Into<String>) -> Self {
        Self::new(11, request)
    }
    /// Success response with a string body (__20__)
    ///
    /// > The request was handled successfully and a response body will follow the
    /// > response header. The <META> line is a MIME media type which applies to
    /// > the response body.
    ///
    /// ### Example
    ///
    /// ```
    /// use gemfra::response::Response;
    ///
    /// let reponse = Response::success("text/gemini", "Hello World!");
    /// ```
    #[inline]
    pub fn success(mime: impl Into<String>, body: impl Into<Bytes>) -> Self {
        Self::new(20, mime).body(body)
    }
    /// Success response with a synchronous read body (__20__)
    ///
    /// > The request was handled successfully and a response body will follow the
    /// > response header. The <META> line is a MIME media type which applies to
    /// > the response body.
    ///
    /// ### Example
    ///
    /// ```no_run
    /// # use std::io;
    /// use std::fs::File;
    /// use gemfra::response::Response;
    ///
    /// let file = File::open("index.gmi").unwrap();
    ///
    /// let response = Response::success_sync("text/gemini", file);
    /// # Ok::<(), io::Error>(())
    /// ```
    #[inline]
    pub fn success_sync<M, R>(mime: M, body: R) -> Self
    where
        M: Into<String>,
        R: Read + Send + Sync + 'static,
    {
        Self::new(20, mime).body_sync(body)
    }
    /// Success response with an asynchronous read body (__20__)
    ///
    /// > The request was handled successfully and a response body will follow the
    /// > response header. The <META> line is a MIME media type which applies to
    /// > the response body.
    ///
    /// ### Example
    ///
    /// ```no_run
    /// # use std::io;
    /// use tokio::fs::File;
    /// use gemfra::response::Response;
    ///
    /// # tokio_test::block_on(async {
    /// let file = File::open("index.gmi").await?;
    ///
    /// let response = Response::success_async("text/gemini", file);
    /// # Ok::<(), io::Error>(()) }).unwrap();
    /// ```
    #[inline]
    pub fn success_async<M, R>(mime: M, body: R) -> Self
    where
        M: Into<String>,
        R: AsyncReadExt + Send + Sync + 'static,
    {
        Self::new(20, mime).body_async(body)
    }
    /// Redirect response (__30__)
    ///
    /// > The server is redirecting the client to a new location for the requested
    /// > resource. There is no response body. <META> is a new URL for the
    /// > requested resource. The URL may be absolute or relative. If relative, it
    /// > should be resolved against the URL used in the original request. If the
    /// > URL used in the original request contained a query string, the client
    /// > MUST NOT apply this string to the redirect URL, instead using the
    /// > redirect URL "as is". The redirect should be considered temporary, i.e.
    /// > clients should continue to request the resource at the original address
    /// > and should not perform convenience actions like automatically updating
    /// > bookmarks. There is no response body.
    #[inline]
    pub fn redirect(redirect: impl Into<String>) -> Self {
        Self::new(30, redirect)
    }
    /// Permanent redirect response (__31__)
    ///
    /// > The requested resource should be consistently requested from the new URL
    /// > provided in future. Tools like search engine indexers or content
    /// > aggregators should update their configurations to avoid requesting the
    /// > old URL, and end-user clients may automatically update bookmarks, etc.
    /// > Note that clients which only pay attention to the initial digit of
    /// > status codes will treat this as a temporary redirect. They will still
    /// > end up at the right place, they just won't be able to make use of the
    /// > knowledge that this redirect is permanent, so they'll pay a small
    /// > performance penalty by having to follow the redirect each time.
    #[inline]
    pub fn redirect_perm(redirect: impl Into<String>) -> Self {
        Self::new(31, redirect)
    }
    /// Temporary error response (__40__)
    ///
    /// > The request has failed. There is no response body. The nature of the
    /// > failure is temporary, i.e. an identical request MAY succeed in the
    /// > future. The contents of <META> may provide additional information on the
    /// > failure, and should be displayed to human users.
    #[inline]
    pub fn error_temp(message: impl Into<String>) -> Self {
        Self::new(40, message)
    }
    /// Unavailable response (__41__)
    ///
    /// > The server is unavailable due to overload or maintenance. (cf HTTP 503)
    #[inline]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(41, message)
    }
    /// CGI error response (__42__)
    ///
    /// > A CGI process, or similar system for generating dynamic content, died
    /// > unexpectedly or timed out.
    #[inline]
    pub fn error_cgi(message: impl Into<String>) -> Self {
        Self::new(42, message)
    }
    /// Proxy error response (__43__)
    ///
    /// > A proxy request failed because the server was unable to successfully
    /// > complete a transaction with the remote host. (cf HTTP 502, 504)
    #[inline]
    pub fn error_proxy(message: impl Into<String>) -> Self {
        Self::new(43, message)
    }
    /// Slow down response (__44__)
    ///
    /// > Rate limiting is in effect. <META> is an integer number of seconds which
    /// > the client must wait before another request is made to this server. (cf
    /// > HTTP 429)
    #[inline]
    pub fn slow_down(seconds: u32) -> Self {
        Self::new(44, seconds.to_string())
    }
    /// Permanent error response (__50__)
    ///
    /// > The request has failed. There is no response body. The nature of the
    /// > failure is permanent, i.e. identical future requests will reliably fail
    /// > for the same reason. The contents of <META> may provide additional
    /// > information on the failure, and should be displayed to human users.
    /// > Automatic clients such as aggregators or indexing crawlers should not
    /// > repeat this request.
    #[inline]
    pub fn error_perm(message: impl Into<String>) -> Self {
        Self::new(50, message)
    }
    /// Not found response (__51__)
    ///
    /// > The requested resource could not be found but may be available in the
    /// > future. (cf HTTP 404) (struggling to remember this important status
    /// > code? Easy: you can't find things hidden at Area 51!)
    #[inline]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(51, message)
    }
    /// Gone response (__52__)
    ///
    /// > The resource requested is no longer available and will not be available
    /// > again. Search engines and similar tools should remove this resource from
    /// > their indices. Content aggregators should stop requesting the resource
    /// > and convey to their human users that the subscribed resource is gone. (
    /// > cf HTTP 410)
    #[inline]
    pub fn gone(message: impl Into<String>) -> Self {
        Self::new(52, message)
    }
    /// Proxy Refused response (__53__)
    ///
    /// > The request was for a resource at a domain not served by the server and
    /// > the server does not accept proxy requests.
    #[inline]
    pub fn proxy_refused(message: impl Into<String>) -> Self {
        Self::new(53, message)
    }
    /// Bad Request response (__59__)
    ///
    /// > The server was unable to parse the client's request, presumably due to a
    /// > malformed request. (cf HTTP 400)
    #[inline]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(59, message)
    }
    /// Cert Required response (__60__)
    ///
    /// > The requested resource requires a client certificate to access. If the
    /// > request was made without a certificate, it should be repeated with one.
    /// > If the request was made with a certificate, the server did not accept it
    /// > and the request should be repeated with a different certificate. The
    /// > contents of <META> (and/or the specific 6x code) may provide additional
    /// > information on certificate requirements or the reason a certificate was
    /// > rejected.
    #[inline]
    pub fn cert_required(message: impl Into<String>) -> Self {
        Self::new(60, message)
    }
    /// Cert Not Authorised response (__61__)
    ///
    /// > The supplied client certificate is not authorised for accessing the
    /// > particular requested resource. The problem is not with the certificate
    /// > itself, which may be authorised for other resources.
    #[inline]
    pub fn cert_not_authorised(message: impl Into<String>) -> Self {
        Self::new(61, message)
    }
    /// Cert Not Valid response (__62__)
    ///
    /// > The supplied client certificate was not accepted because it is not
    /// > valid. This indicates a problem with the certificate in and of itself,
    /// > with no consideration of the particular requested resource. The most
    /// > likely cause is that the certificate's validity start date is in the
    /// > future or its expiry date has passed, but this code may also indicate an
    /// > invalid signature, or a violation of X509 standard requirements. The
    /// > <META> should provide more information about the exact error.
    #[inline]
    pub fn cert_not_valid(message: impl Into<String>) -> Self {
        Self::new(62, message)
    }

    /// Get the full header for this response
    pub fn header(&self) -> String {
        format!("{} {}\r\n", self.code, self.meta)
    }

    /// Send the response to an async stream
    pub(crate) async fn send_async<W>(self, writer: &mut W) -> Result<(), io::Error>
    where
        W: AsyncWrite + Unpin + ?Sized,
    {
        let header = self.header();
        writer.write_all(header.as_bytes()).await?;

        match self.body {
            Some(ResponseBody::Async(mut reader)) => {
                tokio::io::copy(&mut reader, writer).await?;
            }
            Some(ResponseBody::Sync(mut reader)) => {
                let mut buf = [0; 1024];

                loop {
                    let read = reader.read(&mut buf)?;
                    if read == 0 {
                        break;
                    }
                    writer.write_all(&buf[..read]).await?;
                }
            }
            None => {}
        }

        Ok(())
    }

    /// Send the response to a sync stream
    pub(crate) async fn send_sync<W>(self, writer: &mut W) -> Result<(), io::Error>
    where
        W: Write + ?Sized,
    {
        let header = self.header();
        writer.write_all(header.as_bytes())?;

        match self.body {
            Some(ResponseBody::Async(mut reader)) => {
                let mut buf = [0; 1024];

                loop {
                    let read = reader.read(&mut buf).await?;
                    if read == 0 {
                        break;
                    }
                    writer.write_all(&buf[..read])?;
                }
            }
            Some(ResponseBody::Sync(mut reader)) => {
                io::copy(&mut reader, writer)?;
            }
            None => {}
        };

        Ok(())
    }
}

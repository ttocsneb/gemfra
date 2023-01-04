//! Custom error helpers
//!
//! The [GemError] is used for custom error responses when an unrecoverable error
//! occurs.
//!
//! [ToGemError] has been created to help convert any error into a [GemError]. This
//! can be useful for returning error responses other than __42__ CGI Error.

use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::response::Response;

/// Shorthand for a boxed error
pub type AnyError = Box<dyn Error + Send + Sync>;

/// Convert a struct into a [GemError] result.
///
/// This is implemented already for [Result](std::result::Result) and
/// [Option](std::option::Option) so you can easily convert an error into a GemError.
pub trait ToGemError<T> {
    /// Convert the object into a GemError Result.
    ///
    /// By default this will be converted into a [RuntimeError](GemErrorType::RuntimeError).
    ///
    /// ### Example
    ///
    /// ```no_run
    /// # use gemfra::error::GemError;
    /// use gemfra::error::ToGemError;
    /// use std::fs::File;
    ///
    /// let file = File::open("foo.txt").into_gem()?;
    /// // On failure, the response would be `42 Internal Server Error`
    /// # Ok::<(), GemError>(())
    /// ```
    ///
    fn into_gem(self) -> Result<T, GemError>;
    /// Convert the object into a GemError Result overriding the default error type.
    ///
    /// ### Example
    ///
    /// ```no_run
    /// # use gemfra::error::GemError;
    /// use gemfra::error::GemErrorType;
    /// use gemfra::error::ToGemError;
    /// use std::fs::File;
    ///
    /// let file = File::open("foo.txt").into_gem_type(GemErrorType::NotFound)?;
    /// // On failure, the response would be `51 File not found`
    /// # Ok::<(), GemError>(())
    /// ```
    fn into_gem_type(self, error_type: GemErrorType) -> Result<T, GemError>;
    /// Replace the object with a GemError Result.
    ///
    /// ### Example
    ///
    /// ```no_run
    /// # use gemfra::error::GemError;
    /// use gemfra::error::GemErrorType;
    /// use gemfra::error::ToGemError;
    /// use std::fs::File;
    ///
    /// let file = File::open("foo.txt").replace_gem(
    ///     GemErrorType::NotFound,
    ///     "The file doesn't exist"
    /// )?;
    /// // On failure, the response would be `51 The file doesn't exist`
    /// # Ok::<(), GemError>(())
    /// ```
    fn replace_gem(self, error_type: GemErrorType, msg: impl Into<String>) -> Result<T, GemError>;
}

impl<T, E> ToGemError<T> for Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    fn into_gem(self) -> Result<T, GemError> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(GemError::from_err(GemErrorType::RuntimeError, e)),
        }
    }

    fn into_gem_type(self, error_type: GemErrorType) -> Result<T, GemError> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(GemError::from_err(error_type, e)),
        }
    }

    fn replace_gem(self, error_type: GemErrorType, msg: impl Into<String>) -> Result<T, GemError> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => Err(GemError::new(error_type, msg)),
        }
    }
}

impl<T> ToGemError<T> for Option<T> {
    fn into_gem(self) -> Result<T, GemError> {
        match self {
            Some(v) => Ok(v),
            None => Err(GemError::new(GemErrorType::RuntimeError, "Cannot be None")),
        }
    }

    fn into_gem_type(self, error_type: GemErrorType) -> Result<T, GemError> {
        match self {
            Some(v) => Ok(v),
            None => Err(GemError::new(error_type, "Cannot be None")),
        }
    }

    fn replace_gem(self, error_type: GemErrorType, msg: impl Into<String>) -> Result<T, GemError> {
        match self {
            Some(v) => Ok(v),
            None => Err(GemError::new(error_type, msg)),
        }
    }
}

/// The type of error that has happened
///
/// These errors corespond to gemini response codes and are used to determine
/// the type of response that is sent.
#[derive(Debug, PartialEq, Eq)]
pub enum GemErrorType {
    /// __40__ Temporary Error
    TempError,
    /// __50__ Permanent Error
    PermError,
    /// __41__ Server Unavailable
    Unavailable,
    /// __42__ CGI Error
    RuntimeError,
    /// __43__ Unable to fetch Proxy
    ProxyError,
    /// __44__ Too many requests -- The message should be the number of seconds
    /// before another request is made
    TooManyRequests,
    /// __51__ File not found
    NotFound,
    /// __52__ File no longer exists
    Gone,
    /// __53__ Proxy requests are not allowed
    ProxyRefused,
    /// __59__ Unable to parse request
    BadRequest,
    /// __60__ A certificate is needed
    CertNeeded,
    /// __61__ The certificate was not authorised
    CertUnAuthorised,
    /// __62__ The certificate is invalid
    BadCert,
}

impl Display for GemErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GemErrorType::TempError => "Temporary Error",
            GemErrorType::PermError => "Permanent Error",
            GemErrorType::Unavailable => "Server Unavailable",
            GemErrorType::RuntimeError => "Internal Server Error",
            GemErrorType::ProxyError => "Proxy Error",
            GemErrorType::TooManyRequests => "10",
            GemErrorType::NotFound => "File not found",
            GemErrorType::Gone => "File no longer exists",
            GemErrorType::ProxyRefused => "Proxies are not allowed",
            GemErrorType::BadRequest => "Invalid Request",
            GemErrorType::CertNeeded => "Certificate needed",
            GemErrorType::CertUnAuthorised => "Certificate not Authorised",
            GemErrorType::BadCert => "Invalid Certificate",
        })
    }
}

/// The message of the error.
///
/// This can be either an embedded error, or a string
#[derive(Debug)]
enum GemErrorMsg {
    Error(AnyError),
    Message(String),
}

impl Display for GemErrorMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GemErrorMsg::Error(err) => Display::fmt(&err, f),
            GemErrorMsg::Message(msg) => f.write_str(msg),
        }
    }
}

/// A custom error type for gemfra
///
/// This error can be converted into a gemini [Response](crate::response::Response).
///
/// The error consists of an error type and a message. The message can be either
/// an embedded error or a string.
#[derive(Debug)]
pub struct GemError {
    pub error_type: GemErrorType,
    msg: GemErrorMsg,
}

impl Error for GemError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.msg {
            GemErrorMsg::Error(err) => Some(err.as_ref()),
            GemErrorMsg::Message(_) => None,
        }
    }
}

impl Display for GemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.error_type, f)?;
        f.write_str(": ")?;

        match self.error_type {
            GemErrorType::TooManyRequests => f.write_fmt(format_args!("{} seconds", self.msg)),
            _ => Display::fmt(&self.msg, f),
        }
    }
}

impl From<GemError> for Response {
    fn from(err: GemError) -> Self {
        let message = match err.msg {
            GemErrorMsg::Error(_) => err.error_type.to_string(),
            GemErrorMsg::Message(msg) => msg,
        };
        match err.error_type {
            GemErrorType::TempError => Response::error_temp(message),
            GemErrorType::PermError => Response::error_perm(message),
            GemErrorType::Unavailable => Response::unavailable(message),
            GemErrorType::RuntimeError => Response::error_cgi(message),
            GemErrorType::ProxyError => Response::error_proxy(message),
            GemErrorType::TooManyRequests => {
                let seconds = match message.parse() {
                    Ok(val) => val,
                    Err(_) => {
                        eprintln!(
                            "Unable to parse TooManyRequests delay, defaulting to 10 seconds"
                        );
                        10
                    }
                };
                Response::slow_down(seconds)
            }
            GemErrorType::NotFound => Response::not_found(message),
            GemErrorType::Gone => Response::gone(message),
            GemErrorType::ProxyRefused => Response::proxy_refused(message),
            GemErrorType::BadRequest => Response::bad_request(message),
            GemErrorType::CertNeeded => Response::cert_required(message),
            GemErrorType::CertUnAuthorised => Response::cert_not_authorised(message),
            GemErrorType::BadCert => Response::cert_not_valid(message),
        }
    }
}

impl GemError {
    /// Create a new error using a string message
    #[inline]
    pub fn new(error_type: GemErrorType, msg: impl Into<String>) -> Self {
        Self {
            error_type,
            msg: GemErrorMsg::Message(msg.into()),
        }
    }
    /// Create a new error using an existing error
    #[inline]
    pub fn from_err<E>(error_type: GemErrorType, msg: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self {
            error_type,
            msg: GemErrorMsg::Error(Box::new(msg)),
        }
    }

    /// Get the error message as a string
    pub fn message(&self) -> String {
        self.msg.to_string()
    }

    /// Map an existing error into a GemError
    pub fn map<R, E>(error_type: GemErrorType, result: Result<R, E>) -> Result<R, Self>
    where
        E: Error + Send + Sync + 'static,
    {
        match result {
            Ok(val) => Ok(val),
            Err(err) => Err(Self::from_err(error_type, err)),
        }
    }

    #[inline]
    pub fn temp_error(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::TempError, msg)
    }

    #[inline]
    pub fn perm_error(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::PermError, msg)
    }

    #[inline]
    pub fn unavailable(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::Unavailable, msg)
    }

    #[inline]
    pub fn runtime_error(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::RuntimeError, msg)
    }

    #[inline]
    pub fn proxy_error(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::ProxyError, msg)
    }

    #[inline]
    pub fn too_many_requests(timeout: u32) -> Self {
        Self::new(GemErrorType::TooManyRequests, timeout.to_string())
    }

    #[inline]
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::NotFound, msg)
    }

    #[inline]
    pub fn gone(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::Gone, msg)
    }

    #[inline]
    pub fn proxy_refused(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::ProxyRefused, msg)
    }

    #[inline]
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::BadRequest, msg)
    }

    #[inline]
    pub fn cert_needed(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::CertNeeded, msg)
    }

    #[inline]
    pub fn cert_unauthorised(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::CertUnAuthorised, msg)
    }

    #[inline]
    pub fn bad_cert(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::BadCert, msg)
    }
}

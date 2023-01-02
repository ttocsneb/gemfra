//! Custom error helpers
//!
//! The [GemError] is used for custom error responses when an unrecoverable error
//! occurs.
//!
//! A helper trait will be created that can convert any error into a [GemError].

use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::response::Response;

/// The type of error that has happened
///
/// TODO: There should be more error types with better names.
#[derive(Debug, PartialEq, Eq)]
pub enum GemErrorType {
    /// Bad parameters have been provided to the request
    BadParams,
    /// Invalid certificate
    BadCert,
}

impl Display for GemErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GemErrorType::BadParams => "Bad Params",
            GemErrorType::BadCert => "Bad Certificate",
        })
    }
}

/// The message of the error.
///
/// This can be either an embedded error, or a string
#[derive(Debug)]
enum GemErrorMsg {
    Error(Box<dyn Error + Send + Sync + 'static>),
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
///
/// Helper functions are provided to create a GemError, so [GemErrorType] is
/// generally not needed.
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
        f.write_fmt(format_args!("{}: {}", self.error_type, self.msg))
    }
}

impl From<GemError> for Response {
    fn from(err: GemError) -> Self {
        let msg = err.message();
        match err.error_type {
            GemErrorType::BadParams => Response::bad_request(msg),
            GemErrorType::BadCert => Response::cert_not_valid(msg),
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
    pub fn new_err<E>(error_type: GemErrorType, msg: E) -> Self
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
            Err(err) => Err(Self::new_err(error_type, err)),
        }
    }

    /// Create a [BadParams](GemErrorType::BadParams) Error using a string
    #[inline]
    pub fn bad_params(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::BadParams, msg)
    }
    /// Create a [BadParams](GemErrorType::BadParams) Error using an error
    #[inline]
    pub fn bad_params_err<E>(err: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::new_err(GemErrorType::BadParams, err)
    }

    /// Create a [BadCert](GemErrorType::BadCert) Error using a string
    #[inline]
    pub fn bad_cert(msg: impl Into<String>) -> Self {
        Self::new(GemErrorType::BadCert, msg)
    }
    /// Create a [BadCert](GemErrorType::BadCert) Error using an error
    #[inline]
    pub fn bad_cert_err<E>(err: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::new_err(GemErrorType::BadCert, err)
    }
}

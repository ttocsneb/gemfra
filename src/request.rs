//! Gemini Request
//!
//! The gemini request contains all the information needed to handle a request.

use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};

use crate::error::{GemError, ToGemError};

/// Parse an X.509 Name into a hashmap.
fn parse_client_name(name: impl AsRef<str>) -> Result<HashMap<String, String>, GemError> {
    let mut mapping = HashMap::new();
    for group in name.as_ref().split(',') {
        if let Some((k, v)) = group.split_once('=') {
            mapping.insert(k.to_owned(), v.to_owned());
        } else {
            return Err(GemError::bad_cert("Invalid X.509 Name"));
        }
    }
    Ok(mapping)
}

/// Client Certificate
///
/// [hash][Certificate::hash] is the primary identifyer for the certificate, you
/// can get information about the certificate from [subject](Certificate::subject),
/// and you can determine wether the certificate is valid if the date is between
/// [not_before](Certificate::not_before) and [not_after](Certificate::not_after).
pub struct Certificate {
    /// The identifying token for the certificate
    pub hash: String,
    /// Information about the issuing certificate. This should be the same as
    /// subject. subject is prefered
    pub issuer: HashMap<String, String>,
    /// Information about the certificate.
    pub subject: HashMap<String, String>,
    /// The time when the certificate expires
    pub not_after: DateTime<FixedOffset>,
    /// The time when the certificate was created
    pub not_before: DateTime<FixedOffset>,
}

impl Certificate {
    pub fn parse_cert<F>(get_var: F) -> Result<Self, GemError>
    where
        F: Fn(&str) -> Result<String, GemError>,
    {
        let hash = get_var("TLS_CLIENT_HASH")?;
        let issuer = get_var("TLS_CLIENT_ISSUER")?;
        let subject = get_var("TLS_CLIENT_SUBJECT")?;
        let not_after = get_var("TLS_CLIENT_NOT_AFTER")?;
        let not_before = get_var("TLS_CLIENT_NOT_BEFORE")?;
        let not_after = DateTime::parse_from_rfc3339(&not_after).unwrap();
        let not_before = DateTime::parse_from_rfc3339(&not_before).unwrap();
        Ok(Self {
            hash,
            not_before,
            not_after,
            issuer: parse_client_name(issuer)?,
            subject: parse_client_name(subject)?,
        })
    }
}

/// Information about a request
pub struct Request {
    /// URL Path relative to the script
    pub path: String,
    /// URL Path of the script
    pub script: String,
    /// Query component of the URL
    pub query: String,
    /// Server component of the URL
    pub server_name: String,
    /// Port component of the URL
    pub server_port: u16,
    /// Full URL
    pub url: String,
    /// IP address of the client
    pub remote_addr: String,
    /// FQDN of the client (if unresolvable, will be the same as remote_addr)
    pub remote_host: String,
    /// The protocol of the URL (should always be "GEMINI")
    pub protocol: String,
    /// The client certificate if one was provided
    pub client_cert: Option<Certificate>,
}

impl Request {
    pub fn parse_request<F>(get_var: F) -> Result<Self, GemError>
    where
        F: Fn(&str) -> Result<String, GemError>,
    {
        let path = get_var("PATH_INFO")?;
        let script = get_var("SCRIPT_NAME")?;
        let server = get_var("SERVER_NAME")?;
        let query = get_var("QUERY_STRING")?;
        let port: u16 = get_var("SERVER_PORT")?.parse().into_gem()?;
        let url = get_var("GEMINI_URL")?;
        let remote_addr = get_var("REMOTE_ADDR")?;
        let remote_host = get_var("REMOTE_HOST")?;
        let protocol = get_var("SERVER_PROTOCOL")?;

        let cert = if get_var("AUTH_TYPE").unwrap_or("".to_owned()) == "CERTIFICATE" {
            Some(Certificate::parse_cert(get_var)?)
        } else {
            None
        };

        Ok(Self {
            path,
            script,
            query,
            server_name: server,
            server_port: port,
            url,
            remote_addr,
            remote_host,
            protocol,
            client_cert: cert,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::error::GemErrorType;

    use super::*;

    #[test]
    fn test_client_name_parse() {
        let parsed = parse_client_name("CN=foobar").unwrap();
        assert_eq!(parsed.get("CN").unwrap(), "foobar");

        let parsed = parse_client_name("CN=foobar,OU=cheese").unwrap();
        assert_eq!(parsed.get("CN").unwrap(), "foobar");
        assert_eq!(parsed.get("OU").unwrap(), "cheese");

        let err = parse_client_name("CN").expect_err("Expected Error");
        assert_eq!(err.error_type, GemErrorType::BadCert);
    }
}

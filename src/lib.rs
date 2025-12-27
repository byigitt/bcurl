//! bcurl - A blazingly fast, minimal HTTP client library in Rust
//!
//! This library provides basic HTTP functionality similar to curl.
//! Uses ureq for minimal binary size and fast startup.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use ureq::{Agent, AgentBuilder};

/// Custom error types for minimal-curl
#[derive(Error, Debug)]
pub enum CurlError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] ureq::Error),

    #[error("Invalid header format: {0}")]
    InvalidHeader(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

/// HTTP methods supported by minimal-curl
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Patch,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Patch => write!(f, "PATCH"),
        }
    }
}

/// Configuration for an HTTP request
#[derive(Debug, Clone)]
pub struct RequestConfig {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>, // Vec is faster than HashMap for small collections
    pub data: Option<String>,
    pub timeout: Option<Duration>,
    pub follow_redirects: bool,
    pub verbose: bool,
    pub output_file: Option<String>,
    pub include_headers: bool,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: HttpMethod::Get,
            headers: Vec::with_capacity(8), // Pre-allocate for common case
            data: None,
            timeout: Some(Duration::from_secs(30)),
            follow_redirects: true,
            verbose: false,
            output_file: None,
            include_headers: false,
        }
    }
}

impl RequestConfig {
    /// Create a new RequestConfig with the given URL
    #[inline]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set the HTTP method
    #[inline]
    pub fn method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    /// Add a header to the request
    #[inline]
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Set the request body data
    #[inline]
    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Set the request timeout
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set whether to follow redirects
    #[inline]
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    /// Set verbose mode
    #[inline]
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set output file path
    #[inline]
    pub fn output_file(mut self, path: impl Into<String>) -> Self {
        self.output_file = Some(path.into());
        self
    }

    /// Set whether to include headers in output
    #[inline]
    pub fn include_headers(mut self, include: bool) -> Self {
        self.include_headers = include;
        self
    }
}

/// Response from an HTTP request
#[derive(Debug)]
pub struct CurlResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl CurlResponse {
    /// Check if the response status indicates success (2xx)
    #[inline]
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Get a specific header value
    #[inline]
    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }
}

/// The main HTTP client
pub struct MinimalCurl {
    agent: Agent,
}

impl Default for MinimalCurl {
    fn default() -> Self {
        Self::new()
    }
}

impl MinimalCurl {
    /// Create a new MinimalCurl client with default settings
    pub fn new() -> Self {
        Self::with_config(true, Duration::from_secs(30))
    }

    /// Create a new MinimalCurl client with custom configuration
    pub fn with_config(follow_redirects: bool, timeout: Duration) -> Self {
        // Create native-tls connector
        let tls = native_tls::TlsConnector::new()
            .expect("Failed to create TLS connector");

        let mut builder = AgentBuilder::new()
            .tls_connector(Arc::new(tls))
            .timeout(timeout)
            .user_agent("bcurl/0.2.0");

        if follow_redirects {
            builder = builder.redirects(10);
        } else {
            builder = builder.redirects(0);
        }

        Self {
            agent: builder.build(),
        }
    }

    /// Execute an HTTP request with the given configuration
    pub fn execute(&self, config: &RequestConfig) -> Result<CurlResponse, CurlError> {
        if config.url.is_empty() {
            return Err(CurlError::InvalidUrl("URL cannot be empty".to_string()));
        }

        // Print verbose request information
        if config.verbose {
            eprintln!("> {} {}", config.method, config.url);
            for (key, value) in &config.headers {
                eprintln!("> {}: {}", key, value);
            }
            eprintln!(">");
        }

        // Build the request based on method
        let mut request = match config.method {
            HttpMethod::Get => self.agent.get(&config.url),
            HttpMethod::Post => self.agent.post(&config.url),
            HttpMethod::Put => self.agent.put(&config.url),
            HttpMethod::Delete => self.agent.delete(&config.url),
            HttpMethod::Head => self.agent.head(&config.url),
            HttpMethod::Patch => self.agent.request("PATCH", &config.url),
        };

        // Add headers
        for (key, value) in &config.headers {
            request = request.set(key, value);
        }

        // Set timeout if different from default
        if let Some(timeout) = config.timeout {
            request = request.timeout(timeout);
        }

        // Execute the request - handle both success and HTTP error status codes
        let response = if let Some(ref data) = config.data {
            match request.send_string(data) {
                Ok(resp) => resp,
                Err(ureq::Error::Status(_code, resp)) => resp, // HTTP errors are still valid responses
                Err(e) => return Err(CurlError::RequestError(e)),
            }
        } else {
            match request.call() {
                Ok(resp) => resp,
                Err(ureq::Error::Status(_code, resp)) => resp, // HTTP errors are still valid responses
                Err(e) => return Err(CurlError::RequestError(e)),
            }
        };

        // Extract response information
        let status = response.status();
        let status_text = response.status_text().to_string();

        // Extract headers - pre-allocate with estimated capacity
        let header_names: Vec<_> = response.headers_names();
        let mut headers = HashMap::with_capacity(header_names.len());
        for name in header_names {
            if let Some(value) = response.header(&name) {
                headers.insert(name.to_lowercase(), value.to_string());
            }
        }

        // Print verbose response information
        if config.verbose {
            eprintln!("< HTTP/1.1 {} {}", status, status_text);
            for (key, value) in &headers {
                eprintln!("< {}: {}", key, value);
            }
            eprintln!("<");
        }

        // Read body efficiently
        let body = if config.method == HttpMethod::Head {
            String::new()
        } else {
            // Pre-allocate buffer based on content-length if available
            let content_length = headers
                .get("content-length")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(4096);

            let mut body = String::with_capacity(content_length);
            response.into_reader().read_to_string(&mut body)?;
            body
        };

        // Write to file if specified
        if let Some(ref path) = config.output_file {
            let mut file = File::create(path)?;
            if config.include_headers {
                writeln!(file, "HTTP/1.1 {} {}", status, status_text)?;
                for (key, value) in &headers {
                    writeln!(file, "{}: {}", key, value)?;
                }
                writeln!(file)?;
            }
            file.write_all(body.as_bytes())?;
        }

        Ok(CurlResponse {
            status,
            status_text,
            headers,
            body,
        })
    }

    /// Convenience method for GET requests
    #[inline]
    pub fn get(&self, url: &str) -> Result<CurlResponse, CurlError> {
        let config = RequestConfig::new(url);
        self.execute(&config)
    }

    /// Convenience method for POST requests
    #[inline]
    pub fn post(&self, url: &str, data: Option<&str>) -> Result<CurlResponse, CurlError> {
        let mut config = RequestConfig::new(url).method(HttpMethod::Post);
        if let Some(d) = data {
            config = config.data(d);
        }
        self.execute(&config)
    }

    /// Convenience method for PUT requests
    #[inline]
    pub fn put(&self, url: &str, data: Option<&str>) -> Result<CurlResponse, CurlError> {
        let mut config = RequestConfig::new(url).method(HttpMethod::Put);
        if let Some(d) = data {
            config = config.data(d);
        }
        self.execute(&config)
    }

    /// Convenience method for DELETE requests
    #[inline]
    pub fn delete(&self, url: &str) -> Result<CurlResponse, CurlError> {
        let config = RequestConfig::new(url).method(HttpMethod::Delete);
        self.execute(&config)
    }
}

/// Parse a header string in the format "Key: Value"
#[inline]
pub fn parse_header(header: &str) -> Result<(String, String), CurlError> {
    let parts: Vec<&str> = header.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(CurlError::InvalidHeader(format!(
            "Header must be in format 'Key: Value', got: {}",
            header
        )));
    }
    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_config_default() {
        let config = RequestConfig::default();
        assert!(config.url.is_empty());
        assert_eq!(config.method, HttpMethod::Get);
        assert!(config.headers.is_empty());
        assert!(config.data.is_none());
        assert!(config.follow_redirects);
        assert!(!config.verbose);
    }

    #[test]
    fn test_request_config_builder() {
        let config = RequestConfig::new("https://example.com")
            .method(HttpMethod::Post)
            .header("Content-Type", "application/json")
            .data(r#"{"key": "value"}"#)
            .verbose(true)
            .follow_redirects(false);

        assert_eq!(config.url, "https://example.com");
        assert_eq!(config.method, HttpMethod::Post);
        assert_eq!(config.headers.len(), 1);
        assert_eq!(config.headers[0].0, "Content-Type");
        assert_eq!(config.headers[0].1, "application/json");
        assert_eq!(config.data, Some(r#"{"key": "value"}"#.to_string()));
        assert!(config.verbose);
        assert!(!config.follow_redirects);
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(format!("{}", HttpMethod::Get), "GET");
        assert_eq!(format!("{}", HttpMethod::Post), "POST");
        assert_eq!(format!("{}", HttpMethod::Put), "PUT");
        assert_eq!(format!("{}", HttpMethod::Delete), "DELETE");
        assert_eq!(format!("{}", HttpMethod::Head), "HEAD");
        assert_eq!(format!("{}", HttpMethod::Patch), "PATCH");
    }

    #[test]
    fn test_parse_header_valid() {
        let (key, value) = parse_header("Content-Type: application/json").unwrap();
        assert_eq!(key, "Content-Type");
        assert_eq!(value, "application/json");
    }

    #[test]
    fn test_parse_header_with_spaces() {
        let (key, value) = parse_header("  Accept  :  text/html  ").unwrap();
        assert_eq!(key, "Accept");
        assert_eq!(value, "text/html");
    }

    #[test]
    fn test_parse_header_invalid() {
        let result = parse_header("InvalidHeader");
        assert!(result.is_err());
    }

    #[test]
    fn test_curl_response_is_success() {
        let response = CurlResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: String::new(),
        };
        assert!(response.is_success());

        let response = CurlResponse {
            status: 404,
            status_text: "Not Found".to_string(),
            headers: HashMap::new(),
            body: String::new(),
        };
        assert!(!response.is_success());
    }

    #[test]
    fn test_curl_response_get_header() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let response = CurlResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::new(),
        };

        assert_eq!(
            response.get_header("content-type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(response.get_header("nonexistent"), None);
    }

    #[test]
    fn test_minimal_curl_empty_url() {
        let client = MinimalCurl::new();
        let config = RequestConfig::default();
        let result = client.execute(&config);
        assert!(result.is_err());
    }
}

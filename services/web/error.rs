/*
 * Nuva OS - SystemService - Web - Error Model
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Web service specific error types and data definitions.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// Web service error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebError {
    /// Network request failed
    NetworkError = 0,
    /// HTML/CSS/JS parse error
    ParseError = 1,
    /// JavaScript execution timed out
    JsTimeout = 2,
    /// Memory limit exceeded for page
    MemoryLimitExceeded = 3,
    /// Cross-origin access denied by same-origin policy
    CrossOriginDenied = 4,
    /// Operation requires secure context (HTTPS)
    InsecureContextRequired = 5,
    /// HTTP cache read/write error
    CacheError = 6,
    /// Requested resource not found
    ResourceNotFound = 7,
    /// Service not initialized
    NotInitialized = 8,
    /// Invalid argument
    InvalidArgument = 9,
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebError::NetworkError => write!(f, "Network error"),
            WebError::ParseError => write!(f, "Parse error"),
            WebError::JsTimeout => write!(f, "JavaScript execution timeout"),
            WebError::MemoryLimitExceeded => write!(f, "Memory limit exceeded"),
            WebError::CrossOriginDenied => write!(f, "Cross-origin access denied"),
            WebError::InsecureContextRequired => write!(f, "Secure context required"),
            WebError::CacheError => write!(f, "Cache error"),
            WebError::ResourceNotFound => write!(f, "Resource not found"),
            WebError::NotInitialized => write!(f, "Web service not initialized"),
            WebError::InvalidArgument => write!(f, "Invalid argument"),
        }
    }
}

/// URL representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    /// URL scheme (http, https, file, data, blob)
    pub scheme: String,
    /// Host name
    pub host: String,
    /// Port number (0 means default for scheme)
    pub port: u16,
    /// Path component
    pub path: String,
    /// Query string (without leading '?')
    pub query: String,
    /// Fragment identifier (without leading '#')
    pub fragment: String,
}

impl Url {
    /// Create a URL from components
    pub fn new(scheme: String, host: String, port: u16, path: String) -> Self {
        Url {
            scheme,
            host,
            port,
            path,
            query: String::new(),
            fragment: String::new(),
        }
    }

    /// Parse a URL string into a Url struct
    pub fn parse(raw: &str) -> Result<Self, WebError> {
        let scheme_end = raw.find("://").ok_or(WebError::ParseError)?;
        let scheme = String::from(&raw[..scheme_end]);
        let rest = &raw[scheme_end + 3..];

        let (authority, path_query_frag) = if let Some(slash_pos) = rest.find('/') {
            (&rest[..slash_pos], &rest[slash_pos..])
        } else {
            (rest, "/")
        };

        let (host, port) = if let Some(colon_pos) = authority.rfind(':') {
            let h = String::from(&authority[..colon_pos]);
            let p = authority[colon_pos + 1..].parse::<u16>().map_err(|_| WebError::ParseError)?;
            (h, p)
        } else {
            (String::from(authority), 0)
        };

        let path = String::from(path_query_frag);
        Ok(Url {
            scheme,
            host,
            port,
            path,
            query: String::new(),
            fragment: String::new(),
        })
    }

    /// Get the origin string (scheme://host:port)
    pub fn origin(&self) -> String {
        let mut s = String::new();
        s.push_str(&self.scheme);
        s.push_str("://");
        s.push_str(&self.host);
        if self.port != 0 {
            s.push(':');
            let port_str = alloc::fmt::format(format_args!("{}", self.port));
            s.push_str(&port_str);
        }
        s
    }

    /// Check if this URL uses a secure scheme (https, wss)
    pub fn is_secure(&self) -> bool {
        self.scheme == "https" || self.scheme == "wss"
    }
}

/// HTTP status code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpStatus(pub u16);

impl HttpStatus {
    /// HTTP 200 OK
    pub const OK: HttpStatus = HttpStatus(200);
    /// HTTP 301 Moved Permanently
    pub const MOVED_PERMANENTLY: HttpStatus = HttpStatus(301);
    /// HTTP 302 Found
    pub const FOUND: HttpStatus = HttpStatus(302);
    /// HTTP 304 Not Modified
    pub const NOT_MODIFIED: HttpStatus = HttpStatus(304);
    /// HTTP 400 Bad Request
    pub const BAD_REQUEST: HttpStatus = HttpStatus(400);
    /// HTTP 403 Forbidden
    pub const FORBIDDEN: HttpStatus = HttpStatus(403);
    /// HTTP 404 Not Found
    pub const NOT_FOUND: HttpStatus = HttpStatus(404);
    /// HTTP 500 Internal Server Error
    pub const INTERNAL_ERROR: HttpStatus = HttpStatus(500);
}

impl HttpStatus {
    /// Check if this is a success status (2xx)
    pub fn is_success(&self) -> bool {
        self.0 >= 200 && self.0 < 300
    }

    /// Check if this is a redirect status (3xx)
    pub fn is_redirect(&self) -> bool {
        self.0 >= 300 && self.0 < 400
    }
}

/// Page loading configuration
#[derive(Debug, Clone)]
pub struct PageConfig {
    /// Maximum memory budget in bytes for this page
    pub memory_budget: u64,
    /// JavaScript execution timeout in microseconds
    pub js_timeout_us: u64,
    /// Maximum JS heap size in bytes
    pub js_heap_limit: u64,
    /// Whether JavaScript is enabled
    pub js_enabled: bool,
    /// Whether to load images
    pub load_images: bool,
    /// Whether to load CSS stylesheets
    pub load_css: bool,
    /// User agent string
    pub user_agent: String,
    /// Viewport width in pixels
    pub viewport_width: u32,
    /// Viewport height in pixels
    pub viewport_height: u32,
}

impl PageConfig {
    /// Create a default page configuration
    pub fn default_config() -> Self {
        PageConfig {
            memory_budget: 64 * 1024 * 1024,
            js_timeout_us: 5_000_000,
            js_heap_limit: 32 * 1024 * 1024,
            js_enabled: true,
            load_images: true,
            load_css: true,
            user_agent: String::from("NuvaWeb/1.0"),
            viewport_width: 1920,
            viewport_height: 1080,
        }
    }
}

/// JavaScript value representation
#[derive(Debug, Clone, PartialEq)]
pub enum JsValue {
    /// JavaScript undefined
    Undefined,
    /// JavaScript null
    Null,
    /// JavaScript boolean
    Bool(bool),
    /// JavaScript number (f64)
    Number(f64),
    /// JavaScript string
    String(String),
    /// JavaScript array
    Array(Vec<JsValue>),
    /// JavaScript object (key-value pairs)
    Object(Vec<(String, JsValue)>),
}

impl JsValue {
    /// Check if this value is truthy per JavaScript semantics
    pub fn is_truthy(&self) -> bool {
        match self {
            JsValue::Undefined | JsValue::Null => false,
            JsValue::Bool(b) => *b,
            JsValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JsValue::String(s) => !s.is_empty(),
            JsValue::Array(a) => !a.is_empty(),
            JsValue::Object(o) => !o.is_empty(),
        }
    }
}

/// HTTP header entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpHeader {
    /// Header name
    pub name: String,
    /// Header value
    pub value: String,
}

/// HTTP response representation
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: HttpStatus,
    /// Response headers
    pub headers: Vec<HttpHeader>,
    /// Response body bytes
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Create a new HTTP response
    pub fn new(status: HttpStatus) -> Self {
        HttpResponse {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Look up a header by name (case-insensitive)
    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.iter().find(|h| h.name.eq_ignore_ascii_case(name)).map(|h| &h.value)
    }
}

/// MIME content type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// text/html
    Html,
    /// text/css
    Css,
    /// application/javascript
    JavaScript,
    /// application/json
    Json,
    /// image/png
    ImagePng,
    /// image/jpeg
    ImageJpeg,
    /// application/octet-stream
    Binary,
    /// Unknown or other
    Other,
}

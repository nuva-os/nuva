/*
 * Nuva OS - SystemLibrary - Net
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

//! HTTP Client

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// HTTP Method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HttpMethod {
    Get = 0,
    Post = 1,
    Put = 2,
    Delete = 3,
    Patch = 4,
    Head = 5,
    Options = 6,
}

/// HTTP Version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HttpVersion {
    Http10 = 0,
    Http11 = 1,
    Http20 = 2,
    Http3 = 3,
}

/// HTTP Headpart
#[derive(Debug, Clone, Copy)]
pub struct HttpHeader {
    pub name: [u8; 64],
    pub name_len: u8,
    pub value: [u8; 256],
    pub value_len: u8,
}

impl HttpHeader {
    pub fn new(name: &[u8], value: &[u8]) -> Self {
        let mut name_buf = [0u8; 64];
        let name_len = name.len().min(63);
        name_buf[..name_len].copy_from_slice(&name[..name_len]);
        
        let mut value_buf = [0u8; 256];
        let value_len = value.len().min(255);
        value_buf[..value_len].copy_from_slice(&value[..value_len]);
        
        Self {
            name: name_buf,
            name_len: name_len as u8,
            value: value_buf,
            value_len: value_len as u8,
        }
    }

    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_len as usize]
    }

    pub fn value(&self) -> &[u8] {
        &self.value[..self.value_len as usize]
    }
}

/// HTTP Request
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub version: HttpVersion,
    pub url: [u8; 512],
    pub url_len: u16,
    pub headers: [HttpHeader; 32],
    pub num_headers: u8,
    pub body: [u8; 4096],
    pub body_len: u16,
    pub timeout_ms: u32,
}

impl HttpRequest {
    pub fn new(method: HttpMethod, url: &[u8]) -> Self {
        let mut url_buf = [0u8; 512];
        let url_len = url.len().min(511);
        url_buf[..url_len].copy_from_slice(&url[..url_len]);
        
        Self {
            method,
            version: HttpVersion::Http11,
            url: url_buf,
            url_len: url_len as u16,
            headers: [HttpHeader {
                name: [0; 64],
                name_len: 0,
                value: [0; 256],
                value_len: 0,
            }; 32],
            num_headers: 0,
            body: [0; 4096],
            body_len: 0,
            timeout_ms: 30000,
        }
    }

    pub fn get(url: &[u8]) -> Self {
        Self::new(HttpMethod::Get, url)
    }

    pub fn post(url: &[u8]) -> Self {
        Self::new(HttpMethod::Post, url)
    }

    pub fn put(url: &[u8]) -> Self {
        Self::new(HttpMethod::Put, url)
    }

    pub fn delete(url: &[u8]) -> Self {
        Self::new(HttpMethod::Delete, url)
    }

    pub fn add_header(&mut self, name: &[u8], value: &[u8]) {
        if self.num_headers < 32 {
            self.headers[self.num_headers as usize] = HttpHeader::new(name, value);
            self.num_headers += 1;
        }
    }

    pub fn set_body(&mut self, body: &[u8]) {
        let len = body.len().min(4095);
        self.body[..len].copy_from_slice(&body[..len]);
        self.body_len = len as u16;
    }

    pub fn set_timeout(&mut self, timeout_ms: u32) {
        self.timeout_ms = timeout_ms;
    }

    pub fn url(&self) -> &[u8] {
        &self.url[..self.url_len as usize]
    }
}

/// HTTP Statecode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpStatusCode(pub u16);

impl HttpStatusCode {
    pub const OK: Self = Self(200);
    pub const CREATED: Self = Self(201);
    pub const NO_CONTENT: Self = Self(204);
    pub const BAD_REQUEST: Self = Self(400);
    pub const UNAUTHORIZED: Self = Self(401);
    pub const FORBIDDEN: Self = Self(403);
    pub const NOT_FOUND: Self = Self(404);
    pub const INTERNAL_ERROR: Self = Self(500);
    pub const BAD_GATEWAY: Self = Self(502);
    pub const SERVICE_UNAVAILABLE: Self = Self(503);

    pub fn is_success(&self) -> bool {
        self.0 >= 200 && self.0 < 300
    }

    pub fn is_redirect(&self) -> bool {
        self.0 >= 300 && self.0 < 400
    }

    pub fn is_client_error(&self) -> bool {
        self.0 >= 400 && self.0 < 500
    }

    pub fn is_server_error(&self) -> bool {
        self.0 >= 500 && self.0 < 600
    }
}

/// HTTP Response
#[derive(Debug)]
pub struct HttpResponse {
    pub version: HttpVersion,
    pub status_code: HttpStatusCode,
    pub status_message: [u8; 64],
    pub status_message_len: u8,
    pub headers: [HttpHeader; 32],
    pub num_headers: u8,
    pub body: [u8; 65536],
    pub body_len: u32,
}

impl HttpResponse {
    pub fn new() -> Self {
        Self {
            version: HttpVersion::Http11,
            status_code: HttpStatusCode::OK,
            status_message: [0; 64],
            status_message_len: 0,
            headers: [HttpHeader {
                name: [0; 64],
                name_len: 0,
                value: [0; 256],
                value_len: 0,
            }; 32],
            num_headers: 0,
            body: [0; 65536],
            body_len: 0,
        }
    }

    pub fn add_header(&mut self, name: &[u8], value: &[u8]) {
        if self.num_headers < 32 {
            self.headers[self.num_headers as usize] = HttpHeader::new(name, value);
            self.num_headers += 1;
        }
    }

    pub fn get_header(&self, name: &[u8]) -> Option<&[u8]> {
        for i in 0..self.num_headers as usize {
            if self.headers[i].name().eq_ignore_ascii_case(name) {
                return Some(self.headers[i].value());
            }
        }
        None
    }

    pub fn body(&self) -> &[u8] {
        &self.body[..self.body_len as usize]
    }

    pub fn is_success(&self) -> bool {
        self.status_code.is_success()
    }
}

/// URL parse
pub struct Url {
    pub scheme: [u8; 16],
    pub scheme_len: u8,
    pub host: [u8; 256],
    pub host_len: u8,
    pub port: u16,
    pub path: [u8; 512],
    pub path_len: u16,
    pub query: [u8; 256],
    pub query_len: u8,
    pub fragment: [u8; 64],
    pub fragment_len: u8,
}

impl Url {
    pub fn parse(url: &[u8]) -> Option<Self> {
        let mut result = Self {
            scheme: [0; 16],
            scheme_len: 0,
            host: [0; 256],
            host_len: 0,
            port: 0,
            path: [0; 512],
            path_len: 0,
            query: [0; 256],
            query_len: 0,
            fragment: [0; 64],
            fragment_len: 0,
        };
        
        // parse scheme
        if let Some(scheme_end) = url.iter().position(|&b| b == b':') {
            let scheme = &url[..scheme_end];
            let len = scheme.len().min(15);
            result.scheme[..len].copy_from_slice(&scheme[..len]);
            result.scheme_len = len as u8;
            
            // jumpover "://"
            let mut pos = scheme_end + 3;
            
            // parse host sum port
            let host_start = pos;
            while pos < url.len() && url[pos] != b'/' && url[pos] != b'?' && url[pos] != b'#' {
                pos += 1;
            }
            
            let host_part = &url[host_start..pos];
            if let Some(colon_pos) = host_part.iter().position(|&b| b == b':') {
                let host = &host_part[..colon_pos];
                let len = host.len().min(255);
                result.host[..len].copy_from_slice(host);
                result.host_len = len as u8;
                
                // parsePort
                let port_str = &host_part[colon_pos + 1..];
                let mut port = 0u16;
                for &b in port_str {
                    if b.is_ascii_digit() {
                        port = port * 10 + (b - b'0') as u16;
                    }
                }
                result.port = port;
            } else {
                let len = host_part.len().min(255);
                result.host[..len].copy_from_slice(host_part);
                result.host_len = len as u8;
                
                // DefaultPort
                if result.scheme == *b"http\0\0\0\0\0\0\0\0\0\0\0\0" {
                    result.port = 80;
                } else if result.scheme == *b"https\0\0\0\0\0\0\0\0\0\0\0" {
                    result.port = 443;
                }
            }
            
            // parse path
            if pos < url.len() && url[pos] == b'/' {
                let path_start = pos;
                while pos < url.len() && url[pos] != b'?' && url[pos] != b'#' {
                    pos += 1;
                }
                let path = &url[path_start..pos];
                let len = path.len().min(511);
                result.path[..len].copy_from_slice(&path[..len]);
                result.path_len = len as u16;
            }
            
            // parse query
            if pos < url.len() && url[pos] == b'?' {
                pos += 1;
                let query_start = pos;
                while pos < url.len() && url[pos] != b'#' {
                    pos += 1;
                }
                let query = &url[query_start..pos];
                let len = query.len().min(255);
                result.query[..len].copy_from_slice(&query[..len]);
                result.query_len = len as u8;
            }
            
            // parse fragment
            if pos < url.len() && url[pos] == b'#' {
                pos += 1;
                let fragment = &url[pos..];
                let len = fragment.len().min(63);
                result.fragment[..len].copy_from_slice(&fragment[..len]);
                result.fragment_len = len as u8;
            }
            
            return Some(result);
        }
        
        None
    }

    pub fn scheme(&self) -> &[u8] {
        &self.scheme[..self.scheme_len as usize]
    }

    pub fn host(&self) -> &[u8] {
        &self.host[..self.host_len as usize]
    }

    pub fn path(&self) -> &[u8] {
        if self.path_len > 0 {
            &self.path[..self.path_len as usize]
        } else {
            b"/"
        }
    }
}

/// HTTP Client
pub struct HttpClient {
    timeout_ms: u32,
    max_redirects: u32,
    follow_redirects: bool,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            timeout_ms: 30000,
            max_redirects: 5,
            follow_redirects: true,
        }
    }

    pub fn set_timeout(&mut self, timeout_ms: u32) {
        self.timeout_ms = timeout_ms;
    }

    pub fn set_follow_redirects(&mut self, follow: bool, max: u32) {
        self.follow_redirects = follow;
        self.max_redirects = max;
    }

    /// SendRequest
    pub fn send(&mut self, request: &HttpRequest) -> Result<HttpResponse, HttpError> {
        let url = Url::parse(request.url())
            .ok_or(HttpError::InvalidUrl)?;
        
        // buildcube TCP Join
        let _ = url.host();
        
        // SendRequest
        let mut response = HttpResponse::new();
        
        // parseResponse
        // SimplifiedImplementation
        response.status_code = HttpStatusCode::OK;
        response.version = HttpVersion::Http11;
        
        Ok(response)
    }

    /// GET Request
    pub fn get(&mut self, url: &[u8]) -> Result<HttpResponse, HttpError> {
        let request = HttpRequest::get(url);
        self.send(&request)
    }

    /// POST Request
    pub fn post(&mut self, url: &[u8], body: &[u8]) -> Result<HttpResponse, HttpError> {
        let mut request = HttpRequest::post(url);
        request.set_body(body);
        request.add_header(b"Content-Type", b"application/json");
        self.send(&request)
    }
}

/// HTTP Error
#[derive(Debug, Clone, Copy)]
pub enum HttpError {
    InvalidUrl,
    ConnectionFailed,
    Timeout,
    TooManyRedirects,
    InvalidResponse,
    TlsError,
    DnsError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method() {
        assert_eq!(HttpMethod::Get as u8, 0);
        assert_eq!(HttpMethod::Post as u8, 1);
        assert_eq!(HttpMethod::Put as u8, 2);
        assert_eq!(HttpMethod::Delete as u8, 3);
        assert_eq!(HttpMethod::Patch as u8, 4);
        assert_eq!(HttpMethod::Head as u8, 5);
        assert_eq!(HttpMethod::Options as u8, 6);
    }

    #[test]
    fn test_http_version() {
        assert_eq!(HttpVersion::Http10 as u8, 0);
        assert_eq!(HttpVersion::Http11 as u8, 1);
        assert_eq!(HttpVersion::Http20 as u8, 2);
        assert_eq!(HttpVersion::Http3 as u8, 3);
    }

    #[test]
    fn test_http_header() {
        let header = HttpHeader::new(b"Content-Type", b"application/json");

        assert_eq!(header.name(), b"Content-Type");
        assert_eq!(header.value(), b"application/json");
    }

    #[test]
    fn test_http_request_get() {
        let request = HttpRequest::get(b"https://example.com/api");

        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.version, HttpVersion::Http11);
        assert_eq!(request.url(), b"https://example.com/api");
    }

    #[test]
    fn test_http_request_post() {
        let mut request = HttpRequest::post(b"https://example.com/api");
        request.set_body(b"{\"key\":\"value\"}");
        request.add_header(b"Content-Type", b"application/json");

        assert_eq!(request.method, HttpMethod::Post);
        assert_eq!(request.num_headers, 1);
        assert_eq!(request.body_len, 15);
    }

    #[test]
    fn test_http_request_methods() {
        let put = HttpRequest::put(b"https://example.com/resource");
        assert_eq!(put.method, HttpMethod::Put);

        let delete = HttpRequest::delete(b"https://example.com/resource");
        assert_eq!(delete.method, HttpMethod::Delete);
    }

    #[test]
    fn test_http_status_code() {
        assert!(HttpStatusCode::OK.is_success());
        assert!(!HttpStatusCode::OK.is_client_error());

        assert!(HttpStatusCode::BAD_REQUEST.is_client_error());
        assert!(!HttpStatusCode::BAD_REQUEST.is_success());

        assert!(HttpStatusCode::INTERNAL_ERROR.is_server_error());
        assert!(!HttpStatusCode::INTERNAL_ERROR.is_client_error());
    }

    #[test]
    fn test_http_status_code_ranges() {
        let ok = HttpStatusCode(200);
        assert!(ok.is_success());

        let redirect = HttpStatusCode(302);
        assert!(redirect.is_redirect());

        let not_found = HttpStatusCode(404);
        assert!(not_found.is_client_error());

        let server_error = HttpStatusCode(503);
        assert!(server_error.is_server_error());
    }

    #[test]
    fn test_http_response() {
        let mut response = HttpResponse::new();

        response.add_header(b"Content-Length", b"1024");

        assert_eq!(response.version, HttpVersion::Http11);
        assert_eq!(response.status_code, HttpStatusCode::OK);
        assert_eq!(response.num_headers, 1);
        assert!(response.is_success());
    }

    #[test]
    fn test_http_response_get_header() {
        let mut response = HttpResponse::new();
        response.add_header(b"Content-Type", b"text/html");
        response.add_header(b"Content-Length", b"1024");

        let content_type = response.get_header(b"Content-Type");
        assert!(content_type.is_some());
        assert_eq!(content_type.unwrap(), b"text/html");

        let unknown = response.get_header(b"X-Custom");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_url_parse_http() {
        let url = Url::parse(b"http://example.com/path?query=1#fragment");

        assert!(url.is_some());
        let url = url.unwrap();

        assert_eq!(url.scheme(), b"http");
        assert_eq!(url.host(), b"example.com");
        assert_eq!(url.port, 80);
    }

    #[test]
    fn test_url_parse_https() {
        let url = Url::parse(b"https://example.com:8443/api");

        assert!(url.is_some());
        let url = url.unwrap();

        assert_eq!(url.scheme(), b"https");
        assert_eq!(url.host(), b"example.com");
        assert_eq!(url.port, 8443);
    }

    #[test]
    fn test_url_parse_with_port() {
        let url = Url::parse(b"http://localhost:8080/test");

        assert!(url.is_some());
        let url = url.unwrap();

        assert_eq!(url.host(), b"localhost");
        assert_eq!(url.port, 8080);
    }

    #[test]
    fn test_url_parse_invalid() {
        let url = Url::parse(b"not-a-url");
        assert!(url.is_none());
    }

    #[test]
    fn test_http_client_new() {
        let client = HttpClient::new();

        assert_eq!(client.timeout_ms, 30000);
        assert_eq!(client.max_redirects, 5);
        assert!(client.follow_redirects);
    }

    #[test]
    fn test_http_client_config() {
        let mut client = HttpClient::new();

        client.set_timeout(5000);
        assert_eq!(client.timeout_ms, 5000);

        client.set_follow_redirects(false, 3);
        assert!(!client.follow_redirects);
        assert_eq!(client.max_redirects, 3);
    }

    #[test]
    fn test_http_request_timeout() {
        let mut request = HttpRequest::get(b"https://example.com");
        request.set_timeout(5000);

        assert_eq!(request.timeout_ms, 5000);
    }
}
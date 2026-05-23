/*
 * Nuva OS - SystemService - Web - Security
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

//! Security policy enforcement for the web engine.
//! Implements same-origin policy, secure context verification,
//! CORS whitelist management, and content security policy.

use alloc::string::String;
use alloc::vec::Vec;

use super::error::{Url, WebError};

/// URL scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    /// HTTP (insecure)
    Http,
    /// HTTPS (secure)
    Https,
    /// File (local)
    File,
    /// Data (inline)
    Data,
    /// Blob
    Blob,
    /// WebSocket (insecure)
    Ws,
    /// WebSocket Secure
    Wss,
    /// About (blank, etc.)
    About,
    /// Unknown scheme
    Unknown,
}

impl Scheme {
    /// Parse a scheme from a string
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            "file" => Scheme::File,
            "data" => Scheme::Data,
            "blob" => Scheme::Blob,
            "ws" => Scheme::Ws,
            "wss" => Scheme::Wss,
            "about" => Scheme::About,
            _ => Scheme::Unknown,
        }
    }

    /// Check if this scheme is cryptographically secure
    pub fn is_secure(&self) -> bool {
        matches!(self, Scheme::Https | Scheme::Wss | Scheme::File)
    }
}

/// Origin representation (scheme + host + port)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    /// URL scheme
    pub scheme: Scheme,
    /// Host name
    pub host: String,
    /// Port number (0 = default for scheme)
    pub port: u16,
}

impl Origin {
    /// Create an origin from a URL
    pub fn from_url(url: &Url) -> Self {
        Origin {
            scheme: Scheme::from_str(&url.scheme),
            host: url.host.clone(),
            port: url.port,
        }
    }

    /// Get the default port for a scheme
    pub fn default_port(scheme: Scheme) -> u16 {
        match scheme {
            Scheme::Http | Scheme::Ws => 80,
            Scheme::Https | Scheme::Wss => 443,
            Scheme::File => 0,
            _ => 0,
        }
    }

    /// Get the effective port (explicit or default)
    pub fn effective_port(&self) -> u16 {
        if self.port != 0 {
            self.port
        } else {
            Self::default_port(self.scheme)
        }
    }

    /// Check if two origins are the same (same-origin policy)
    pub fn is_same_origin(&self, other: &Origin) -> bool {
        // Special cases: file:// and null origins are never same-origin
        if self.scheme == Scheme::File || other.scheme == Scheme::File {
            return false;
        }
        if self.scheme == Scheme::Data || other.scheme == Scheme::Data {
            return false;
        }
        if self.scheme == Scheme::About || other.scheme == Scheme::About {
            return false;
        }

        self.scheme == other.scheme
            && self.host == other.host
            && self.effective_port() == other.effective_port()
    }
}

/// CORS mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorsMode {
    /// No CORS (same-origin requests only)
    NoCors,
    /// CORS with credentials
    CorsWithCredentials,
    /// CORS without credentials
    Cors,
}

/// Content Security Policy directive
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CspDirective {
    /// Directive name (e.g. "script-src", "img-src")
    pub name: String,
    /// Allowed sources
    pub sources: Vec<String>,
}

impl CspDirective {
    /// Check if a source is allowed by this directive
    pub fn allows(&self, source: &str) -> bool {
        // 'none' blocks everything
        if self.sources.contains(&String::from("'none'")) {
            return false;
        }
        // 'star' allows everything
        if self.sources.contains(&String::from("*")) {
            return true;
        }
        // 'self' allows same-origin (checked by caller)
        // Check explicit source list
        self.sources.iter().any(|s| s == source)
    }
}

/// Security policy for a page
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Page origin
    pub origin: Origin,
    /// Whether this is a secure context
    pub is_secure_context: bool,
    /// CORS whitelist (list of allowed cross-origin origins)
    pub cors_whitelist: Vec<Origin>,
    /// Content Security Policy directives
    pub csp_directives: Vec<CspDirective>,
    /// Whether to enforce strict mixed content checking
    pub strict_mixed_content: bool,
    /// Whether to block all mixed content
    pub block_mixed_content: bool,
    /// Whether referrer policy is strict-origin-when-cross-origin
    pub strict_referrer: bool,
    /// Whether to enforce Subresource Integrity
    pub enforce_sri: bool,
}

impl SecurityPolicy {
    /// Create a security policy for a given origin
    pub fn for_origin(origin: Origin) -> Self {
        let is_secure = origin.scheme.is_secure();
        SecurityPolicy {
            origin,
            is_secure_context: is_secure,
            cors_whitelist: Vec::new(),
            csp_directives: Vec::new(),
            strict_mixed_content: true,
            block_mixed_content: true,
            strict_referrer: true,
            enforce_sri: false,
        }
    }

    /// Create a security policy from a URL
    pub fn from_url(url: &Url) -> Self {
        let origin = Origin::from_url(url);
        Self::for_origin(origin)
    }

    /// Check if access from source to target is allowed (same-origin policy)
    pub fn check_access(&self, source: &Origin, target: &Origin) -> Result<(), WebError> {
        // Same-origin access is always allowed
        if source.is_same_origin(target) {
            return Ok(());
        }

        // Check CORS whitelist
        if self.cors_whitelist.iter().any(|o| o.is_same_origin(target)) {
            return Ok(());
        }

        Err(WebError::CrossOriginDenied)
    }

    /// Check if a cross-origin request is allowed via CORS
    pub fn check_cors(
        &self,
        request_origin: &Origin,
        mode: CorsMode,
    ) -> Result<(), WebError> {
        match mode {
            CorsMode::NoCors => {
                // No CORS mode: opaque responses only, no access to response data
                // But the request itself is allowed
                Ok(())
            }
            CorsMode::Cors => {
                // Same-origin or whitelisted
                if self.origin.is_same_origin(request_origin) {
                    return Ok(());
                }
                if self.cors_whitelist.iter().any(|o| o.is_same_origin(request_origin)) {
                    return Ok(());
                }
                Err(WebError::CrossOriginDenied)
            }
            CorsMode::CorsWithCredentials => {
                // Credential mode: origin must be explicitly whitelisted
                // and wildcard (*) is not accepted
                if self.cors_whitelist.iter().any(|o| o.is_same_origin(request_origin)) {
                    return Ok(());
                }
                Err(WebError::CrossOriginDenied)
            }
        }
    }

    /// Check if the current context is secure (HTTPS required for sensitive APIs)
    pub fn check_secure_context(&self) -> Result<(), WebError> {
        if self.is_secure_context {
            Ok(())
        } else {
            Err(WebError::InsecureContextRequired)
        }
    }

    /// Check if loading a mixed-content subresource is allowed
    pub fn check_mixed_content(&self, subresource_scheme: Scheme) -> Result<(), WebError> {
        // Mixed content: secure page loading insecure subresource
        if self.is_secure_context && !subresource_scheme.is_secure() {
            if self.block_mixed_content {
                return Err(WebError::InsecureContextRequired);
            }
            if self.strict_mixed_content && subresource_scheme == Scheme::Http {
                return Err(WebError::InsecureContextRequired);
            }
        }
        Ok(())
    }

    /// Check if a CSP directive allows a given source
    pub fn check_csp(&self, directive_name: &str, source: &str) -> Result<(), WebError> {
        for directive in &self.csp_directives {
            if directive.name == directive_name {
                if !directive.allows(source) {
                    return Err(WebError::CrossOriginDenied);
                }
                return Ok(());
            }
        }
        // No CSP directive found: allow by default
        Ok(())
    }

    /// Add a CORS-whitelisted origin
    pub fn add_cors_whitelist(&mut self, origin: Origin) {
        if !self.cors_whitelist.iter().any(|o| o.is_same_origin(&origin)) {
            self.cors_whitelist.push(origin);
        }
    }

    /// Remove a CORS-whitelisted origin
    pub fn remove_cors_whitelist(&mut self, origin: &Origin) {
        self.cors_whitelist.retain(|o| !o.is_same_origin(origin));
    }

    /// Set a CSP directive
    pub fn set_csp_directive(&mut self, name: String, sources: Vec<String>) {
        if let Some(d) = self.csp_directives.iter_mut().find(|d| d.name == name) {
            d.sources = sources;
        } else {
            self.csp_directives.push(CspDirective { name, sources });
        }
    }
}

/// Security manager (coordinates policies for all pages)
pub struct SecurityManager {
    /// Total access checks performed
    total_checks: u64,
    /// Total denials
    total_denials: u64,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        SecurityManager {
            total_checks: 0,
            total_denials: 0,
        }
    }

    /// Check access and record statistics
    pub fn check_access(&mut self, policy: &SecurityPolicy, source: &Origin, target: &Origin) -> Result<(), WebError> {
        self.total_checks += 1;
        let result = policy.check_access(source, target);
        if result.is_err() {
            self.total_denials += 1;
        }
        result
    }

    /// Get the denial rate
    pub fn denial_rate(&self) -> f32 {
        if self.total_checks == 0 {
            0.0
        } else {
            self.total_denials as f32 / self.total_checks as f32
        }
    }
}

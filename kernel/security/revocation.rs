/*
 * Nuva OS - Kernel - Certificate Revocation Checking
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

//! Certificate Revocation Checking
//!
//! Implements OCSP (RFC 6960) and CRL (RFC 5280 section 5) checking
//! with configurable soft-fail/hard-fail behavior and timeout.

use core::sync::atomic::{AtomicU64, Ordering};
use alloc::vec::Vec;

use crate::kernel::security::x509::X509Certificate;
use crate::{pr_info, pr_warn, pr_debug};

/// Revocation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationStatus {
    /// Certificate is not revoked
    Good,
    /// Certificate is revoked
    Revoked,
    /// Revocation status unknown (OCSP/CRL unavailable)
    Unknown,
    /// Revocation check timed out
    Timeout,
}

/// Revocation check error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationError {
    /// OCSP request failed
    OcspRequestFailed,
    /// OCSP response invalid
    OcspResponseInvalid,
    /// CRL download failed
    CrlDownloadFailed,
    /// CRL parsing failed
    CrlParseFailed,
    /// OCSP/CRL responder unreachable
    Unreachable,
    /// Timeout
    Timeout,
    /// No revocation endpoint in certificate
    NoEndpoint,
}

/// Revocation failure mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationMode {
    /// Timeout/network failure -> treat as valid (log warning)
    SoftFail,
    /// Timeout/network failure -> reject certificate
    HardFail,
}

/// Revocation checking configuration
#[derive(Debug, Clone, Copy)]
pub struct RevocationConfig {
    /// Enable OCSP checking
    pub ocsp_enabled: bool,
    /// Enable CRL checking
    pub crl_enabled: bool,
    /// OCSP request timeout in milliseconds (default: 1500)
    pub ocsp_timeout_ms: u32,
    /// CRL download timeout in milliseconds (default: 5000)
    pub crl_timeout_ms: u32,
    /// Failure mode
    pub mode: RevocationMode,
    /// Prefer OCSP over CRL when both are available
    pub prefer_ocsp: bool,
    /// Maximum CRL cache age in seconds (default: 3600)
    pub crl_cache_max_age_secs: u32,
}

impl RevocationConfig {
    /// Create default configuration
    pub const fn default() -> Self {
        RevocationConfig {
            ocsp_enabled: true,
            crl_enabled: true,
            ocsp_timeout_ms: 1500,
            crl_timeout_ms: 5000,
            mode: RevocationMode::SoftFail,
            prefer_ocsp: true,
            crl_cache_max_age_secs: 3600,
        }
    }
}

/// Revocation checking trait
pub trait RevocationChecker {
    /// Check revocation status of a certificate
    fn check_revocation(
        &self,
        cert: &X509Certificate,
        issuer: &X509Certificate,
    ) -> Result<RevocationStatus, RevocationError>;
}

/// OCSP checker (RFC 6960)
pub struct OcspChecker {
    /// Configuration
    pub config: RevocationConfig,
    /// Request count
    request_count: AtomicU64,
}

impl OcspChecker {
    /// Create a new OCSP checker
    pub fn new(config: RevocationConfig) -> Self {
        OcspChecker {
            config,
            request_count: AtomicU64::new(0),
        }
    }

    /// Check OCSP status for a certificate
    pub fn check(
        &self,
        cert: &X509Certificate,
        issuer: &X509Certificate,
    ) -> Result<RevocationStatus, RevocationError> {
        self.request_count.fetch_add(1, Ordering::AcqRel);

        // Get OCSP URL from Authority Information Access
        let aia = cert.authority_info_access();
        let ocsp_url = aia.and_then(|a| a.ocsp_url.as_ref());

        if ocsp_url.is_none() {
            return Err(RevocationError::NoEndpoint);
        }

        // In a full implementation:
        // 1. Build OCSPRequest with nonce for replay protection
        // 2. Send HTTP POST to OCSP responder
        // 3. Wait for response (with timeout)
        // 4. Verify OCSP Response signature
        // 5. Parse CertStatus: Good / Revoked / Unknown

        // Placeholder: return Unknown for now
        Ok(RevocationStatus::Unknown)
    }
}

impl RevocationChecker for OcspChecker {
    fn check_revocation(
        &self,
        cert: &X509Certificate,
        issuer: &X509Certificate,
    ) -> Result<RevocationStatus, RevocationError> {
        self.check(cert, issuer)
    }
}

/// CRL checker (RFC 5280 section 5)
pub struct CrlChecker {
    /// Configuration
    pub config: RevocationConfig,
    /// Check count
    check_count: AtomicU64,
}

impl CrlChecker {
    /// Create a new CRL checker
    pub fn new(config: RevocationConfig) -> Self {
        CrlChecker {
            config,
            check_count: AtomicU64::new(0),
        }
    }

    /// Check CRL status for a certificate
    pub fn check(
        &self,
        cert: &X509Certificate,
        _issuer: &X509Certificate,
    ) -> Result<RevocationStatus, RevocationError> {
        self.check_count.fetch_add(1, Ordering::AcqRel);

        // Get CRL Distribution Points
        let crl_dp = cert.crl_distribution_points();

        if crl_dp.is_none() || crl_dp.unwrap().points.is_empty() {
            return Err(RevocationError::NoEndpoint);
        }

        // In a full implementation:
        // 1. Download CRL from distribution point
        // 2. Verify CRL signature
        // 3. Check if certificate serial number is in CRL

        // Placeholder: return Unknown for now
        Ok(RevocationStatus::Unknown)
    }
}

impl RevocationChecker for CrlChecker {
    fn check_revocation(
        &self,
        cert: &X509Certificate,
        issuer: &X509Certificate,
    ) -> Result<RevocationStatus, RevocationError> {
        self.check(cert, issuer)
    }
}

/// Composite revocation checker: OCSP (preferred) + CRL (fallback)
pub struct CompositeRevocationChecker {
    /// OCSP checker
    pub ocsp: OcspChecker,
    /// CRL checker
    pub crl: CrlChecker,
}

impl CompositeRevocationChecker {
    /// Create with OCSP and CRL checkers
    pub fn new(config: RevocationConfig) -> Self {
        CompositeRevocationChecker {
            ocsp: OcspChecker::new(config),
            crl: CrlChecker::new(config),
        }
    }

    /// Check revocation: try OCSP first, fallback to CRL on failure
    pub fn check(
        &self,
        cert: &X509Certificate,
        issuer: &X509Certificate,
    ) -> Result<RevocationStatus, RevocationError> {
        let config = self.ocsp.config;

        if config.prefer_ocsp && config.ocsp_enabled {
            match self.ocsp.check(cert, issuer) {
                Ok(RevocationStatus::Good) => return Ok(RevocationStatus::Good),
                Ok(RevocationStatus::Revoked) => return Ok(RevocationStatus::Revoked),
                Ok(RevocationStatus::Unknown) => {
                    // OCSP unknown, try CRL fallback
                    if config.crl_enabled {
                        return self.crl.check(cert, issuer);
                    }
                    return Ok(RevocationStatus::Unknown);
                }
                Ok(RevocationStatus::Timeout) => {
                    // OCSP timeout, apply failure mode
                    if config.mode == RevocationMode::HardFail {
                        return Err(RevocationError::Timeout);
                    }
                    // Soft-fail: try CRL
                    if config.crl_enabled {
                        return self.crl.check(cert, issuer);
                    }
                    return Ok(RevocationStatus::Unknown);
                }
                Err(_) => {
                    // OCSP error, try CRL fallback
                    if config.crl_enabled {
                        return self.crl.check(cert, issuer);
                    }
                    if config.mode == RevocationMode::SoftFail {
                        return Ok(RevocationStatus::Unknown);
                    }
                    return Err(RevocationError::OcspRequestFailed);
                }
            }
        }

        // CRL only or OCSP disabled
        if config.crl_enabled {
            return self.crl.check(cert, issuer);
        }

        // No revocation checking enabled
        Ok(RevocationStatus::Unknown)
    }
}

impl RevocationChecker for CompositeRevocationChecker {
    fn check_revocation(
        &self,
        cert: &X509Certificate,
        issuer: &X509Certificate,
    ) -> Result<RevocationStatus, RevocationError> {
        self.check(cert, issuer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revocation_config_default() {
        let config = RevocationConfig::default();
        assert!(config.ocsp_enabled);
        assert!(config.crl_enabled);
        assert_eq!(config.ocsp_timeout_ms, 1500);
        assert_eq!(config.crl_timeout_ms, 5000);
        assert_eq!(config.mode, RevocationMode::SoftFail);
        assert!(config.prefer_ocsp);
    }

    #[test]
    fn test_revocation_status_values() {
        assert_ne!(RevocationStatus::Good, RevocationStatus::Revoked);
        assert_ne!(RevocationStatus::Unknown, RevocationStatus::Timeout);
    }
}
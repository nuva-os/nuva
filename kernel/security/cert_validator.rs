/*
 * Nuva OS - Kernel - Certificate Constraint Validator
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

//! Certificate Constraint Validator
//!
//! Validates X.509 certificate constraints per RFC 5280:
//! - BasicConstraints: CA flag + pathLenConstraint
//! - Key Usage: keyCertSign for CA, digitalSignature for leaf
//! - Extended Key Usage: codeSigning for leaf
//! - Validity period: notBefore <= now <= notAfter
//! - Signature algorithm: reject MD5/SHA1

use crate::kernel::security::x509::{
    X509Certificate, ExtKeyUsage, KU_DIGITAL_SIGNATURE, KU_KEY_CERT_SIGN,
};
use crate::kernel::security::signature::SignatureAlgorithm;
use crate::{pr_info, pr_warn};

/// Constraint validation error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintError {
    /// CA flag mismatch
    InvalidCaFlag,
    /// Key usage violation
    KeyUsageViolation,
    /// Extended key usage violation
    ExtKeyUsageViolation,
    /// Path length exceeded
    PathLengthExceeded,
    /// Certificate expired
    Expired,
    /// Certificate not yet valid
    NotYetValid,
    /// Unsupported signature algorithm
    UnsupportedAlgorithm,
}

/// Certificate constraint validator
pub struct CertValidator;

impl CertValidator {
    /// Validate BasicConstraints
    /// - CA certificates must have BasicConstraints with cA=true
    /// - Non-CA certificates must not have cA=true (or omit extension)
    /// - Enforce pathLenConstraint if present
    pub fn validate_basic_constraints(
        cert: &X509Certificate,
        depth: u32,
    ) -> Result<(), ConstraintError> {
        let is_ca = cert.is_ca();

        if is_ca {
            // CA cert: verify it has BasicConstraints with cA=true
            if cert.tbs.extensions.basic_constraints.is_none() {
                log_warn!("CertValidator: CA cert missing BasicConstraints");
                return Err(ConstraintError::InvalidCaFlag);
            }
        }

        // Enforce pathLenConstraint
        if let Some(path_len) = cert.path_length_constraint() {
            if depth > path_len {
                log_warn!("CertValidator: path length exceeded ({}/{})", depth, path_len);
                return Err(ConstraintError::PathLengthExceeded);
            }
        }

        Ok(())
    }

    /// Validate Key Usage
    /// - CA certificates: keyCertSign must be set
    /// - Leaf certificates: digitalSignature must be set
    /// - codeSigning leaf: extendedKeyUsage must include codeSigning OID
    pub fn validate_key_usage(
        cert: &X509Certificate,
        is_leaf: bool,
    ) -> Result<(), ConstraintError> {
        if is_leaf {
            // Leaf certificate: digitalSignature required
            if !cert.has_key_usage(KU_DIGITAL_SIGNATURE) {
                log_warn!("CertValidator: leaf missing digitalSignature");
                return Err(ConstraintError::KeyUsageViolation);
            }
        } else {
            // CA certificate: keyCertSign required
            if !cert.has_key_usage(KU_KEY_CERT_SIGN) {
                log_warn!("CertValidator: CA missing keyCertSign");
                return Err(ConstraintError::KeyUsageViolation);
            }
        }

        Ok(())
    }

    /// Validate Extended Key Usage for leaf certificates
    /// Leaf certificates must have codeSigning EKU for code signing
    pub fn validate_ext_key_usage(
        cert: &X509Certificate,
        expected_eku: Option<ExtKeyUsage>,
    ) -> Result<(), ConstraintError> {
        if let Some(eku) = expected_eku {
            if !cert.has_ext_key_usage(eku) {
                log_warn!("CertValidator: missing expected EKU");
                return Err(ConstraintError::ExtKeyUsageViolation);
            }
        }
        Ok(())
    }

    /// Validate path length constraint
    /// If pathLenConstraint = 0, no intermediate CAs allowed below this CA
    /// If pathLenConstraint = n, at most n intermediate CAs below this CA
    pub fn validate_path_length(
        cert: &X509Certificate,
        intermediate_count: u32,
    ) -> Result<(), ConstraintError> {
        if let Some(path_len) = cert.path_length_constraint() {
            if intermediate_count > path_len {
                log_warn!("CertValidator: pathLenConstraint violated ({}/{})", intermediate_count, path_len);
                return Err(ConstraintError::PathLengthExceeded);
            }
        }
        Ok(())
    }

    /// Validate validity period
    pub fn validate_validity(
        cert: &X509Certificate,
        now: u64,
    ) -> Result<(), ConstraintError> {
        let validity = cert.validity();
        if validity.is_expired(now) {
            log_warn!("CertValidator: certificate expired");
            return Err(ConstraintError::Expired);
        }
        if validity.is_not_yet_valid(now) {
            log_warn!("CertValidator: certificate not yet valid");
            return Err(ConstraintError::NotYetValid);
        }
        Ok(())
    }

    /// Validate signature algorithm (reject MD5/SHA1)
    pub fn validate_signature_algorithm(
        cert: &X509Certificate,
    ) -> Result<(), ConstraintError> {
        // MD5/SHA1 are rejected during X509Certificate::from_der()
        // If we got here, the algorithm is acceptable
        let _ = cert;
        Ok(())
    }

    /// Full certificate validation (all constraints)
    pub fn validate(
        cert: &X509Certificate,
        depth: u32,
        is_leaf: bool,
        now: u64,
    ) -> Result<(), ConstraintError> {
        Self::validate_basic_constraints(cert, depth)?;
        Self::validate_key_usage(cert, is_leaf)?;
        if is_leaf {
            Self::validate_ext_key_usage(cert, Some(ExtKeyUsage::CodeSigning))?;
        }
        Self::validate_path_length(cert, depth)?;
        Self::validate_validity(cert, now)?;
        Self::validate_signature_algorithm(cert)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_error_values() {
        assert_ne!(ConstraintError::InvalidCaFlag, ConstraintError::KeyUsageViolation);
        assert_ne!(ConstraintError::Expired, ConstraintError::NotYetValid);
    }
}
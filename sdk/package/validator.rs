/*
 * Nuva OS
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

// ! Package validation and integrity verification

use super::meta::Package;
use crate::error::SdkError;

/// Package validator
pub struct PackageValidator {
    /// Maximum allowed package name length
    max_name_length: usize,
    /// Maximum allowed package size in bytes
    max_package_size: usize,
}

impl PackageValidator {
    pub fn new() -> Self {
        Self {
            max_name_length: 128,
            max_package_size: 100 * 1024 * 1024,
        }
    }

    /// Validate a package
    pub fn validate(&self, pkg: &Package) -> Result<ValidationResult, SdkError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if pkg.name.is_empty() {
            errors.push("Package name is empty".to_string());
        }

        if pkg.name.len() > self.max_name_length {
            errors.push(format!(
                "Package name exceeds maximum length ({} > {})",
                pkg.name.len(),
                self.max_name_length
            ));
        }

        for c in pkg.name.chars() {
            if !is_valid_name_char(c) {
                errors.push(format!("Package name contains invalid character: '{}'", c));
                break;
            }
        }

        if pkg.name.starts_with('-') || pkg.name.starts_with('.') {
            errors.push("Package name cannot start with '-' or '.'".to_string());
        }

        if pkg.version.major == 0 && pkg.version.minor == 0 && pkg.version.patch == 0 {
            warnings.push("Package version is 0.0.0".to_string());
        }

        if pkg.authors.is_empty() {
            warnings.push("Package has no authors specified".to_string());
        }

        if pkg.description.is_none() || pkg.description.as_ref().map_or(true, |d| d.is_empty()) {
            warnings.push("Package has no description".to_string());
        }

        if pkg.license.is_none() {
            warnings.push("Package has no license specified".to_string());
        }

        for dep in &pkg.dependencies {
            if dep.name.is_empty() {
                errors.push("Dependency has empty name".to_string());
            }
            if dep.name == pkg.name {
                errors.push("Package cannot depend on itself".to_string());
            }
        }

        if let Some(ref data) = pkg.data {
            if data.len() > self.max_package_size {
                errors.push(format!(
                    "Package size exceeds maximum ({} > {})",
                    data.len(),
                    self.max_package_size
                ));
            }
        }

        Ok(ValidationResult { errors, warnings })
    }

    /// Verify package checksum
    pub fn verify_checksum(&self, pkg: &Package, expected: &str) -> Result<bool, SdkError> {
        if expected.is_empty() {
            return Err(SdkError::ValidationError("Empty checksum".to_string()));
        }

        let actual = compute_checksum(pkg);
        Ok(actual == expected)
    }

    /// Validate package format (structure, required files)
    pub fn validate_format(&self, pkg: &Package) -> Result<ValidationResult, SdkError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if pkg.targets.is_empty() {
            warnings.push("Package has no build targets defined".to_string());
        }

        let has_lib = pkg.targets.iter().any(|t| matches!(t.kind, super::meta::TargetKind::Lib));
        let has_bin = pkg.targets.iter().any(|t| matches!(t.kind, super::meta::TargetKind::Bin));

        if !has_lib && !has_bin {
            warnings.push("Package has neither library nor binary targets".to_string());
        }

        for target in &pkg.targets {
            if !target.path.exists() {
                errors.push(format!("Target path does not exist: {}", target.path.display()));
            }
        }

        Ok(ValidationResult { errors, warnings })
    }
}

impl Default for PackageValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Validation errors
    errors: Vec<String>,
    /// Validation warnings
    warnings: Vec<String>,
}

impl ValidationResult {
    /// Whether validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get errors
    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    /// Get warnings
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

/// Check if character is valid in package name
fn is_valid_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'
}

/// Compute SHA-256 checksum for package metadata
fn compute_checksum(pkg: &Package) -> String {
    let data = format!("{}{}.{}.{}", pkg.name, pkg.version.major, pkg.version.minor, pkg.version.patch);
    let hash = sha256(&data.as_bytes());
    let mut hex = String::with_capacity(64);
    for b in &hash {
        hex.push_str(&format!("{:02x}", b));
    }
    hex
}

/// SHA-256 hash (FIPS 180-4)
fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
        0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
        0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
        0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
        0x391c0cb3, 0x4ed8aa4a, 0x5b9aca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let bit_len = (data.len() as u64) * 8;
    let mut i = 0;
    while i + 64 <= data.len() {
        sha256_compress(&mut state, &data[i..i + 64], &K);
        i += 64;
    }
    let mut block = [0u8; 64];
    let remaining = data.len() - i;
    block[..remaining].copy_from_slice(&data[i..]);
    block[remaining] = 0x80;
    if remaining >= 56 {
        sha256_compress(&mut state, &block, &K);
        block = [0u8; 64];
    }
    block[56..64].copy_from_slice(&bit_len.to_be_bytes());
    sha256_compress(&mut state, &block, &K);
    let mut result = [0u8; 32];
    for j in 0..8 {
        result[j * 4..j * 4 + 4].copy_from_slice(&state[j].to_be_bytes());
    }
    result
}

fn sha256_compress(state: &mut [u32; 8], block: &[u8], k: &[u32; 64]) {
    let mut w = [0u32; 64];
    for i in 0..16 {
        w[i] = u32::from_be_bytes([block[i*4], block[i*4+1], block[i*4+2], block[i*4+3]]);
    }
    for i in 16..64 {
        let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
        let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
        w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
    for i in 0..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ (!e & g);
        let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);
        h = g; g = f; f = e; e = d.wrapping_add(temp1);
        d = c; c = b; b = a; a = temp1.wrapping_add(temp2);
    }
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

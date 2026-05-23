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

// NIST Known Answer Test (KAT) Framework for Post-Quantum Cryptography
// File: tests/kat/mod.rs

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// KAT vector for key encapsulation mechanisms (Kyber)
#[derive(Debug, Clone, Default)]
pub struct KemKatVector {
    pub count: u32,
    pub seed: Vec<u8>,
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub shared_secret: Vec<u8>,
}

/// KAT vector for signature schemes (Dilithium)
#[derive(Debug, Clone, Default)]
pub struct SignKatVector {
    pub count: u32,
    pub seed: Vec<u8>,
    pub message: Vec<u8>,
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub signature: Vec<u8>,
}

/// Parse NIST KAT file for KEM schemes
pub fn parse_kem_kat_file<P: AsRef<Path>>(path: P) -> Vec<KemKatVector> {
    let file = File::open(path).expect("Failed to open KAT file");
    let reader = BufReader::new(file);
    let mut vectors = Vec::new();
    let mut current = KemKatVector::default();
    
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some(value) = parse_hex_field(line, "count = ") {
            if current.count != 0 {
                vectors.push(current.clone());
            }
            current = KemKatVector::default();
            current.count = value.iter().rev().enumerate()
                .map(|(i, &b)| (b as u32) << (i * 8))
                .sum();
        } else if let Some(value) = parse_hex_field(line, "seed = ") {
            current.seed = value;
        } else if let Some(value) = parse_hex_field(line, "pk = ") {
            current.public_key = value;
        } else if let Some(value) = parse_hex_field(line, "sk = ") {
            current.secret_key = value;
        } else if let Some(value) = parse_hex_field(line, "ct = ") {
            current.ciphertext = value;
        } else if let Some(value) = parse_hex_field(line, "ss = ") {
            current.shared_secret = value;
        }
    }
    
    if current.count != 0 {
        vectors.push(current);
    }
    
    vectors
}

/// Parse NIST KAT file for signature schemes
pub fn parse_sign_kat_file<P: AsRef<Path>>(path: P) -> Vec<SignKatVector> {
    let file = File::open(path).expect("Failed to open KAT file");
    let reader = BufReader::new(file);
    let mut vectors = Vec::new();
    let mut current = SignKatVector::default();
    
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some(value) = parse_hex_field(line, "count = ") {
            if current.count != 0 {
                vectors.push(current.clone());
            }
            current = SignKatVector::default();
            current.count = value.iter().rev().enumerate()
                .map(|(i, &b)| (b as u32) << (i * 8))
                .sum();
        } else if let Some(value) = parse_hex_field(line, "seed = ") {
            current.seed = value;
        } else if let Some(value) = parse_hex_field(line, "msg = ") {
            current.message = value;
        } else if let Some(value) = parse_hex_field(line, "pk = ") {
            current.public_key = value;
        } else if let Some(value) = parse_hex_field(line, "sk = ") {
            current.secret_key = value;
        } else if let Some(value) = parse_hex_field(line, "sig = ") {
            current.signature = value;
        }
    }
    
    if current.count != 0 {
        vectors.push(current);
    }
    
    vectors
}

/// Parse a hex field from a KAT file line
fn parse_hex_field(line: &str, prefix: &str) -> Option<Vec<u8>> {
    if line.starts_with(prefix) {
        let hex_str = &line[prefix.len()..];
        Some(hex_decode(hex_str))
    } else {
        None
    }
}

/// Decode hex string to bytes
fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hex_decode() {
        assert_eq!(hex_decode("00"), vec![0x00]);
        assert_eq!(hex_decode("FF"), vec![0xFF]);
        assert_eq!(hex_decode("0102"), vec![0x01, 0x02]);
        assert_eq!(hex_decode("deadbeef"), vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }
    
    #[test]
    fn test_parse_kem_kat() {
        // Create a test KAT file
        let test_content = r#"
count = 0
seed = 000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F202122232425262728292A2B2C2D2E2F303132333435363738393A3B3C3D3E3F
pk = 000102030405060708090A0B0C0D0E0F
sk = 101112131415161718191A1B1C1D1E1F
ct = 202122232425262728292A2B2C2D2E2F
ss = 303132333435363738393A3B3C3D3E3F
"#;
        
        // Write to temp file
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(test_content.as_bytes()).unwrap();
        
        // Parse
        let vectors = parse_kem_kat_file(temp_file.path());
        assert_eq!(vectors.len(), 1);
        assert_eq!(vectors[0].count, 0);
        assert_eq!(vectors[0].seed.len(), 32);
        assert_eq!(vectors[0].public_key.len(), 16);
    }
}

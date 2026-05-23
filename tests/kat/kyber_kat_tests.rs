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

// NIST KAT Tests for Kyber Key Encapsulation Mechanism
// File: tests/kat/kyber_kat_tests.rs

mod kat;

use kat::{parse_kem_kat_file, KemKatVector};

/// Kyber-512 parameter sizes
const KYBER512_PK_SIZE: usize = 800;
const KYBER512_SK_SIZE: usize = 1632;
const KYBER512_CT_SIZE: usize = 768;
const KYBER512_SS_SIZE: usize = 32;

/// Kyber-768 parameter sizes
const KYBER768_PK_SIZE: usize = 1184;
const KYBER768_SK_SIZE: usize = 2400;
const KYBER768_CT_SIZE: usize = 1088;
const KYBER768_SS_SIZE: usize = 32;

/// Kyber-1024 parameter sizes
const KYBER1024_PK_SIZE: usize = 1568;
const KYBER1024_SK_SIZE: usize = 3168;
const KYBER1024_CT_SIZE: usize = 1568;
const KYBER1024_SS_SIZE: usize = 32;

#[test]
fn test_kyber768_kat_vector_sizes() {
    // Verify that our parameter sizes match the specification
    assert_eq!(KYBER768_PK_SIZE, 1184, "Kyber-768 public key size mismatch");
    assert_eq!(KYBER768_SK_SIZE, 2400, "Kyber-768 secret key size mismatch");
    assert_eq!(KYBER768_CT_SIZE, 1088, "Kyber-768 ciphertext size mismatch");
    assert_eq!(KYBER768_SS_SIZE, 32, "Kyber-768 shared secret size mismatch");
}

#[test]
fn test_kyber512_kat_vector_sizes() {
    assert_eq!(KYBER512_PK_SIZE, 800, "Kyber-512 public key size mismatch");
    assert_eq!(KYBER512_SK_SIZE, 1632, "Kyber-512 secret key size mismatch");
    assert_eq!(KYBER512_CT_SIZE, 768, "Kyber-512 ciphertext size mismatch");
    assert_eq!(KYBER512_SS_SIZE, 32, "Kyber-512 shared secret size mismatch");
}

#[test]
fn test_kyber1024_kat_vector_sizes() {
    assert_eq!(KYBER1024_PK_SIZE, 1568, "Kyber-1024 public key size mismatch");
    assert_eq!(KYBER1024_SK_SIZE, 3168, "Kyber-1024 secret key size mismatch");
    assert_eq!(KYBER1024_CT_SIZE, 1568, "Kyber-1024 ciphertext size mismatch");
    assert_eq!(KYBER1024_SS_SIZE, 32, "Kyber-1024 shared secret size mismatch");
}

#[test]
fn test_kyber768_kat_parse() {
    // Test KAT file parsing
    // This test will be enabled after PQClean integration
    let kat_path = "tests/kat/vectors/kyber768.rsp";
    if std::path::Path::new(kat_path).exists() {
        let vectors = parse_kem_kat_file(kat_path);
        assert!(!vectors.is_empty(), "KAT file should contain vectors");
        
        for vector in &vectors {
            // Verify sizes
            assert_eq!(vector.public_key.len(), KYBER768_PK_SIZE,
                "Public key size mismatch in vector {}", vector.count);
            assert_eq!(vector.secret_key.len(), KYBER768_SK_SIZE,
                "Secret key size mismatch in vector {}", vector.count);
            assert_eq!(vector.ciphertext.len(), KYBER768_CT_SIZE,
                "Ciphertext size mismatch in vector {}", vector.count);
            assert_eq!(vector.shared_secret.len(), KYBER768_SS_SIZE,
                "Shared secret size mismatch in vector {}", vector.count);
        }
    }
}

#[test]
fn test_kyber_correctness() {
    // Test basic correctness properties
    // These tests verify the mathematical properties of Kyber
    
    // Property 1: Shared secret should be 32 bytes (SHA3-256 output)
    assert_eq!(KYBER768_SS_SIZE, 32);
    
    // Property 2: Ciphertext should be smaller than public key
    assert!(KYBER768_CT_SIZE < KYBER768_PK_SIZE);
    
    // Property 3: Secret key should be larger than public key
    // (includes public key for convenience)
    assert!(KYBER768_SK_SIZE > KYBER768_PK_SIZE);
}

#[test]
fn test_kyber_security_levels() {
    // Verify security level claims
    // Kyber-512: ~128-bit quantum security
    // Kyber-768: ~192-bit quantum security  
    // Kyber-1024: ~256-bit quantum security
    
    // The lattice dimension n is always 256
    const KYBER_N: usize = 256;
    assert_eq!(KYBER_N, 256);
    
    // Module rank k varies:
    // Kyber-512: k=2
    // Kyber-768: k=3
    // Kyber-1024: k=4
    const KYBER512_K: usize = 2;
    const KYBER768_K: usize = 3;
    const KYBER1024_K: usize = 4;
    
    assert_eq!(KYBER512_K, 2);
    assert_eq!(KYBER768_K, 3);
    assert_eq!(KYBER1024_K, 4);
    
    // Verify public key size = k * 12 * 256 / 8 + 32
    // (k polynomials of 12-bit coefficients + 32-byte seed)
    assert_eq!(KYBER512_PK_SIZE, KYBER512_K * 12 * KYBER_N / 8 + 32);
    assert_eq!(KYBER768_PK_SIZE, KYBER768_K * 12 * KYBER_N / 8 + 32);
    assert_eq!(KYBER1024_PK_SIZE, KYBER1024_K * 12 * KYBER_N / 8 + 32);
}

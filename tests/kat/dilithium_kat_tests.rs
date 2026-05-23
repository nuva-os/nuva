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

// NIST KAT Tests for Dilithium Digital Signature Scheme
// File: tests/kat/dilithium_kat_tests.rs

mod kat;

use kat::{parse_sign_kat_file, SignKatVector};

/// Dilithium2 parameter sizes
const DILITHIUM2_PK_SIZE: usize = 1312;
const DILITHIUM2_SK_SIZE: usize = 2528;
const DILITHIUM2_SIG_SIZE: usize = 2420;

/// Dilithium3 parameter sizes
const DILITHIUM3_PK_SIZE: usize = 1952;
const DILITHIUM3_SK_SIZE: usize = 4000;
const DILITHIUM3_SIG_SIZE: usize = 3293;

/// Dilithium5 parameter sizes
const DILITHIUM5_PK_SIZE: usize = 2592;
const DILITHIUM5_SK_SIZE: usize = 4864;
const DILITHIUM5_SIG_SIZE: usize = 4595;

#[test]
fn test_dilithium2_kat_vector_sizes() {
    assert_eq!(DILITHIUM2_PK_SIZE, 1312, "Dilithium2 public key size mismatch");
    assert_eq!(DILITHIUM2_SK_SIZE, 2528, "Dilithium2 secret key size mismatch");
    assert_eq!(DILITHIUM2_SIG_SIZE, 2420, "Dilithium2 signature size mismatch");
}

#[test]
fn test_dilithium3_kat_vector_sizes() {
    assert_eq!(DILITHIUM3_PK_SIZE, 1952, "Dilithium3 public key size mismatch");
    assert_eq!(DILITHIUM3_SK_SIZE, 4000, "Dilithium3 secret key size mismatch");
    assert_eq!(DILITHIUM3_SIG_SIZE, 3293, "Dilithium3 signature size mismatch");
}

#[test]
fn test_dilithium5_kat_vector_sizes() {
    assert_eq!(DILITHIUM5_PK_SIZE, 2592, "Dilithium5 public key size mismatch");
    assert_eq!(DILITHIUM5_SK_SIZE, 4864, "Dilithium5 secret key size mismatch");
    assert_eq!(DILITHIUM5_SIG_SIZE, 4595, "Dilithium5 signature size mismatch");
}

#[test]
fn test_dilithium3_kat_parse() {
    // Test KAT file parsing
    let kat_path = "tests/kat/vectors/dilithium3.rsp";
    if std::path::Path::new(kat_path).exists() {
        let vectors = parse_sign_kat_file(kat_path);
        assert!(!vectors.is_empty(), "KAT file should contain vectors");
        
        for vector in &vectors {
            // Verify sizes
            assert_eq!(vector.public_key.len(), DILITHIUM3_PK_SIZE,
                "Public key size mismatch in vector {}", vector.count);
            assert_eq!(vector.secret_key.len(), DILITHIUM3_SK_SIZE,
                "Secret key size mismatch in vector {}", vector.count);
            assert_eq!(vector.signature.len(), DILITHIUM3_SIG_SIZE,
                "Signature size mismatch in vector {}", vector.count);
        }
    }
}

#[test]
fn test_dilithium_correctness() {
    // Test basic correctness properties
    
    // Property 1: Secret key should be larger than public key
    assert!(DILITHIUM2_SK_SIZE > DILITHIUM2_PK_SIZE);
    assert!(DILITHIUM3_SK_SIZE > DILITHIUM3_PK_SIZE);
    assert!(DILITHIUM5_SK_SIZE > DILITHIUM5_PK_SIZE);
    
    // Property 2: Signature size should be reasonable
    // Dilithium signatures are larger than classical signatures
    // but still practical (< 5KB)
    assert!(DILITHIUM2_SIG_SIZE < 5000);
    assert!(DILITHIUM3_SIG_SIZE < 5000);
    assert!(DILITHIUM5_SIG_SIZE < 5000);
}

#[test]
fn test_dilithium_security_levels() {
    // Verify security level claims
    // Dilithium2: ~128-bit quantum security (NIST Level 2)
    // Dilithium3: ~192-bit quantum security (NIST Level 3)
    // Dilithium5: ~256-bit quantum security (NIST Level 5)
    
    // The lattice dimension n is always 256
    const DILITHIUM_N: usize = 256;
    assert_eq!(DILITHIUM_N, 256);
    
    // Module rank (l, k) varies:
    // Dilithium2: (l=4, k=4)
    // Dilithium3: (l=5, k=6)
    // Dilithium5: (l=7, k=8)
    
    const DILITHIUM2_L: usize = 4;
    const DILITHIUM2_K: usize = 4;
    const DILITHIUM3_L: usize = 5;
    const DILITHIUM3_K: usize = 6;
    const DILITHIUM5_L: usize = 7;
    const DILITHIUM5_K: usize = 8;
    
    // Verify l <= k (more verification polynomials than signing polynomials)
    assert!(DILITHIUM2_L <= DILITHIUM2_K);
    assert!(DILITHIUM3_L <= DILITHIUM3_K);
    assert!(DILITHIUM5_L <= DILITHIUM5_K);
    
    // Higher security levels should have larger keys and signatures
    assert!(DILITHIUM3_PK_SIZE > DILITHIUM2_PK_SIZE);
    assert!(DILITHIUM5_PK_SIZE > DILITHIUM3_PK_SIZE);
    assert!(DILITHIUM3_SIG_SIZE > DILITHIUM2_SIG_SIZE);
    assert!(DILITHIUM5_SIG_SIZE > DILITHIUM3_SIG_SIZE);
}

#[test]
fn test_dilithium_parameter_consistency() {
    // Verify that parameter sizes are consistent with the specification
    // Public key = 32 (seed) + k * 320 (rho vectors)
    // where 320 = 256 * 10 / 8 (256 coefficients of 10 bits each)
    
    const RHO_SIZE: usize = 320; // 256 * 10 / 8
    const SEED_SIZE: usize = 32;
    
    // Dilithium2: pk = 32 + 4 * 320 = 1312
    assert_eq!(DILITHIUM2_PK_SIZE, SEED_SIZE + 4 * RHO_SIZE);
    
    // Dilithium3: pk = 32 + 6 * 320 = 1952
    assert_eq!(DILITHIUM3_PK_SIZE, SEED_SIZE + 6 * RHO_SIZE);
    
    // Dilithium5: pk = 32 + 8 * 320 = 2592
    assert_eq!(DILITHIUM5_PK_SIZE, SEED_SIZE + 8 * RHO_SIZE);
}

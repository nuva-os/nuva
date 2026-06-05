/*
 * Nuva OS - Tests - Quantum - QuantumTests
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
/*
 * Quantum Technology Tests
 *
 * Copyright (C) 2026 Nuva OS Team
 */

use crate::hal::quantum::*;

#[test]
fn test_kyber_key_sizes() {
    // Kyber512
    let variant = KyberVariant::Kyber512;
    assert_eq!(variant.public_key_size(), 800);
    assert_eq!(variant.secret_key_size(), 1632);
    assert_eq!(variant.ciphertext_size(), 768);
    assert_eq!(variant.shared_secret_size(), 32);
    
    // Kyber768
    let variant = KyberVariant::Kyber768;
    assert_eq!(variant.public_key_size(), 1184);
    assert_eq!(variant.secret_key_size(), 2400);
    assert_eq!(variant.ciphertext_size(), 1088);
    
    // Kyber1024
    let variant = KyberVariant::Kyber1024;
    assert_eq!(variant.public_key_size(), 1568);
    assert_eq!(variant.secret_key_size(), 3168);
    assert_eq!(variant.ciphertext_size(), 1568);
}

#[test]
fn test_dilithium_key_sizes() {
    // Dilithium2
    let variant = DilithiumVariant::Dilithium2;
    assert_eq!(variant.public_key_size(), 1312);
    assert_eq!(variant.secret_key_size(), 2560);
    assert_eq!(variant.signature_size(), 2420);
    
    // Dilithium3
    let variant = DilithiumVariant::Dilithium3;
    assert_eq!(variant.public_key_size(), 1952);
    assert_eq!(variant.secret_key_size(), 4032);
    assert_eq!(variant.signature_size(), 3293);
    
    // Dilithium5
    let variant = DilithiumVariant::Dilithium5;
    assert_eq!(variant.public_key_size(), 2592);
    assert_eq!(variant.secret_key_size(), 4864);
    assert_eq!(variant.signature_size(), 4595);
}

#[test]
fn test_secret_key_zeroization() {
    let mut key = SecretKey {
        data: vec![0x42; 32],
        algorithm: PqcAlgorithm::Kyber768,
    };
    
    // Verify initial data
    assert!(key.data.iter().all(|&b| b == 0x42));
    
    // Zeroize
    key.zeroize();
    
    // Verify zeroized
    assert!(key.data.iter().all(|&b| b == 0));
}

#[test]
fn test_shared_secret_zeroization() {
    let mut secret = SharedSecret {
        data: vec![0xAB; 32],
    };
    
    // Verify initial data
    assert!(secret.data.iter().all(|&b| b == 0xAB));
    
    // Zeroize
    secret.zeroize();
    
    // Verify zeroized
    assert!(secret.data.iter().all(|&b| b == 0));
}

#[test]
fn test_pqc_algorithm_support() {
    let algorithms = vec![
        PqcAlgorithm::Kyber512,
        PqcAlgorithm::Kyber768,
        PqcAlgorithm::Kyber1024,
        PqcAlgorithm::Dilithium2,
        PqcAlgorithm::Dilithium3,
        PqcAlgorithm::Dilithium5,
    ];
    
    // Verify all algorithms are distinct
    for (i, algo1) in algorithms.iter().enumerate() {
        for (j, algo2) in algorithms.iter().enumerate() {
            if i != j {
                assert_ne!(algo1, algo2);
            }
        }
    }
}

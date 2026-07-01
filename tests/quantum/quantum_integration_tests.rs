/*
 * Nuva OS - Tests - Quantum - QuantumIntegrationTests
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
 * Quantum Algorithm Integration Tests
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Comprehensive tests for CRYSTALS-Kyber and CRYSTALS-Dilithium
 */

use crate::hal::quantum::pqc::*;

#[cfg(test)]
mod kyber_tests {
    use super::*;
    
    /// Test Kyber-512 key generation
    #[test]
    fn test_kyber512_keygen() {
        let kyber = Kyber::new(KyberVariant::Kyber512);
        
        let result = kyber.keygen();
        assert!(result.is_ok());
        
        let (pk, sk) = result.unwrap();
        
        // Check key sizes
        assert_eq!(pk.data.len(), 800);
        assert_eq!(sk.data.len(), 1632);
        
        // Check algorithm type
        assert_eq!(pk.algorithm, PqcAlgorithm::Kyber512);
        assert_eq!(sk.algorithm, PqcAlgorithm::Kyber512);
    }
    
    /// Test Kyber-768 key generation
    #[test]
    fn test_kyber768_keygen() {
        let kyber = Kyber::new(KyberVariant::Kyber768);
        
        let result = kyber.keygen();
        assert!(result.is_ok());
        
        let (pk, sk) = result.unwrap();
        
        // Check key sizes
        assert_eq!(pk.data.len(), 1184);
        assert_eq!(sk.data.len(), 2400);
        
        // Check algorithm type
        assert_eq!(pk.algorithm, PqcAlgorithm::Kyber768);
        assert_eq!(sk.algorithm, PqcAlgorithm::Kyber768);
    }
    
    /// Test Kyber-1024 key generation
    #[test]
    fn test_kyber1024_keygen() {
        let kyber = Kyber::new(KyberVariant::Kyber1024);
        
        let result = kyber.keygen();
        assert!(result.is_ok());
        
        let (pk, sk) = result.unwrap();
        
        // Check key sizes
        assert_eq!(pk.data.len(), 1568);
        assert_eq!(sk.data.len(), 3168);
        
        // Check algorithm type
        assert_eq!(pk.algorithm, PqcAlgorithm::Kyber1024);
        assert_eq!(sk.algorithm, PqcAlgorithm::Kyber1024);
    }
    
    /// Test Kyber encapsulation and decapsulation
    #[test]
    fn test_kyber_encapsulate_decapsulate() {
        let kyber = Kyber::new(KyberVariant::Kyber768);
        
        // Generate keys
        let (pk, sk) = kyber.keygen().unwrap();
        
        // Encapsulate
        let encap_result = kyber.encapsulate(&pk);
        assert!(encap_result.is_ok());
        
        let (ss1, ct) = encap_result.unwrap();
        
        // Check shared secret size
        assert_eq!(ss1.data.len(), 32);
        
        // Check ciphertext size
        assert_eq!(ct.data.len(), 1088);
        
        // Decapsulate
        let decap_result = kyber.decapsulate(&sk, &ct);
        assert!(decap_result.is_ok());
        
        let ss2 = decap_result.unwrap();
        
        // Verify shared secrets match
        assert_eq!(ss1.data, ss2.data);
    }
    
    /// Test Kyber-512 full cycle
    #[test]
    fn test_kyber512_full_cycle() {
        test_kyber_full_cycle(KyberVariant::Kyber512);
    }
    
    /// Test Kyber-768 full cycle
    #[test]
    fn test_kyber768_full_cycle() {
        test_kyber_full_cycle(KyberVariant::Kyber768);
    }
    
    /// Test Kyber-1024 full cycle
    #[test]
    fn test_kyber1024_full_cycle() {
        test_kyber_full_cycle(KyberVariant::Kyber1024);
    }
    
    /// Helper function for full cycle test
    fn test_kyber_full_cycle(variant: KyberVariant) {
        let kyber = Kyber::new(variant);
        
        // Key generation
        let (pk, sk) = kyber.keygen().expect("KeyGen failed");
        
        // Encapsulation
        let (ss1, ct) = kyber.encapsulate(&pk).expect("Encapsulate failed");
        
        // Decapsulation
        let ss2 = kyber.decapsulate(&sk, &ct).expect("Decapsulate failed");
        
        // Verify
        assert_eq!(ss1.data, ss2.data, "Shared secrets do not match");
    }
    
    /// Test Kyber variant sizes
    #[test]
    fn test_kyber_variant_sizes() {
        // Kyber512
        let v = KyberVariant::Kyber512;
        assert_eq!(v.public_key_size(), 800);
        assert_eq!(v.secret_key_size(), 1632);
        assert_eq!(v.ciphertext_size(), 768);
        assert_eq!(v.shared_secret_size(), 32);
        
        // Kyber768
        let v = KyberVariant::Kyber768;
        assert_eq!(v.public_key_size(), 1184);
        assert_eq!(v.secret_key_size(), 2400);
        assert_eq!(v.ciphertext_size(), 1088);
        assert_eq!(v.shared_secret_size(), 32);
        
        // Kyber1024
        let v = KyberVariant::Kyber1024;
        assert_eq!(v.public_key_size(), 1568);
        assert_eq!(v.secret_key_size(), 3168);
        assert_eq!(v.ciphertext_size(), 1568);
        assert_eq!(v.shared_secret_size(), 32);
    }
    
    /// Test Kyber security levels
    #[test]
    fn test_kyber_security_levels() {
        assert_eq!(KyberVariant::Kyber512.security_level(), 128);
        assert_eq!(KyberVariant::Kyber768.security_level(), 192);
        assert_eq!(KyberVariant::Kyber1024.security_level(), 256);
    }
}

#[cfg(test)]
mod dilithium_tests {
    use super::*;
    
    /// Test Dilithium-2 key generation
    #[test]
    fn test_dilithium2_keygen() {
        let dilithium = Dilithium::new(DilithiumVariant::Dilithium2);
        
        let result = dilithium.keygen();
        assert!(result.is_ok());
        
        let (pk, sk) = result.unwrap();
        
        // Check key sizes
        assert_eq!(pk.data.len(), 1312);
        assert_eq!(sk.data.len(), 2560);
        
        // Check algorithm type
        assert_eq!(pk.algorithm, PqcAlgorithm::Dilithium2);
        assert_eq!(sk.algorithm, PqcAlgorithm::Dilithium2);
    }
    
    /// Test Dilithium-3 key generation
    #[test]
    fn test_dilithium3_keygen() {
        let dilithium = Dilithium::new(DilithiumVariant::Dilithium3);
        
        let result = dilithium.keygen();
        assert!(result.is_ok());
        
        let (pk, sk) = result.unwrap();
        
        // Check key sizes
        assert_eq!(pk.data.len(), 1952);
        assert_eq!(sk.data.len(), 4032);
        
        // Check algorithm type
        assert_eq!(pk.algorithm, PqcAlgorithm::Dilithium3);
        assert_eq!(sk.algorithm, PqcAlgorithm::Dilithium3);
    }
    
    /// Test Dilithium-5 key generation
    #[test]
    fn test_dilithium5_keygen() {
        let dilithium = Dilithium::new(DilithiumVariant::Dilithium5);
        
        let result = dilithium.keygen();
        assert!(result.is_ok());
        
        let (pk, sk) = result.unwrap();
        
        // Check key sizes
        assert_eq!(pk.data.len(), 2592);
        assert_eq!(sk.data.len(), 4864);
        
        // Check algorithm type
        assert_eq!(pk.algorithm, PqcAlgorithm::Dilithium5);
        assert_eq!(sk.algorithm, PqcAlgorithm::Dilithium5);
    }
    
    /// Test Dilithium sign and verify
    #[test]
    fn test_dilithium_sign_verify() {
        let dilithium = Dilithium::new(DilithiumVariant::Dilithium3);
        
        // Generate keys
        let (pk, sk) = dilithium.keygen().unwrap();
        
        // Message to sign
        let message = b"Hello, Quantum World!";
        
        // Sign
        let sign_result = dilithium.sign(&sk, message);
        assert!(sign_result.is_ok());
        
        let signature = sign_result.unwrap();
        
        // Check signature size
        assert_eq!(signature.data.len(), 3293);
        
        // Verify
        let verify_result = dilithium.verify(&pk, message, &signature);
        assert!(verify_result.is_ok());
        
        let valid = verify_result.unwrap();
        assert!(valid, "Signature verification failed");
    }
    
    /// Test Dilithium-2 full cycle
    #[test]
    fn test_dilithium2_full_cycle() {
        test_dilithium_full_cycle(DilithiumVariant::Dilithium2);
    }
    
    /// Test Dilithium-3 full cycle
    #[test]
    fn test_dilithium3_full_cycle() {
        test_dilithium_full_cycle(DilithiumVariant::Dilithium3);
    }
    
    /// Test Dilithium-5 full cycle
    #[test]
    fn test_dilithium5_full_cycle() {
        test_dilithium_full_cycle(DilithiumVariant::Dilithium5);
    }
    
    /// Helper function for full cycle test
    fn test_dilithium_full_cycle(variant: DilithiumVariant) {
        let dilithium = Dilithium::new(variant);
        
        // Key generation
        let (pk, sk) = dilithium.keygen().expect("KeyGen failed");
        
        // Message
        let message = b"Test message for Dilithium";
        
        // Sign
        let signature = dilithium.sign(&sk, message).expect("Sign failed");
        
        // Verify
        let valid = dilithium.verify(&pk, message, &signature).expect("Verify failed");
        
        // Check
        assert!(valid, "Signature verification failed");
    }
    
    /// Test Dilithium with wrong message
    #[test]
    fn test_dilithium_wrong_message() {
        let dilithium = Dilithium::new(DilithiumVariant::Dilithium3);
        
        let (pk, sk) = dilithium.keygen().unwrap();
        
        let message1 = b"Original message";
        let message2 = b"Different message";
        
        // Sign message1
        let signature = dilithium.sign(&sk, message1).unwrap();
        
        // Verify with message2 (should fail)
        let valid = dilithium.verify(&pk, message2, &signature).unwrap();
        
        assert!(!valid, "Verification should fail with wrong message");
    }
    
    /// Test Dilithium variant sizes
    #[test]
    fn test_dilithium_variant_sizes() {
        // Dilithium2
        let v = DilithiumVariant::Dilithium2;
        assert_eq!(v.public_key_size(), 1312);
        assert_eq!(v.secret_key_size(), 2560);
        assert_eq!(v.signature_size(), 2420);
        
        // Dilithium3
        let v = DilithiumVariant::Dilithium3;
        assert_eq!(v.public_key_size(), 1952);
        assert_eq!(v.secret_key_size(), 4032);
        assert_eq!(v.signature_size(), 3293);
        
        // Dilithium5
        let v = DilithiumVariant::Dilithium5;
        assert_eq!(v.public_key_size(), 2592);
        assert_eq!(v.secret_key_size(), 4864);
        assert_eq!(v.signature_size(), 4595);
    }
    
    /// Test Dilithium security levels
    #[test]
    fn test_dilithium_security_levels() {
        assert_eq!(DilithiumVariant::Dilithium2.security_level(), 128);
        assert_eq!(DilithiumVariant::Dilithium3.security_level(), 192);
        assert_eq!(DilithiumVariant::Dilithium5.security_level(), 256);
    }
}

#[cfg(test)]
mod key_security_tests {
    use super::*;
    
    /// Test secret key zeroization
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
    
    /// Test shared secret zeroization
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
    
    /// Test secret key drop auto-zeroization
    #[test]
    fn test_secret_key_drop() {
        let key = SecretKey {
            data: vec![0x42; 32],
            algorithm: PqcAlgorithm::Kyber768,
        };
        
        // Drop should auto-zeroize
        drop(key);
        
        // Memory should be zeroized (can't directly verify)
    }
}

#[cfg(test)]
mod algorithm_support_tests {
    use super::*;
use alloc::vec;
    
    /// Test PQC algorithm equality
    #[test]
    fn test_pqc_algorithm_equality() {
        assert_eq!(PqcAlgorithm::Kyber512, PqcAlgorithm::Kyber512);
        assert_ne!(PqcAlgorithm::Kyber512, PqcAlgorithm::Kyber768);
        assert_ne!(PqcAlgorithm::Kyber768, PqcAlgorithm::Dilithium3);
    }
    
    /// Test all PQC algorithms are distinct
    #[test]
    fn test_pqc_algorithms_distinct() {
        let algorithms = vec![
            PqcAlgorithm::Kyber512,
            PqcAlgorithm::Kyber768,
            PqcAlgorithm::Kyber1024,
            PqcAlgorithm::Dilithium2,
            PqcAlgorithm::Dilithium3,
            PqcAlgorithm::Dilithium5,
        ];
        
        // Check all are distinct
        for (i, algo1) in algorithms.iter().enumerate() {
            for (j, algo2) in algorithms.iter().enumerate() {
                if i != j {
                    assert_ne!(algo1, algo2);
                }
            }
        }
    }
}

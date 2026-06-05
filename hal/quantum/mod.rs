/*
 * Nuva OS - Hal - Quantum - Mod
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
 * Quantum Technology HAL
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides hardware abstraction for quantum
 * technologies including QRNG and PQC.
 */

pub mod pqc;
pub mod qkd;
pub mod qrng;
pub mod security;

// Re-export main types
pub use pqc::{
    PqcProvider, KyberVariant, DilithiumVariant, PqcAlgorithm,
    PublicKey, SecretKey, SharedSecret, Ciphertext, Signature, PqcError,
};
pub use qkd::{QkdSession, QkdConfig, QkdKey, QkdManager, QkdError, Bb84Basis, QkdChannel, init_qkd};
pub use qrng::{QrngProvider, RandomnessQuality, QrngError};
pub use security::{QuantumSafeSecurity, SecurityConfig, SecurityLevel};

/// Initialize quantum HAL subsystem
pub fn init_quantum_hal() -> Result<(), QuantumError> {
    // Initialize QRNG hardware subsystem
    qrng::hardware::init_hardware_qrng();

    // Initialize QKD BB84 protocol subsystem
    init_qkd();

    // Initialize PQC provider
    // The PQC provider is initialized as part of QuantumSafeSecurity::new()
    // which creates Kyber and Dilithium instances with the configured variants

    // Initialize quantum-safe security
    let config = SecurityConfig::default();
    let _security = QuantumSafeSecurity::new(config).map_err(QuantumError::PqcError)?;

    Ok(())
}

/// Quantum error type
#[derive(Debug, Clone)]
pub enum QuantumError {
    /// QRNG error
    QrngError(QrngError),

    /// PQC error
    PqcError(PqcError),

    /// Initialization failed
    InitFailed,

    /// Hardware not available
    HardwareNotAvailable,
}

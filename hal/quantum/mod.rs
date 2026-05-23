/*
 * Quantum Technology HAL
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides hardware abstraction for quantum
 * technologies including QRNG and PQC.
 */

pub mod pqc;
pub mod qrng;
pub mod security;

// Re-export main types
pub use pqc::{
    PqcProvider, KyberVariant, DilithiumVariant, PqcAlgorithm,
    PublicKey, SecretKey, SharedSecret, Ciphertext, Signature, PqcError,
};
pub use qrng::{QrngProvider, RandomnessQuality, QrngError};
pub use security::{QuantumSafeSecurity, SecurityConfig, SecurityLevel};

/// Initialize quantum HAL subsystem
pub fn init_quantum_hal() -> Result<(), QuantumError> {
    // Initialize QRNG
    // Detect hardware QRNG availability
    // In a real implementation, this would:
    // 1. Check for hardware QRNG device (e.g., via device tree or ACPI)
    // 2. If available, initialize the hardware QRNG driver
    // 3. If not available, fall back to software PRNG with entropy collection
    // For now, we use the software-based DummyQrngProvider
    
    // Initialize PQC provider
    // The PQC provider is initialized as part of QuantumSafeSecurity::new()
    // which creates Kyber and Dilithium instances with the configured variants
    // No separate initialization step is needed

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

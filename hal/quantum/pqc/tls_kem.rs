/*
 * Kyber-768 TLS Key Encapsulation Integration
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Integrates Kyber-768 KEM into TLS 1.3 handshake as defined
 * in draft-ietf-tls-hybrid-design and RFC 9180.
 *
 * Supports:
 * - Pure Kyber-768 KEM in TLS
 * - X25519+Kyber768 hybrid key exchange (recommended)
 * - Backward-compatible fallback to X25519
 */

use core::sync::atomic::{AtomicU8, Ordering};
use alloc::vec::Vec;
use alloc::string::String;

use super::hybrid::{
    HybridKem, HybridKeyPair, HybridCiphertext, HybridSharedSecret,
    HybridKemError, HYBRID_SHARED_SECRET_SIZE,
};
use super::kyber::{
    Kyber, KyberVariant, PublicKey as KyberPublicKey,
    SecretKey as KyberSecretKey, Ciphertext as KyberCiphertext,
    SharedSecret as KyberSharedSecret, KyberError,
};

/// TLS KEM operations trait
/// Abstract interface for key encapsulation in TLS handshake.
/// Implementations can be classical (ECDH), post-quantum (Kyber),
/// or hybrid (X25519+Kyber).
pub trait TlsKemOps {
    /// Generate keypair for KEM
    fn generate_keypair(&mut self) -> Result<TlsKemKeyPair, TlsKemError>;

    /// Encapsulate: generate shared secret and ciphertext from peer's public key
    fn encapsulate(&mut self, peer_pk: &TlsKemPublicKey) -> Result<(TlsKemCiphertext, TlsKemSharedSecret), TlsKemError>;

    /// Decapsulate: recover shared secret from ciphertext using secret key
    fn decapsulate(&mut self, sk: &TlsKemSecretKey, ct: &TlsKemCiphertext) -> Result<TlsKemSharedSecret, TlsKemError>;

    /// Get KEM algorithm identifier for TLS
    fn kem_id(&self) -> TlsKemId;
}

/// TLS KEM algorithm identifiers
/// Allocated from the TLS Supported Groups registry
/// (draft-ietf-tls-hybrid-design)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsKemId {
    /// X25519 (0x001D)
    X25519 = 0x001D,
    /// Kyber-768 (0x0D38, provisional)
    Kyber768 = 0x0D38,
    /// X25519+Kyber768 hybrid (0x6399, draft-ietf-tls-hybrid-design)
    X25519Kyber768 = 0x6399,
    /// SecP256r1+Kyber768 (0x639A)
    SecP256r1Kyber768 = 0x639A,
}

/// TLS KEM error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsKemError {
    /// Key generation failed
    KeygenFailed,
    /// Encapsulation failed
    EncapsulateFailed,
    /// Decapsulation failed
    DecapsulateFailed,
    /// Invalid public key
    InvalidPublicKey,
    /// Invalid secret key
    InvalidSecretKey,
    /// Invalid ciphertext
    InvalidCiphertext,
    /// Hybrid KEM error
    HybridError(HybridKemError),
    /// Kyber error
    KyberError(KyberError),
    /// Protocol error
    ProtocolError,
}

/// TLS KEM public key (variable length, max Kyber1024 pk)
#[derive(Debug, Clone)]
pub struct TlsKemPublicKey {
    /// Key data
    pub data: Vec<u8>,
    /// KEM algorithm
    pub kem_id: TlsKemId,
}

/// TLS KEM secret key (variable length)
#[derive(Debug, Clone)]
pub struct TlsKemSecretKey {
    /// Key data
    pub data: Vec<u8>,
    /// KEM algorithm
    pub kem_id: TlsKemId,
}

/// TLS KEM key pair
#[derive(Debug, Clone)]
pub struct TlsKemKeyPair {
    /// Public key
    pub public_key: TlsKemPublicKey,
    /// Secret key
    pub secret_key: TlsKemSecretKey,
}

/// TLS KEM ciphertext
#[derive(Debug, Clone)]
pub struct TlsKemCiphertext {
    /// Ciphertext data
    pub data: Vec<u8>,
    /// KEM algorithm
    pub kem_id: TlsKemId,
}

/// TLS KEM shared secret
#[derive(Debug, Clone)]
pub struct TlsKemSharedSecret {
    /// Shared secret data (32 bytes)
    pub data: Vec<u8>,
}

/// Kyber-768 TLS KEM implementation
/// Implements TlsKemOps using pure Kyber-768.
/// Suitable for environments where only PQ security is needed.
pub struct Kyber768TlsKem {
    /// Kyber instance
    kyber: Kyber,
}

impl Kyber768TlsKem {
    /// Create new Kyber-768 TLS KEM
    pub fn new() -> Self {
        Kyber768TlsKem {
            kyber: Kyber::new(KyberVariant::Kyber768),
        }
    }
}

impl TlsKemOps for Kyber768TlsKem {
    fn generate_keypair(&mut self) -> Result<TlsKemKeyPair, TlsKemError> {
        let (pk, sk) = self.kyber.keygen()
            .map_err(TlsKemError::KyberError)?;

        Ok(TlsKemKeyPair {
            public_key: TlsKemPublicKey {
                data: pk.as_bytes().to_vec(),
                kem_id: TlsKemId::Kyber768,
            },
            secret_key: TlsKemSecretKey {
                data: sk.as_bytes().to_vec(),
                kem_id: TlsKemId::Kyber768,
            },
        })
    }

    fn encapsulate(&mut self, peer_pk: &TlsKemPublicKey) -> Result<(TlsKemCiphertext, TlsKemSharedSecret), TlsKemError> {
        let mut kyber_pk = KyberPublicKey::new(KyberVariant::Kyber768);
        let pk_len = peer_pk.data.len().min(1184);
        // SAFETY: copying peer key bytes into Kyber public key buffer of correct size
        unsafe {
            core::ptr::copy_nonoverlapping(peer_pk.data.as_ptr(), kyber_pk.as_mut_ptr(), pk_len);
        }

        let (ct, ss) = self.kyber.encapsulate(&kyber_pk)
            .map_err(TlsKemError::KyberError)?;

        Ok((
            TlsKemCiphertext {
                data: ct.as_bytes().to_vec(),
                kem_id: TlsKemId::Kyber768,
            },
            TlsKemSharedSecret {
                data: ss.as_bytes().to_vec(),
            },
        ))
    }

    fn decapsulate(&mut self, sk: &TlsKemSecretKey, ct: &TlsKemCiphertext) -> Result<TlsKemSharedSecret, TlsKemError> {
        let mut kyber_sk = KyberSecretKey::new(KyberVariant::Kyber768);
        let sk_len = sk.data.len().min(2400);
        // SAFETY: copying secret key bytes into Kyber secret key buffer of correct size
        unsafe {
            core::ptr::copy_nonoverlapping(sk.data.as_ptr(), kyber_sk.as_mut_ptr(), sk_len);
        }

        let mut kyber_ct = KyberCiphertext::new(KyberVariant::Kyber768);
        let ct_len = ct.data.len().min(1088);
        // SAFETY: copying ciphertext bytes into Kyber ciphertext buffer of correct size
        unsafe {
            core::ptr::copy_nonoverlapping(ct.data.as_ptr(), kyber_ct.as_mut_ptr(), ct_len);
        }

        let ss = self.kyber.decapsulate(&kyber_sk, &kyber_ct)
            .map_err(TlsKemError::KyberError)?;

        Ok(TlsKemSharedSecret {
            data: ss.as_bytes().to_vec(),
        })
    }

    fn kem_id(&self) -> TlsKemId {
        TlsKemId::Kyber768
    }
}

/// X25519+Kyber768 hybrid key exchange for TLS
/// Implements TlsKemOps using hybrid X25519+Kyber768.
/// This is the RECOMMENDED configuration for TLS 1.3:
/// - Security if either X25519 OR Kyber-768 is unbroken
/// - Backward compatible with existing X25519 deployments
/// - Shared secret = HKDF(SHA3-256, ss_x25519 || ss_kyber)
pub struct TlsHybridKeyExchange {
    /// Hybrid KEM instance
    hybrid_kem: HybridKem,
    /// Current handshake state
    state: AtomicU8,
}

/// TLS handshake states
const STATE_INIT: u8 = 0;
const STATE_CLIENT_HELLO: u8 = 1;
const STATE_SERVER_HELLO: u8 = 2;
const STATE_KEY_EXCHANGE: u8 = 3;
const STATE_FINISHED: u8 = 4;

impl TlsHybridKeyExchange {
    /// Create new hybrid key exchange
    pub fn new() -> Self {
        TlsHybridKeyExchange {
            hybrid_kem: HybridKem::new(),
            state: AtomicU8::new(STATE_INIT),
        }
    }

    /// Enable/disable X25519 fallback mode
    /// When enabled, if Kyber-768 fails, the handshake
    /// falls back to pure X25519 (less secure but still
    /// classical-security).
    pub fn set_fallback(&self, enabled: bool) {
        self.hybrid_kem.set_fallback(enabled);
    }

    /// Get current handshake state
    pub fn state(&self) -> u8 {
        self.state.load(Ordering::Acquire)
    }
}

impl TlsKemOps for TlsHybridKeyExchange {
    fn generate_keypair(&mut self) -> Result<TlsKemKeyPair, TlsKemError> {
        let kp = self.hybrid_kem.hybrid_keygen()
            .map_err(TlsKemError::HybridError)?;

        self.state.store(STATE_CLIENT_HELLO, Ordering::Release);

        let mut pk_data = alloc::vec![];
        pk_data.extend_from_slice(&kp.x25519.public_key);
        pk_data.extend_from_slice(kp.kyber_pk.as_bytes());

        let mut sk_data = alloc::vec![];
        sk_data.extend_from_slice(&kp.x25519.secret_key);
        sk_data.extend_from_slice(kp.kyber_sk.as_bytes());

        Ok(TlsKemKeyPair {
            public_key: TlsKemPublicKey {
                data: pk_data,
                kem_id: TlsKemId::X25519Kyber768,
            },
            secret_key: TlsKemSecretKey {
                data: sk_data,
                kem_id: TlsKemId::X25519Kyber768,
            },
        })
    }

    fn encapsulate(&mut self, peer_pk: &TlsKemPublicKey) -> Result<(TlsKemCiphertext, TlsKemSharedSecret), TlsKemError> {
        if peer_pk.data.len() < 32 {
            return Err(TlsKemError::InvalidPublicKey);
        }

        let mut x25519_pk = [0u8; 32];
        x25519_pk.copy_from_slice(&peer_pk.data[..32]);

        let mut kyber_pk = KyberPublicKey::new(KyberVariant::Kyber768);
        let kyber_pk_len = (peer_pk.data.len() - 32).min(1184);
        // SAFETY: copying peer Kyber public key bytes into correctly-sized buffer
        unsafe {
            core::ptr::copy_nonoverlapping(peer_pk.data[32..].as_ptr(), kyber_pk.as_mut_ptr(), kyber_pk_len);
        }

        let peer_kp = HybridKeyPair {
            x25519: super::hybrid::X25519KeyPair {
                public_key: x25519_pk,
                secret_key: [0u8; 32],
            },
            kyber_pk,
            kyber_sk: KyberSecretKey::new(KyberVariant::Kyber768),
        };

        let (ct, ss) = self.hybrid_kem.hybrid_encapsulate(&peer_kp)
            .map_err(TlsKemError::HybridError)?;

        self.state.store(STATE_SERVER_HELLO, Ordering::Release);

        let mut ct_data = alloc::vec![];
        ct_data.extend_from_slice(&ct.x25519_ct);
        ct_data.extend_from_slice(ct.kyber_ct.as_bytes());

        Ok((
            TlsKemCiphertext {
                data: ct_data,
                kem_id: TlsKemId::X25519Kyber768,
            },
            TlsKemSharedSecret {
                data: ss.data,
            },
        ))
    }

    fn decapsulate(&mut self, sk: &TlsKemSecretKey, ct: &TlsKemCiphertext) -> Result<TlsKemSharedSecret, TlsKemError> {
        if sk.data.len() < 32 || ct.data.len() < 32 {
            return Err(TlsKemError::InvalidSecretKey);
        }

        let mut x25519_sk = [0u8; 32];
        x25519_sk.copy_from_slice(&sk.data[..32]);

        let mut x25519_ct = [0u8; 32];
        x25519_ct.copy_from_slice(&ct.data[..32]);

        let mut kyber_sk = KyberSecretKey::new(KyberVariant::Kyber768);
        let kyber_sk_len = (sk.data.len() - 32).min(2400);
        // SAFETY: copying secret key bytes into correctly-sized Kyber buffer
        unsafe {
            core::ptr::copy_nonoverlapping(sk.data[32..].as_ptr(), kyber_sk.as_mut_ptr(), kyber_sk_len);
        }

        let mut kyber_ct = KyberCiphertext::new(KyberVariant::Kyber768);
        let kyber_ct_len = (ct.data.len() - 32).min(1088);
        // SAFETY: copying ciphertext bytes into correctly-sized Kyber buffer
        unsafe {
            core::ptr::copy_nonoverlapping(ct.data[32..].as_ptr(), kyber_ct.as_mut_ptr(), kyber_ct_len);
        }

        let our_kp = HybridKeyPair {
            x25519: super::hybrid::X25519KeyPair {
                public_key: [0u8; 32],
                secret_key: x25519_sk,
            },
            kyber_pk: KyberPublicKey::new(KyberVariant::Kyber768),
            kyber_sk,
        };

        let hybrid_ct = HybridCiphertext {
            x25519_ct,
            kyber_ct,
        };

        let ss = self.hybrid_kem.hybrid_decapsulate(&our_kp, &hybrid_ct)
            .map_err(TlsKemError::HybridError)?;

        self.state.store(STATE_KEY_EXCHANGE, Ordering::Release);

        Ok(TlsKemSharedSecret {
            data: ss.data,
        })
    }

    fn kem_id(&self) -> TlsKemId {
        TlsKemId::X25519Kyber768
    }
}

/// TLS handshake step enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsHandshakeStep {
    /// ClientHello: send supported groups + Kyber public key
    ClientHello,
    /// ServerHello: select group + send encapsulation
    ServerHello,
    /// ClientFinished: decapsulate + verify
    ClientFinished,
    /// ServerFinished: verify
    ServerFinished,
    /// Application data
    ApplicationData,
}

/// Execute a single TLS handshake step with Kyber KEM
/// This function drives the TLS 1.3 hybrid key exchange
/// through its states, performing the appropriate KEM
/// operation at each step.
pub fn kyber_tls_handshake_step(
    kem: &mut dyn TlsKemOps,
    step: TlsHandshakeStep,
    peer_pk: Option<&TlsKemPublicKey>,
    ct: Option<&TlsKemCiphertext>,
) -> Result<TlsHandshakeResult, TlsKemError> {
    match step {
        TlsHandshakeStep::ClientHello => {
            let kp = kem.generate_keypair()?;
            Ok(TlsHandshakeResult {
                public_key: Some(kp.public_key),
                ciphertext: None,
                shared_secret: None,
                next_step: TlsHandshakeStep::ServerHello,
            })
        }
        TlsHandshakeStep::ServerHello => {
            if let Some(pk) = peer_pk {
                let (ct, ss) = kem.encapsulate(pk)?;
                Ok(TlsHandshakeResult {
                    public_key: None,
                    ciphertext: Some(ct),
                    shared_secret: Some(ss),
                    next_step: TlsHandshakeStep::ClientFinished,
                })
            } else {
                Err(TlsKemError::InvalidPublicKey)
            }
        }
        TlsHandshakeStep::ClientFinished => {
            if let Some(c) = ct {
                let kp = kem.generate_keypair()?;
                let ss = kem.decapsulate(&kp.secret_key, c)?;
                Ok(TlsHandshakeResult {
                    public_key: None,
                    ciphertext: None,
                    shared_secret: Some(ss),
                    next_step: TlsHandshakeStep::ServerFinished,
                })
            } else {
                Err(TlsKemError::InvalidCiphertext)
            }
        }
        TlsHandshakeStep::ServerFinished => {
            Ok(TlsHandshakeResult {
                public_key: None,
                ciphertext: None,
                shared_secret: None,
                next_step: TlsHandshakeStep::ApplicationData,
            })
        }
        TlsHandshakeStep::ApplicationData => {
            Ok(TlsHandshakeResult {
                public_key: None,
                ciphertext: None,
                shared_secret: None,
                next_step: TlsHandshakeStep::ApplicationData,
            })
        }
    }
}

/// TLS handshake step result
#[derive(Debug, Clone)]
pub struct TlsHandshakeResult {
    /// Public key to send (if any)
    pub public_key: Option<TlsKemPublicKey>,
    /// Ciphertext to send (if any)
    pub ciphertext: Option<TlsKemCiphertext>,
    /// Derived shared secret (if available)
    pub shared_secret: Option<TlsKemSharedSecret>,
    /// Next handshake step
    pub next_step: TlsHandshakeStep,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_kem_id_values() {
        assert_eq!(TlsKemId::X25519 as u16, 0x001D);
        assert_eq!(TlsKemId::X25519Kyber768 as u16, 0x6399);
    }

    #[test]
    fn test_kyber768_tls_kem_new() {
        let kem = Kyber768TlsKem::new();
        assert_eq!(kem.kem_id(), TlsKemId::Kyber768);
    }

    #[test]
    fn test_hybrid_key_exchange_new() {
        let kem = TlsHybridKeyExchange::new();
        assert_eq!(kem.kem_id(), TlsKemId::X25519Kyber768);
        assert_eq!(kem.state(), STATE_INIT);
    }

    #[test]
    fn test_hybrid_fallback() {
        let kem = TlsHybridKeyExchange::new();
        kem.set_fallback(false);
        kem.set_fallback(true);
    }

    #[test]
    fn test_handshake_step_client_hello() {
        let mut kem = Kyber768TlsKem::new();
        let result = kyber_tls_handshake_step(
            &mut kem,
            TlsHandshakeStep::ClientHello,
            None,
            None,
        );
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.public_key.is_some());
        assert_eq!(r.next_step, TlsHandshakeStep::ServerHello);
    }

    #[test]
    fn test_handshake_step_application_data() {
        let mut kem = Kyber768TlsKem::new();
        let result = kyber_tls_handshake_step(
            &mut kem,
            TlsHandshakeStep::ApplicationData,
            None,
            None,
        );
        assert!(result.is_ok());
    }
}

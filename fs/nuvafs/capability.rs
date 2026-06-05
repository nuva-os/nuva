/*
 * Nuva OS - NuvaFS Capability Gate
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

//! NuvaFS Capability Gate
//! Nuva OS capability-based access control for filesystem operations.
//! Replaces traditional Linux UID/GID permission model with fine-grained
//! capability tokens that can be delegated and revoked.

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Capability rights (bitflags)
pub const CAP_READ: u64 = 1 << 0;
pub const CAP_WRITE: u64 = 1 << 1;
pub const CAP_EXEC: u64 = 1 << 2;
pub const CAP_DELETE: u64 = 1 << 3;
pub const CAP_SNAPSHOT: u64 = 1 << 4;
pub const CAP_ROLLBACK: u64 = 1 << 5;
pub const CAP_ADMIN: u64 = 1 << 6;
pub const CAP_DELEGATE: u64 = 1 << 7;
pub const CAP_AUDIT: u64 = 1 << 8;
pub const CAP_MOUNT: u64 = 1 << 9;

/// All capability rights
pub const CAP_ALL: u64 = CAP_READ | CAP_WRITE | CAP_EXEC | CAP_DELETE
    | CAP_SNAPSHOT | CAP_ROLLBACK | CAP_ADMIN | CAP_DELEGATE | CAP_AUDIT | CAP_MOUNT;

/// A capability token granting specific rights to a subject over an object.
#[derive(Debug, Clone, Copy)]
pub struct CapabilityToken {
    /// Unique token ID
    pub id: u64,
    /// Subject ID (e.g., process, task, or namespace)
    pub subject: u64,
    /// Object ID (e.g., inode, snapshot, or mount point)
    pub object: u64,
    /// Granted rights (bitmask of CAP_* flags)
    pub rights: u64,
    /// Delegatable rights (subset of rights that can be further delegated)
    pub delegatable: u64,
    /// Whether this token is revoked
    pub revoked: AtomicBool,
    /// Generation for revocation propagation
    pub generation: AtomicU32,
}

impl CapabilityToken {
    /// Create a new capability token
    pub fn new(id: u64, subject: u64, object: u64, rights: u64, delegatable: u64) -> Self {
        // Delegatable must be a subset of rights
        let delegatable = delegatable & rights;
        Self {
            id,
            subject,
            object,
            rights,
            delegatable,
            revoked: AtomicBool::new(false),
            generation: AtomicU32::new(0),
        }
    }

    /// Check if the token is revoked
    pub fn is_revoked(&self) -> bool {
        self.revoked.load(Ordering::Relaxed)
    }

    /// Revoke this token
    pub fn revoke(&self) {
        self.revoked.store(true, Ordering::Relaxed);
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if a specific right is granted
    pub fn has_right(&self, right: u64) -> bool {
        !self.is_revoked() && (self.rights & right) != 0
    }

    /// Check if a set of rights is granted (all must be present)
    pub fn has_rights(&self, rights: u64) -> bool {
        !self.is_revoked() && (self.rights & rights) == rights
    }

    /// Check if a right can be delegated
    pub fn can_delegate(&self, right: u64) -> bool {
        !self.is_revoked() && (self.delegatable & right) != 0
    }
}

/// Capability gate: the central authority for capability-based access control.
pub struct CapabilityGate {
    /// All capability tokens indexed by token ID
    tokens: BTreeMap<u64, CapabilityToken>,
    /// Tokens indexed by (subject, object) for fast lookup
    index: BTreeMap<(u64, u64), Vec<u64>>,
    /// Next token ID allocator
    next_id: AtomicU64,
    /// Total active (non-revoked) tokens
    active_count: AtomicU32,
}

/// Capability gate errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityError {
    /// Token not found
    TokenNotFound,
    /// Token is revoked
    TokenRevoked,
    /// Insufficient rights for the operation
    InsufficientRights,
    /// Cannot delegate the requested rights
    CannotDelegate,
    /// Object not found
    ObjectNotFound,
    /// Subject not found
    SubjectNotFound,
    /// Too many tokens
    TooManyTokens,
}

impl CapabilityGate {
    /// Create a new empty capability gate
    pub fn new() -> Self {
        Self {
            tokens: BTreeMap::new(),
            index: BTreeMap::new(),
            next_id: AtomicU64::new(1),
            active_count: AtomicU32::new(0),
        }
    }

    /// Grant a capability token to a subject for an object.
    pub fn grant(
        &mut self,
        subject: u64,
        object: u64,
        rights: u64,
        delegatable: u64,
    ) -> Result<u64, CapabilityError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let token = CapabilityToken::new(id, subject, object, rights, delegatable);
        self.tokens.insert(id, token);

        let key = (subject, object);
        self.index.entry(key).or_insert_with(Vec::new).push(id);

        self.active_count.fetch_add(1, Ordering::Relaxed);
        Ok(id)
    }

    /// Delegate rights from an existing token to a new subject.
    /// The delegatable subset of the source token determines what can be delegated.
    pub fn delegate(
        &mut self,
        source_token_id: u64,
        new_subject: u64,
        rights: u64,
    ) -> Result<u64, CapabilityError> {
        let (object, delegatable) = {
            let source = self.tokens.get(&source_token_id).ok_or(CapabilityError::TokenNotFound)?;
            if source.is_revoked() {
                return Err(CapabilityError::TokenRevoked);
            }
            if (source.delegatable & rights) != rights {
                return Err(CapabilityError::CannotDelegate);
            }
            (source.object, source.delegatable & rights)
        };

        self.grant(new_subject, object, rights, delegatable)
    }

    /// Revoke a capability token by ID.
    pub fn revoke(&mut self, token_id: u64) -> Result<(), CapabilityError> {
        let token = self.tokens.get(&token_id).ok_or(CapabilityError::TokenNotFound)?;
        if token.is_revoked() {
            return Err(CapabilityError::TokenRevoked);
        }
        token.revoke();
        self.active_count.fetch_sub(1, Ordering::Relaxed);
        Ok(())
    }

    /// Check if a subject has the required rights on an object.
    /// Returns true if any non-revoked token grants all required rights.
    pub fn check(&self, subject: u64, object: u64, required_rights: u64) -> bool {
        if let Some(token_ids) = self.index.get(&(subject, object)) {
            for &tid in token_ids.iter() {
                if let Some(token) = self.tokens.get(&tid) {
                    if token.has_rights(required_rights) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get a capability token by ID
    pub fn get(&self, token_id: u64) -> Option<&CapabilityToken> {
        self.tokens.get(&token_id)
    }

    /// List all tokens for a subject
    pub fn list_subject(&self, subject: u64) -> Vec<u64> {
        let mut result = Vec::new();
        for (&(s, _), token_ids) in self.index.iter() {
            if s == subject {
                for &tid in token_ids.iter() {
                    if let Some(token) = self.tokens.get(&tid) {
                        if !token.is_revoked() {
                            result.push(tid);
                        }
                    }
                }
            }
        }
        result
    }

    /// Get the number of active tokens
    pub fn active_count(&self) -> u32 {
        self.active_count.load(Ordering::Relaxed)
    }

    /// Revoke all tokens for a subject (e.g., on process exit)
    pub fn revoke_all_subject(&mut self, subject: u64) -> u32 {
        let mut count = 0u32;
        // Scan all tokens for this subject
        for (_id, token) in self.tokens.iter() {
            if token.subject == subject && !token.is_revoked() {
                token.revoke();
                count += 1;
            }
        }
        if count > 0 {
            self.active_count.fetch_sub(count, Ordering::Relaxed);
        }
        count
    }
}

impl Default for CapabilityGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_token() {
        let token = CapabilityToken::new(1, 100, 200, CAP_READ | CAP_WRITE, CAP_READ);
        assert!(token.has_right(CAP_READ));
        assert!(token.has_right(CAP_WRITE));
        assert!(!token.has_right(CAP_ADMIN));
        assert!(token.can_delegate(CAP_READ));
        assert!(!token.can_delegate(CAP_WRITE));
    }

    #[test]
    fn test_capability_token_revocation() {
        let token = CapabilityToken::new(1, 100, 200, CAP_READ, 0);
        assert!(!token.is_revoked());
        token.revoke();
        assert!(token.is_revoked());
        assert!(!token.has_right(CAP_READ)); // revoked
    }

    #[test]
    fn test_grant_and_check() {
        let mut gate = CapabilityGate::new();
        let tid = gate.grant(100, 200, CAP_READ | CAP_WRITE, CAP_READ).unwrap();
        assert!(gate.check(100, 200, CAP_READ));
        assert!(gate.check(100, 200, CAP_WRITE));
        assert!(!gate.check(100, 200, CAP_ADMIN));
        assert!(!gate.check(999, 200, CAP_READ)); // wrong subject
    }

    #[test]
    fn test_delegate() {
        let mut gate = CapabilityGate::new();
        let source = gate.grant(100, 200, CAP_READ | CAP_WRITE, CAP_READ).unwrap();
        let _delegated = gate.delegate(source, 300, CAP_READ).unwrap();
        assert!(gate.check(300, 200, CAP_READ));
        assert!(!gate.check(300, 200, CAP_WRITE)); // not delegated
    }

    #[test]
    fn test_delegate_cannot_exceed() {
        let mut gate = CapabilityGate::new();
        let source = gate.grant(100, 200, CAP_READ | CAP_WRITE, CAP_READ).unwrap();
        assert_eq!(gate.delegate(source, 300, CAP_WRITE), Err(CapabilityError::CannotDelegate));
    }

    #[test]
    fn test_revoke() {
        let mut gate = CapabilityGate::new();
        let tid = gate.grant(100, 200, CAP_READ, 0).unwrap();
        assert!(gate.check(100, 200, CAP_READ));
        gate.revoke(tid).unwrap();
        assert!(!gate.check(100, 200, CAP_READ));
    }
}

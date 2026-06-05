/*
 * Nuva OS - Kernel - Security - Credential
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
 * Nuva OS - Kernel - Process Credentials
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Process credentials management.
 */

use core::sync::atomic::AtomicU32;
use super::capability::CapSet;
use super::nsm_manager::SecId;

/// Process credentials
#[repr(C)]
pub struct Credentials {
    /// Real user ID
    pub uid: u32,
    /// Real group ID
    pub gid: u32,
    /// Effective user ID
    pub euid: u32,
    /// Effective group ID
    pub egid: u32,
    /// Saved user ID
    pub suid: u32,
    /// Saved group ID
    pub sgid: u32,
    /// Filesystem user ID
    pub fsuid: u32,
    /// Filesystem group ID
    pub fsgid: u32,
    /// Supplementary groups
    pub groups: [u32; 32],
    /// Number of supplementary groups
    pub ngroups: u8,
    /// Permitted capabilities
    pub cap_permitted: CapSet,
    /// Effective capabilities
    pub cap_effective: CapSet,
    /// Inheritable capabilities
    pub cap_inheritable: CapSet,
    /// Ambient capabilities
    pub cap_ambient: CapSet,
    /// Bounding capabilities
    pub cap_bounding: CapSet,
    /// Secure bits
    pub securebits: u32,
    /// Security ID
    pub sid: SecId,
    /// Reference count
    pub ref_count: AtomicU32,
}

impl Credentials {
    /// Create new credentials for the given user and group
    pub fn new(uid: u32, gid: u32) -> Self {
        Credentials {
            uid,
            gid,
            euid: uid,
            egid: gid,
            suid: uid,
            sgid: gid,
            fsuid: uid,
            fsgid: gid,
            groups: [0; 32],
            ngroups: 0,
            cap_permitted: CapSet::new(),
            cap_effective: CapSet::new(),
            cap_inheritable: CapSet::new(),
            cap_ambient: CapSet::new(),
            cap_bounding: CapSet::new(),
            securebits: 0,
            sid: 0,
            ref_count: AtomicU32::new(1),
        }
    }

    /// Check if the credentials have the given capability
    pub fn has_cap(&self, cap: u32) -> bool {
        self.cap_effective.has(cap)
    }

    /// Check if the credentials grant the requested permission
    pub fn has_perm(&self, uid: u32, gid: u32, mode: u32) -> bool {
        // Root has all permissions
        if self.euid == 0 {
            return true;
        }

        // Owner permission
        if self.euid == uid {
            return (mode & 0x700) != 0;
        }

        // Group permission
        if self.egid == gid {
            return (mode & 0x070) != 0;
        }

        // Check supplementary groups
        for i in 0..self.ngroups as usize {
            if self.groups[i] == gid {
                return (mode & 0x070) != 0;
            }
        }

        // Other permission
        (mode & 0x007) != 0
    }
}

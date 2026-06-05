/*
 * Nuva OS - Kernel - Net - Ndp - Dad
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
 *
 * Nuva OS - Kernel - NDP Duplicate Address Detection
 *
 * DAD engine per RFC 4861 Section 5.4.
 */

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::kernel::net::ipv6::Ipv6Addr;
use crate::kernel::error::{KernelError, KernelResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DadState { Idle, Tentative, Verified, Duplicate }

#[derive(Debug, Clone)]
pub struct DadEntry {
    pub addr: Ipv6Addr,
    pub ifindex: u32,
    pub state: DadState,
    pub ns_sent: u32,
    pub dad_transmits: u32,
    pub last_ns_time: u64,
}

impl DadEntry {
    pub fn new(addr: Ipv6Addr, ifindex: u32, dad_transmits: u32) -> Self {
        DadEntry { addr, ifindex, state: DadState::Idle, ns_sent: 0, dad_transmits, last_ns_time: 0 }
    }
    pub fn needs_more_ns(&self) -> bool {
        self.state == DadState::Tentative && self.ns_sent < self.dad_transmits
    }
    pub fn is_complete(&self) -> bool {
        matches!(self.state, DadState::Verified | DadState::Duplicate)
    }
}

pub struct DadEngine {
    entries: BTreeMap<(Ipv6Addr, u32), DadEntry>,
    default_dad_transmits: u32,
    dad_timeout_ms: u64,
}

impl DadEngine {
    pub fn new(default_dad_transmits: u32, dad_timeout_ms: u64) -> Self {
        DadEngine { entries: BTreeMap::new(), default_dad_transmits, dad_timeout_ms }
    }

    pub fn start_dad(&mut self, addr: Ipv6Addr, ifindex: u32, now: u64) -> KernelResult<DadState> {
        let key = (addr, ifindex);
        if let Some(entry) = self.entries.get(&key) { return Ok(entry.state); }
        let mut entry = DadEntry::new(addr, ifindex, self.default_dad_transmits);
        entry.state = DadState::Tentative;
        entry.ns_sent = 1;
        entry.last_ns_time = now;
        self.entries.insert(key, entry);
        Ok(DadState::Tentative)
    }

    pub fn record_ns_sent(&mut self, addr: &Ipv6Addr, ifindex: u32, now: u64) -> KernelResult<()> {
        let key = (*addr, ifindex);
        if let Some(entry) = self.entries.get_mut(&key) { entry.ns_sent += 1; entry.last_ns_time = now; Ok(()) }
        else { Err(KernelError::NotFound) }
    }

    pub fn should_retransmit(&self, addr: &Ipv6Addr, ifindex: u32, now: u64) -> bool {
        let key = (*addr, ifindex);
        if let Some(entry) = self.entries.get(&key) {
            if !entry.needs_more_ns() { return false; }
            now.saturating_sub(entry.last_ns_time) >= self.dad_timeout_ms
        } else { false }
    }

    pub fn mark_verified(&mut self, addr: &Ipv6Addr, ifindex: u32) -> KernelResult<()> {
        let key = (*addr, ifindex);
        if let Some(entry) = self.entries.get_mut(&key) { entry.state = DadState::Verified; Ok(()) }
        else { Err(KernelError::NotFound) }
    }

    pub fn handle_conflict(&mut self, addr: &Ipv6Addr, ifindex: u32) -> KernelResult<bool> {
        let key = (*addr, ifindex);
        if let Some(entry) = self.entries.get_mut(&key) {
            if entry.state == DadState::Duplicate { return Ok(false); }
            entry.state = DadState::Duplicate;
            Ok(true)
        } else {
            let mut entry = DadEntry::new(*addr, ifindex, self.default_dad_transmits);
            entry.state = DadState::Duplicate;
            self.entries.insert(key, entry);
            Ok(true)
        }
    }

    pub fn get_state(&self, addr: &Ipv6Addr, ifindex: u32) -> Option<DadState> {
        self.entries.get(&(*addr, ifindex)).map(|e| e.state)
    }
    pub fn is_verified(&self, addr: &Ipv6Addr, ifindex: u32) -> bool {
        self.entries.get(&(*addr, ifindex)).map_or(false, |e| e.state == DadState::Verified)
    }
    pub fn is_duplicate(&self, addr: &Ipv6Addr, ifindex: u32) -> bool {
        self.entries.get(&(*addr, ifindex)).map_or(false, |e| e.state == DadState::Duplicate)
    }
    pub fn get_tentative_addresses(&self, now: u64) -> Vec<(Ipv6Addr, u32)> {
        self.entries.iter()
            .filter(|(_, e)| e.needs_more_ns())
            .filter(|(_, e)| now.saturating_sub(e.last_ns_time) >= e.dad_timeout_ms)
            .map(|(k, _)| (k.0, k.1)).collect()
    }
    pub fn remove(&mut self, addr: &Ipv6Addr, ifindex: u32) -> Option<DadEntry> {
        self.entries.remove(&(*addr, ifindex))
    }
    pub fn duplicate_count(&self) -> usize {
        self.entries.values().filter(|e| e.state == DadState::Duplicate).count()
    }
}

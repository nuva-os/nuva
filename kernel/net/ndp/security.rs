/*
 * Nuva OS - Kernel - Net - Ndp - Security
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
 * Nuva OS - Kernel - NDP Security
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NDP message validation: hop limit = 255 check, source address
 * link-local verification, RA Guard, and SEND (Secure NDP) framework.
 */

use alloc::vec::Vec;

use crate::kernel::net::ipv6::Ipv6Addr;
use crate::kernel::error::{KernelError, KernelResult};

pub struct NdpSecurity {
    authorized_routers: Vec<Ipv6Addr>,
    ra_guard_enabled: bool,
    send_enabled: bool,
}

impl NdpSecurity {
    pub fn new() -> Self {
        NdpSecurity { authorized_routers: Vec::new(), ra_guard_enabled: false, send_enabled: false }
    }

    pub fn validate_ndp_message(&self, hop_limit: u8, src_addr: &Ipv6Addr, is_redirect: bool) -> KernelResult<()> {
        if hop_limit != 255 { return Err(KernelError::AccessDenied); }
        if !is_redirect && !src_addr.is_link_local() { return Err(KernelError::AccessDenied); }
        Ok(())
    }

    pub fn validate_ra_source(&self, router_addr: &Ipv6Addr) -> KernelResult<()> {
        if !self.ra_guard_enabled { return Ok(()); }
        if self.authorized_routers.iter().any(|a| a == router_addr) { Ok(()) }
        else { Err(KernelError::AccessDenied) }
    }

    pub fn validate_redirect(&self, src_addr: &Ipv6Addr, target_addr: &Ipv6Addr, hop_limit: u8) -> KernelResult<()> {
        if hop_limit != 255 { return Err(KernelError::AccessDenied); }
        if !src_addr.is_link_local() { return Err(KernelError::AccessDenied); }
        if !target_addr.is_link_local() { return Err(KernelError::AccessDenied); }
        Ok(())
    }

    pub fn enable_ra_guard(&mut self, authorized: Vec<Ipv6Addr>) {
        self.authorized_routers = authorized; self.ra_guard_enabled = true;
    }
    pub fn disable_ra_guard(&mut self) { self.ra_guard_enabled = false; self.authorized_routers.clear(); }
    pub fn is_ra_guard_enabled(&self) -> bool { self.ra_guard_enabled }
    pub fn is_send_enabled(&self) -> bool { self.send_enabled }
    pub fn enable_send(&mut self) { self.send_enabled = true; }
    pub fn disable_send(&mut self) { self.send_enabled = false; }
}

pub trait SendVerifier {
    fn verify_cga(&self, addr: &Ipv6Addr, cga_params: &[u8], public_key: &[u8]) -> KernelResult<()>;
    fn verify_signature(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> KernelResult<()>;
    fn verify_send_message(&self, message: &[u8], cga_params: &[u8], signature: &[u8], public_key: &[u8], addr: &Ipv6Addr) -> KernelResult<()> {
        self.verify_cga(addr, cga_params, public_key)?;
        self.verify_signature(message, signature, public_key)?;
        Ok(())
    }
}

pub struct NoopSendVerifier;

impl SendVerifier for NoopSendVerifier {
    fn verify_cga(&self, _addr: &Ipv6Addr, _cga_params: &[u8], _public_key: &[u8]) -> KernelResult<()> { Ok(()) }
    fn verify_signature(&self, _message: &[u8], _signature: &[u8], _public_key: &[u8]) -> KernelResult<()> { Ok(()) }
}

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

// ! memoryOperation

use crate::error::SdkError;

/// memoryzonedomain
#[derive(Debug, Clone)]
pub struct MemoryRegion {
 /// startbeginaddress
 pub address: u64,
 /// size
 pub size: usize,
 /// permission
 pub permissions: MemoryPermissions,
 /// name
 pub name: Option<String>,
}

/// memorypermission
#[derive(Debug, Clone, Copy)]
pub struct MemoryPermissions {
 pub read: bool,
 pub write: bool,
 pub execute: bool,
}

impl MemoryPermissions {
 pub fn readonly() -> Self {
 Self { read: true, write: false, execute: false }
 }

 pub fn readwrite() -> Self {
 Self { read: true, write: true, execute: false }
 }

 pub fn executable() -> Self {
 Self { read: true, write: false, execute: true }
 }
}

/// memoryinspectiondevice
pub struct MemoryViewer {
 /// data
 data: Vec<u8>,
 /// baseaddress
 base_address: u64,
}

impl MemoryViewer {
 pub fn new(data: Vec<u8>, base_address: u64) -> Self {
 Self { data, base_address }
 }

 /// getcharacterSection
 pub fn get_byte(&self, offset: usize) -> Option<u8> {
 self.data.get(offset).copied()
 }

 /// get 16 positionInteger
 pub fn get_u16(&self, offset: usize) -> Option<u16> {
 if offset + 2 <= self.data.len() {
 Some(u16::from_le_bytes([self.data[offset], self.data[offset + 1]]))
 } else {
 None
 }
 }

 /// get 32 positionInteger
 pub fn get_u32(&self, offset: usize) -> Option<u32> {
 if offset + 4 <= self.data.len() {
 let bytes: [u8; 4] = self.data[offset..offset + 4].try_into().ok()?;
 Some(u32::from_le_bytes(bytes))
 } else {
 None
 }
 }

 /// get 64 positionInteger
 pub fn get_u64(&self, offset: usize) -> Option<u64> {
 if offset + 8 <= self.data.len() {
 let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().ok()?;
 Some(u64::from_le_bytes(bytes))
 } else {
 None
 }
 }

 /// convertasentercontrolString
 pub fn to_hex(&self) -> String {
 self.data.iter()
 .map(|b| format!("{:02x}", b))
 .collect::<Vec<_>>()
 .join(" ")
 }

 /// convertas ASCII String
 pub fn to_ascii(&self) -> String {
 self.data.iter()
 .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
 .collect()
 }

 /// formatasentercontrolbranch
 pub fn hex_dump(&self, bytes_per_line: usize) -> String {
 let mut result = String::new();
 
 for (i, chunk) in self.data.chunks(bytes_per_line).enumerate() {
 let addr = self.base_address + (i * bytes_per_line) as u64;
 result.push_str(&format!("{:016x}: ", addr));
 
 for (j, byte) in chunk.iter().enumerate() {
 if j > 0 {
 result.push(' ');
 }
 result.push_str(&format!("{:02x}", byte));
 }
 
 // paddingnotmeet partsplit
 if chunk.len() < bytes_per_line {
 for _ in chunk.len()..bytes_per_line {
 result.push_str(" ");
 }
 }
 
 result.push_str(" |");
 for &byte in chunk {
 if byte.is_ascii_graphic() || byte == b' ' {
 result.push(byte as char);
 } else {
 result.push('.');
 }
 }
 result.push_str("|
");
 }
 
 result
 }
}
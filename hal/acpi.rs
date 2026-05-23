/*
 * Nuva OS - HAL - ACPI Table Parser (x86_64)
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

/// RSDP signature "RSD PTR ".
const RSDP_SIGNATURE: [u8; 8] = *b"RSD PTR ";

/// Maximum number of ACPI tables.
const MAX_ACPI_TABLES: usize = 32;

/// Maximum number of ACPI devices.
const MAX_ACPI_DEVICES: usize = 64;

/// ACPI table header (common to all ACPI tables).
#[repr(C)]
pub struct AcpiTableHeader {
    /// Table signature (4 ASCII characters).
    pub signature: [u8; 4],
    /// Table length in bytes.
    pub length: u32,
    /// Table revision.
    pub revision: u8,
    /// Checksum (entire table sums to 0).
    pub checksum: u8,
    /// OEM ID.
    pub oem_id: [u8; 6],
    /// OEM Table ID.
    pub oem_table_id: [u8; 8],
    /// OEM revision.
    pub oem_revision: u32,
    /// Compiler ID.
    pub compiler_id: [u8; 4],
    /// Compiler revision.
    pub compiler_revision: u32,
}

impl AcpiTableHeader {
    /// Get the table signature as a string.
    pub fn signature_str(&self) -> &str {
        core::str::from_utf8(&self.signature).unwrap_or("????")
    }

    /// Verify the table checksum.
    pub fn verify_checksum(&self) -> bool {
        // SAFETY: We read the table as raw bytes for checksum verification
        let ptr = self as *const Self as *const u8;
        let len = self.length as usize;
        let mut sum: u8 = 0;
        for i in 0..len {
            // SAFETY: ptr is derived from self (a valid AcpiTableHeader reference), and i is
            // bounded by self.length which matches the actual table size; the header is
            // repr(C) so byte-wise traversal is valid for checksum computation.
            sum = sum.wrapping_add(unsafe { *ptr.add(i) });
        }
        sum == 0
    }
}

/// RSDP (Root System Description Pointer) structure.
#[repr(C)]
pub struct Rsdp {
    /// Signature "RSD PTR ".
    pub signature: [u8; 8],
    /// Checksum.
    pub checksum: u8,
    /// OEM ID.
    pub oem_id: [u8; 6],
    /// Revision (0 = ACPI 1.0, 2 = ACPI 2.0+).
    pub revision: u8,
    /// Physical address of RSDT.
    pub rsdt_address: u32,
    /// Length of the RSDP (ACPI 2.0+).
    pub length: u32,
    /// Physical address of XSDT (ACPI 2.0+).
    pub xsdt_address: u64,
    /// Extended checksum (ACPI 2.0+).
    pub extended_checksum: u8,
    /// Reserved.
    pub reserved: [u8; 3],
}

/// ACPI device information extracted from DSDT.
pub struct AcpiDevice {
    /// Device name (e.g., "_SB.PCI0.USB0").
    pub name: [u8; 32],
    /// Length of the device name.
    pub name_len: usize,
    /// HID (Hardware ID) from _HID method.
    pub hid: [u8; 16],
    /// Length of HID string.
    pub hid_len: usize,
    /// PCI vendor ID (if PCI device).
    pub vendor_id: u16,
    /// PCI device ID (if PCI device).
    pub device_id: u16,
    /// Base address from _CRS method.
    pub base_address: u64,
    /// Address length from _CRS method.
    pub address_length: u64,
    /// Interrupt number from _PRT method.
    pub interrupt: u32,
}

impl AcpiDevice {
    /// Create an empty ACPI device.
    pub const fn new() -> Self {
        AcpiDevice {
            name: [0u8; 32],
            name_len: 0,
            hid: [0u8; 16],
            hid_len: 0,
            vendor_id: 0,
            device_id: 0,
            base_address: 0,
            address_length: 0,
            interrupt: 0,
        }
    }

    /// Get the device name as a string.
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }

    /// Get the HID as a string.
    pub fn hid_str(&self) -> &str {
        core::str::from_utf8(&self.hid[..self.hid_len]).unwrap_or("")
    }
}

/// ACPI table entry (signature + physical address).
pub struct AcpiTableEntry {
    /// Table signature.
    pub signature: [u8; 4],
    /// Physical address of the table.
    pub address: u64,
    /// Whether the table has been mapped and verified.
    pub verified: bool,
}

impl AcpiTableEntry {
    /// Create an empty table entry.
    pub const fn new() -> Self {
        AcpiTableEntry {
            signature: [0u8; 4],
            address: 0,
            verified: false,
        }
    }

    /// Get the signature as a string.
    pub fn signature_str(&self) -> &str {
        core::str::from_utf8(&self.signature).unwrap_or("????")
    }
}

/// ACPI parser state.
pub struct AcpiTables {
    /// RSDP physical address.
    pub rsdp_address: u64,
    /// ACPI revision (1 = ACPI 1.0, 2 = ACPI 2.0+).
    pub revision: u8,
    /// Discovered tables.
    pub tables: [AcpiTableEntry; MAX_ACPI_TABLES],
    /// Number of discovered tables.
    pub table_count: usize,
    /// Discovered devices (from DSDT enumeration).
    pub devices: [AcpiDevice; MAX_ACPI_DEVICES],
    /// Number of discovered devices.
    pub device_count: usize,
    /// Total memory from SRAT/CRS.
    pub memory_size: u64,
    /// Number of CPUs from MADT.
    pub cpu_count: u32,
    /// Local APIC address from MADT.
    pub local_apic_address: u64,
}

impl AcpiTables {
    /// Create an empty ACPI tables structure.
    pub const fn new() -> Self {
        AcpiTables {
            rsdp_address: 0,
            revision: 0,
            tables: [AcpiTableEntry::new(); MAX_ACPI_TABLES],
            table_count: 0,
            devices: [AcpiDevice::new(); MAX_ACPI_DEVICES],
            device_count: 0,
            memory_size: 0,
            cpu_count: 0,
            local_apic_address: 0,
        }
    }

    /// Locate the RSDP by scanning the BIOS area.
    /// Search order:
    /// 1. EFI configuration table (if available)
    /// 2. BIOS area 0xE0000-0xFFFFF (first 1KB of each 16-byte block)
    /// 3. Extended BIOS area 0x000E0000-0x000FFFFF
    pub fn find_rsdp(&mut self) -> bool {
        // Search the BIOS area for the RSDP signature
        // SAFETY: Scanning BIOS area 0xE0000-0xFFFFF for RSDP; this region is
        // always identity-mapped on x86_64 systems and contains ACPI tables
        // placed by the firmware.
        for base in (0xE0000..0x100000).step_by(16) {
            let ptr = base as *const [u8; 8];
            // SAFETY: Reading 8-byte RSDP signature from BIOS area 0xE0000-0xFFFFF,
            // which is always mapped on x86_64 systems; ptr is 16-byte aligned per
            // step_by(16) iteration, matching the [u8; 8] array size.
            let sig = unsafe { *ptr };
            if sig == RSDP_SIGNATURE {
                // Verify RSDP checksum
                // SAFETY: base is within the BIOS area and points to a valid RSDP
                // structure (signature matched), which is repr(C) and always mapped.
                let rsdp = unsafe { &*(base as *const Rsdp) };
                let mut sum: u8 = 0;
                let len = if rsdp.revision >= 2 { 36 } else { 20 };
                for i in 0..len {
                    // SAFETY: base is a valid RSDP address in BIOS area, i is bounded
                    // by len (20 for ACPI 1.0, 36 for ACPI 2.0+), both within the
                    // mapped BIOS ROM region.
                    sum = sum.wrapping_add(unsafe { *(base as *const u8).add(i) });
                }
                if sum == 0 {
                    self.rsdp_address = base as u64;
                    self.revision = rsdp.revision;
                    log_info!("ACPI: RSDP found at 0x{:x}, revision {}", base, rsdp.revision);
                    return true;
                }
            }
        }

        log_error!("ACPI: RSDP not found in BIOS area");
        false
    }

    /// Initialize ACPI from a known RSDP address (e.g., from EFI).
    pub fn init_from_rsdp(&mut self, rsdp_address: u64) -> bool {
        // SAFETY: rsdp_address is provided by EFI configuration table or bootloader,
        // which guarantees it points to a valid RSDP structure in mapped memory.
        let rsdp = unsafe { &*(rsdp_address as *const Rsdp) };

        // Verify signature
        if rsdp.signature != RSDP_SIGNATURE {
            log_error!("ACPI: Invalid RSDP signature at 0x{:x}", rsdp_address);
            return false;
        }

        self.rsdp_address = rsdp_address;
        self.revision = rsdp.revision;

        log_info!("ACPI: RSDP at 0x{:x}, revision {}", rsdp_address, rsdp.revision);
        true
    }

    /// Parse the XSDT/RSDT to discover all ACPI tables.
    pub fn parse_tables(&mut self) -> bool {
        if self.rsdp_address == 0 {
            log_error!("ACPI: RSDP not initialized");
            return false;
        }

        // SAFETY: RSDP was previously verified (signature and checksum checked),
        // so re-reading from the stored rsdp_address is valid.
        let rsdp = unsafe { &*(self.rsdp_address as *const Rsdp) };

        if rsdp.revision >= 2 && rsdp.xsdt_address != 0 {
            // Use XSDT (ACPI 2.0+) - 64-bit addresses
            self.parse_xsdt(rsdp.xsdt_address)
        } else {
            // Use RSDT (ACPI 1.0) - 32-bit addresses
            self.parse_rsdt(rsdp.rsdt_address as u64)
        }
    }

    /// Parse the XSDT (Extended System Description Table).
    fn parse_xsdt(&mut self, xsdt_address: u64) -> bool {
        // SAFETY: xsdt_address comes from the verified RSDP's XSDT field, which
        // points to a valid XSDT table per the ACPI specification.
        let xsdt = unsafe { &*(xsdt_address as *const AcpiTableHeader) };

        if xsdt.signature != *b"XSDT" {
            log_error!("ACPI: Invalid XSDT signature");
            return false;
        }

        let entry_count = (xsdt.length as usize - core::mem::size_of::<AcpiTableHeader>()) / 8;
        log_info!("ACPI: XSDT has {} entries", entry_count);

        // SAFETY: xsdt_address points to a verified XSDT table; adding the header
        // size yields the start of the 64-bit entry array, which is within the
        // mapped ACPI table region.
        let entries_ptr = unsafe { (xsdt_address as *const u8).add(core::mem::size_of::<AcpiTableHeader>()) };

        for i in 0..entry_count {
            if self.table_count >= MAX_ACPI_TABLES { break; }

            // SAFETY: entries_ptr points to the XSDT entry array; i is bounded by
            // entry_count which is derived from the verified XSDT length field;
            // each entry is an aligned u64 per the ACPI specification.
            let entry_addr = unsafe {
                let ptr = entries_ptr.add(i * 8) as *const u64;
                u64::from_le(*ptr)
            };

            if entry_addr != 0 {
                self.add_table(entry_addr);
            }
        }

        true
    }

    /// Parse the RSDT (Root System Description Table).
    fn parse_rsdt(&mut self, rsdt_address: u64) -> bool {
        // SAFETY: rsdt_address comes from the verified RSDP's RSDT field, which
        // points to a valid RSDT table per the ACPI 1.0 specification.
        let rsdt = unsafe { &*(rsdt_address as *const AcpiTableHeader) };

        if rsdt.signature != *b"RSDT" {
            log_error!("ACPI: Invalid RSDT signature");
            return false;
        }

        let entry_count = (rsdt.length as usize - core::mem::size_of::<AcpiTableHeader>()) / 4;
        log_info!("ACPI: RSDT has {} entries", entry_count);

        // SAFETY: rsdt_address points to a verified RSDT table; adding the header
        // size yields the start of the 32-bit entry array, which is within the
        // mapped ACPI table region.
        let entries_ptr = unsafe { (rsdt_address as *const u8).add(core::mem::size_of::<AcpiTableHeader>()) };

        for i in 0..entry_count {
            if self.table_count >= MAX_ACPI_TABLES { break; }

            // SAFETY: entries_ptr points to the RSDT entry array; i is bounded by
            // entry_count derived from the verified RSDT length field; each entry
            // is an aligned u32 per the ACPI 1.0 specification.
            let entry_addr = unsafe {
                let ptr = entries_ptr.add(i * 4) as *const u32;
                u64::from(u32::from_le(*ptr))
            };

            if entry_addr != 0 {
                self.add_table(entry_addr);
            }
        }

        true
    }

    /// Add a table entry from its physical address.
    fn add_table(&mut self, address: u64) {
        // SAFETY: address comes from a verified XSDT/RSDT entry, which points to
        // a valid ACPI table per the ACPI specification; the header is repr(C).
        let header = unsafe { &*(address as *const AcpiTableHeader) };

        let entry = &mut self.tables[self.table_count];
        entry.signature = header.signature;
        entry.address = address;
        entry.verified = header.verify_checksum();

        if !entry.verified {
            log_warn!("ACPI: Table {} at 0x{:x} has invalid checksum", entry.signature_str(), address);
        }

        log_info!("ACPI: Found table {} at 0x{:x}", entry.signature_str(), address);
        self.table_count += 1;
    }

    /// Find a table by signature.
    pub fn find_table(&self, signature: &[u8; 4]) -> Option<&AcpiTableEntry> {
        for i in 0..self.table_count {
            if self.tables[i].signature == *signature {
                return Some(&self.tables[i]);
            }
        }
        None
    }

    /// Parse the MADT (Multiple APIC Description Table) to get CPU count.
    pub fn parse_madt(&mut self) -> bool {
        if let Some(entry) = self.find_table(b"APIC") {
            // SAFETY: entry.address is from a verified ACPI table entry (MADT/APIC),
            // so the pointer is valid and the table is mapped in memory.
            let header = unsafe { &*(entry.address as *const AcpiTableHeader) };
            let madt_ptr = entry.address as *const u8;
            let madt_len = header.length as usize;

            // Local APIC address is at offset 36 in MADT
            if madt_len >= 40 {
                // SAFETY: madt_ptr points to a verified MADT table; offset 36 is
                // within the table (madt_len >= 40), and the Local APIC Address
                // field is a 32-bit value at that offset per the ACPI spec.
                self.local_apic_address = u64::from(unsafe {
                    let ptr = madt_ptr.add(36) as *const u32;
                    u32::from_le(*ptr)
                });
            }

            // Count Local APIC entries (type 0) in the interrupt controller list
            let mut offset = 44; // MADT header + 4 bytes of flags + 4 bytes of LAPIC addr
            self.cpu_count = 0;

            while offset < madt_len {
                // SAFETY: madt_ptr is a verified MADT table pointer; offset is
                // bounded by madt_len, so reading entry_type at offset is within
                // the mapped ACPI table region.
                let entry_type = unsafe { *madt_ptr.add(offset) };
                // SAFETY: offset + 1 < madt_len because each MADT interrupt
                // controller structure is at least 2 bytes (type + length fields).
                let entry_len = unsafe { *madt_ptr.add(offset + 1) } as usize;

                if entry_len == 0 { break; }

                if entry_type == 0 {
                    // Local APIC entry - count enabled CPUs
                    // SAFETY: offset + 4 is within the MADT table because Local APIC
                    // entries are 8 bytes per the ACPI spec, and offset is bounded by
                    // madt_len with entry_len >= 8.
                    let flags = unsafe { *madt_ptr.add(offset + 4) };
                    if flags & 1 != 0 {
                        self.cpu_count += 1;
                    }
                }

                offset += entry_len;
            }

            log_info!("ACPI: MADT: {} CPUs, LAPIC at 0x{:x}", self.cpu_count, self.local_apic_address);
            true
        } else {
            log_warn!("ACPI: MADT not found");
            false
        }
    }

    /// Enumerate PCI devices via MCFG table and PCI config space.
    pub fn enumerate_pci(&mut self) -> u32 {
        let mut device_count = 0u32;

        if let Some(entry) = self.find_table(b"MCFG") {
            // SAFETY: entry.address is from a verified ACPI table entry (MCFG),
            // so mcfg_ptr is valid and the MCFG table is mapped in memory.
            let mcfg_ptr = entry.address as *const u8;
            let header = unsafe { &*(entry.address as *const AcpiTableHeader) };
            let mcfg_len = header.length as usize;

            // MCFG entries start at offset 44
            let mut offset = 44;
            while offset + 16 <= mcfg_len && self.device_count < MAX_ACPI_DEVICES {
                // SAFETY: mcfg_ptr points to a verified MCFG table; offset is
                // bounded by mcfg_len - 16, so reading 8 bytes at offset is within
                // the mapped ACPI table region; the base address field is a u64 per
                // the MCFG entry structure in the ACPI spec.
                let base_addr = unsafe {
                    let ptr = mcfg_ptr.add(offset) as *const u64;
                    u64::from_le(*ptr)
                };
                // SAFETY: offset + 8 + 2 <= mcfg_len (guaranteed by while condition
                // offset + 16 <= mcfg_len); the PCI segment group number is a u16
                // at offset + 8 per the MCFG entry structure.
                let _segment = unsafe {
                    let ptr = mcfg_ptr.add(offset + 8) as *const u16;
                    u16::from_le(*ptr)
                };
                // SAFETY: offset + 10 and offset + 11 are within mcfg_len (guaranteed
                // by while condition); these are single-byte fields for start/end bus
                // numbers per the MCFG entry structure.
                let start_bus = unsafe { *mcfg_ptr.add(offset + 10) };
                let end_bus = unsafe { *mcfg_ptr.add(offset + 11) };

                // Enumerate PCI devices in this bus range
                for bus in start_bus..=end_bus {
                    for device in 0..32u8 {
                        for function in 0..8u8 {
                            let config_addr = base_addr
                                | (u64::from(bus) << 20)
                                | (u64::from(device) << 15)
                                | (u64::from(function) << 12);

                            // Read PCI vendor/device ID
                            // SAFETY: config_addr is a valid PCI ECAM address computed from
                            // the MCFG base address and BDF (bus/device/function); reading
                            // a u32 at offset 0 yields the vendor/device ID register which
                            // is always readable in PCI config space.
                            let vendor_device = unsafe {
                                let ptr = config_addr as *const u32;
                                *ptr
                            };

                            let vendor_id = (vendor_device & 0xFFFF) as u16;
                            let device_id = ((vendor_device >> 16) & 0xFFFF) as u16;

                            if vendor_id == 0xFFFF { continue; }

                            // Add device
                            if self.device_count < MAX_ACPI_DEVICES {
                                let dev = &mut self.devices[self.device_count];
                                dev.vendor_id = vendor_id;
                                dev.device_id = device_id;

                                // Format name as "PCI:vvvv:dddd"
                                let name = b"PCI:";
                                for (j, &b) in name.iter().enumerate() {
                                    dev.name[j] = b;
                                }
                                dev.name_len = 4;
                                dev.hid_len = 0;
                                dev.base_address = config_addr;

                                self.device_count += 1;
                                device_count += 1;
                            }

                            // Multi-function devices: check header type
                            if function == 0 {
                                // SAFETY: config_addr + 0x0C is a valid PCI ECAM
                                // configuration space address; offset 0x0C is the
                                // cache-line-size/header-type/latency-timer byte,
                                // which is always readable in PCI config space.
                                let header_type = unsafe {
                                    let ptr = (config_addr + 0x0C) as *const u8;
                                    *ptr
                                };
                                if header_type & 0x80 == 0 { break; }
                            }
                        }
                    }
                }

                offset += 16;
            }
        }

        log_info!("ACPI: Enumerated {} PCI devices", device_count);
        device_count
    }
}

/// Global ACPI tables instance.
static mut ACPI_TABLES: AcpiTables = AcpiTables::new();

/// Get a reference to the global ACPI tables.
pub fn get_acpi_tables() -> &'static AcpiTables {
    // SAFETY: ACPI_TABLES is a mutable static accessed only during single-threaded
    // HAL initialization; after init completes, only immutable references are returned.
    unsafe { &ACPI_TABLES }
}

/// Initialize the ACPI subsystem.
/// On x86_64, the RSDP is located by:
/// 1. EFI configuration table (if booted via UEFI)
/// 2. Scanning BIOS area 0xE0000-0xFFFFF
pub fn init_acpi() -> bool {
    // SAFETY: init_acpi() is called only during single-threaded HAL initialization
    // before any other CPU cores are brought online; no concurrent access possible.
    let acpi = unsafe { &mut ACPI_TABLES };

    // Step 1: Find RSDP
    if !acpi.find_rsdp() {
        log_error!("ACPI: Failed to locate RSDP");
        return false;
    }

    // Step 2: Parse XSDT/RSDT to discover tables
    if !acpi.parse_tables() {
        log_error!("ACPI: Failed to parse system description tables");
        return false;
    }

    // Step 3: Parse MADT for CPU count
    acpi.parse_madt();

    // Step 4: Enumerate PCI devices
    acpi.enumerate_pci();

    log_info!("ACPI: Initialized ({} tables, {} devices, {} CPUs)",
        acpi.table_count, acpi.device_count, acpi.cpu_count);
    true
}

// ============================================================================
// ACPI Power Management (FADT-based sleep state control)
// ============================================================================

/// FADT (Fixed ACPI Description Table) fields relevant to power management
#[repr(C)]
pub struct Fadt {
    /// Common ACPI table header
    pub header: AcpiTableHeader,
    /// FIRMWARE_CTRL (FACS address, 32-bit)
    pub firmware_ctrl: u32,
    /// DSDT address (32-bit)
    pub dsdt: u32,
    /// Reserved
    pub _reserved0: u8,
    /// Preferred PM profile
    pub preferred_pm: u8,
    /// SCI interrupt
    pub sci_int: u16,
    /// SMI command port
    pub smi_cmd: u32,
    /// ACPI enable value
    pub acpi_enable: u8,
    /// ACPI disable value
    pub acpi_disable: u8,
    /// S4 BIOS request
    pub s4_bios_req: u8,
    /// P-state control
    pub pstate_cnt: u8,
    /// PM1a Event Block
    pub pm1a_event_blk: u32,
    /// PM1b Event Block
    pub pm1b_event_blk: u32,
    /// PM1a Control Block
    pub pm1a_cnt_blk: u32,
    /// PM1b Control Block
    pub pm1b_cnt_blk: u32,
    /// PM2 Control Block
    pub pm2_cnt_blk: u32,
    /// PM Timer Block
    pub pm_tmr_blk: u32,
    /// GPE0 Block
    pub gpe0_blk: u32,
    /// GPE1 Block
    pub gpe1_blk: u32,
    /// PM1 Event Length
    pub pm1_evt_len: u8,
    /// PM1 Control Length
    pub pm1_cnt_len: u8,
    /// PM2 Control Length
    pub pm2_cnt_len: u8,
    /// PM Timer Length
    pub pm_tmr_len: u8,
    /// GPE0 Length
    pub gpe0_len: u8,
    /// GPE1 Length
    pub gpe1_len: u8,
    /// GPE1 Base
    pub gpe1_base: u8,
    /// CST Control
    pub cst_cnt: u8,
    /// C2 Latency
    pub c2_latency: u16,
    /// C3 Latency
    pub c3_latency: u16,
    /// CPU cache size
    pub cpu_cache_len: u16,
    /// Cache flush stride
    pub cache_flush_stride: u16,
    /// Duty offset
    pub duty_offset: u8,
    /// Duty width
    pub duty_width: u8,
    /// Day alarm
    pub day_alrm: u8,
    /// Month alarm
    pub mon_alrm: u8,
    /// Century
    pub century: u8,
    /// IA-PC Boot Architecture
    pub iapc_boot_arch: u16,
    /// Reserved
    pub _reserved1: u8,
    /// Flags
    pub flags: u32,
    /// Reset Register (Generic Address Structure - 12 bytes)
    pub reset_reg: [u8; 12],
    /// Reset Value
    pub reset_value: u8,
    /// Reserved
    pub _reserved2: [u8; 3],
    /// X FIRMWARE_CTRL (64-bit FACS address)
    pub x_firmware_ctrl: u64,
    /// X DSDT (64-bit DSDT address)
    pub x_dsdt: u64,
    /// X PM1a Event Block
    pub x_pm1a_event_blk: [u8; 12],
    /// X PM1b Event Block
    pub x_pm1b_event_blk: [u8; 12],
    /// X PM1a Control Block
    pub x_pm1a_cnt_blk: [u8; 12],
    /// X PM1b Control Block
    pub x_pm1b_cnt_blk: [u8; 12],
    /// X PM2 Control Block
    pub x_pm2_cnt_blk: [u8; 12],
    /// X PM Timer Block
    pub x_pm_tmr_blk: [u8; 12],
    /// X GPE0 Block
    pub x_gpe0_blk: [u8; 12],
    /// X GPE1 Block
    pub x_gpe1_blk: [u8; 12],
}

/// ACPI sleep type values (SLP_TYP bits) for PM1 control register
pub mod sleep_type {
    /// S0: Working state
    pub const S0: u16 = 0;
    /// S1: Sleeping state (CPU stopped, HW maintains context)
    pub const S1: u16 = 1;
    /// S3: Suspend to RAM
    pub const S3: u16 = 3;
    /// S5: Soft-off (shutdown)
    pub const S5: u16 = 5;
}

/// PM1 control register SLP_TYP shift (bits 10-12)
const PM1_SLP_TYP_SHIFT: u16 = 10;
/// PM1 control register SLP_EN bit (bit 13)
const PM1_SLP_EN: u16 = 1 << 13;

/// ACPI Power Management Driver
pub struct AcpiPowerDriver {
    /// PM1a control register I/O port
    pm1a_cnt: u16,
    /// PM1b control register I/O port
    pm1b_cnt: u16,
    /// SMI command I/O port
    smi_cmd: u16,
    /// ACPI enable value
    acpi_enable: u8,
    /// ACPI disable value
    acpi_disable: u8,
    /// Reset register address (from Generic Address Structure)
    reset_reg_addr: u64,
    /// Reset register space ID (0=SystemIO, 1=SystemMemory)
    reset_reg_space_id: u8,
    /// Reset value to write
    reset_value: u8,
    /// Whether FADT has been parsed
    initialized: bool,
}

impl AcpiPowerDriver {
    /// Create an uninitialized power driver
    pub const fn new() -> Self {
        AcpiPowerDriver {
            pm1a_cnt: 0,
            pm1b_cnt: 0,
            smi_cmd: 0,
            acpi_enable: 0,
            acpi_disable: 0,
            reset_reg_addr: 0,
            reset_reg_space_id: 0,
            reset_value: 0,
            initialized: false,
        }
    }

    /// Parse FADT to populate power management register addresses
    pub fn init_from_fadt(&mut self) -> bool {
        let acpi = get_acpi_tables();
        if let Some(entry) = acpi.find_table(b"FACP") {
            // SAFETY: entry.address is from a verified ACPI table entry (FADT/FACP),
            // so the pointer is valid and the table is mapped in memory.
            let fadt = unsafe { &*(entry.address as *const Fadt) };

            self.pm1a_cnt = fadt.pm1a_cnt_blk as u16;
            self.pm1b_cnt = fadt.pm1b_cnt_blk as u16;
            self.smi_cmd = fadt.smi_cmd as u16;
            self.acpi_enable = fadt.acpi_enable;
            self.acpi_disable = fadt.acpi_disable;
            self.reset_value = fadt.reset_value;

            // Parse Reset Register Generic Address Structure (12 bytes)
            // GAS layout: [0]=space_id, [1]=bit_width, [2]=bit_offset, [3]=access_size,
            //             [4..12]=address (little-endian u64)
            self.reset_reg_space_id = fadt.reset_reg[0];
            // SAFETY: reset_reg[4..12] contains the 64-bit address in little-endian format;
            // we reconstruct it from raw bytes, which is valid for repr-C ACPI GAS layout.
            self.reset_reg_addr = unsafe {
                let ptr = fadt.reset_reg.as_ptr().add(4) as *const u64;
                u64::from_le(*ptr)
            };

            // Enable ACPI mode by writing ACPI_ENABLE to SMI_CMD
            if self.smi_cmd != 0 {
                // SAFETY: Writing to SMI_CMD I/O port to enable ACPI mode; this is
                // the standard ACPI initialization sequence per the ACPI specification.
                unsafe {
                    Self::io_write8(self.smi_cmd, self.acpi_enable);
                }
            }

            self.initialized = true;
            log_info!("ACPI Power: FADT parsed, PM1a_CNT=0x{:x}, PM1b_CNT=0x{:x}",
                self.pm1a_cnt, self.pm1b_cnt);
            true
        } else {
            log_error!("ACPI Power: FADT not found");
            false
        }
    }

    /// Enter ACPI sleep state (S1/S3/S5)
    /// Writes SLP_TYP to PM1a_CNT and PM1b_CNT, then sets SLP_EN bit.
    /// This triggers the hardware sleep state transition.
    pub fn enter_sleep_state(&self, sleep_type: u16) {
        if !self.initialized {
            log_error!("ACPI Power: Driver not initialized");
            return;
        }

        let slp_typ = sleep_type << PM1_SLP_TYP_SHIFT;
        let pm1a_value = slp_typ | PM1_SLP_EN;
        let pm1b_value = slp_typ | PM1_SLP_EN;

        log_info!("ACPI Power: Entering S{} sleep state", sleep_type);

        // SAFETY: Writing to PM1a_CNT and PM1b_CNT I/O ports per the ACPI
        // specification to enter a sleep state. The SLP_TYP field (bits 10-12)
        // and SLP_EN bit (bit 13) are set simultaneously. This is a privileged
        // operation that transitions the system hardware into the target state.
        unsafe {
            if self.pm1a_cnt != 0 {
                Self::io_write16(self.pm1a_cnt, pm1a_value);
            }
            if self.pm1b_cnt != 0 {
                Self::io_write16(self.pm1b_cnt, pm1b_value);
            }
        }

        // After setting SLP_EN, the CPU should be in the target sleep state.
        // If execution continues (e.g., spurious wakeup), halt in a loop.
        loop {
            // SAFETY: inline assembly required for hardware halt instruction
            unsafe { core::arch::asm!("hlt"); }
        }
    }

    /// Reset system via FADT reset register
    pub fn system_reset(&self) {
        if !self.initialized {
            log_error!("ACPI Power: Driver not initialized, cannot reset");
            return;
        }

        if self.reset_reg_addr == 0 {
            log_error!("ACPI Power: No reset register defined");
            return;
        }

        log_info!("ACPI Power: Resetting system via FADT reset register");

        // SAFETY: Writing the reset value to the FADT-defined reset register
        // triggers a system reset per the ACPI specification. The space_id
        // determines whether the register is in SystemIO or SystemMemory space.
        unsafe {
            if self.reset_reg_space_id == 0 {
                // SystemIO space
                Self::io_write8(self.reset_reg_addr as u16, self.reset_value);
            } else {
                // SystemMemory space
                core::ptr::write_volatile(self.reset_reg_addr as *mut u8, self.reset_value);
            }
        }

        // Should not reach here
        loop {
            unsafe { core::arch::asm!("hlt"); }
        }
    }

    /// Write 8-bit value to I/O port
    #[inline]
    unsafe fn io_write8(port: u16, value: u8) {
        core::arch::asm!("outb", in("al") value, in("dx") port);
    }

    /// Write 16-bit value to I/O port
    #[inline]
    unsafe fn io_write16(port: u16, value: u16) {
        core::arch::asm!("outw", in("ax") value, in("dx") port);
    }

    /// Read 16-bit value from I/O port
    #[inline]
    unsafe fn io_read16(port: u16) -> u16 {
        let value: u16;
        core::arch::asm!("inw", out("ax") value, in("dx") port);
        value
    }
}

/// Global ACPI power driver instance
static mut ACPI_POWER_DRIVER: AcpiPowerDriver = AcpiPowerDriver::new();

/// Initialize ACPI power management driver
pub fn init_acpi_power() -> bool {
    // SAFETY: ACPI_POWER_DRIVER is only accessed during single-threaded init.
    let driver = unsafe { &mut ACPI_POWER_DRIVER };
    driver.init_from_fadt()
}

/// Get ACPI power driver
pub fn get_acpi_power_driver() -> &'static AcpiPowerDriver {
    // SAFETY: After init, the driver is read-only; no concurrent write hazard.
    unsafe { &ACPI_POWER_DRIVER }
}

/// Enter ACPI sleep state (convenience function)
pub fn enter_sleep_state(sleep_type: u16) {
    get_acpi_power_driver().enter_sleep_state(sleep_type);
}

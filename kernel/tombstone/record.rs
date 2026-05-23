/*
 * Nuva OS - Kernel - Tombstone - Record Data Structures
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

//! Tombstone record data structures and serialization.
/*!*/
//! Defines the core data types for tombstone crash records, including
//! crash reasons, architecture IDs, register arrays, stack frames,
//! and the tombstone record itself with binary serialization support.

use core::fmt;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** Maximum number of general-purpose registers stored per crash */
pub const MAX_REGISTERS: usize = 32;

/** Maximum depth of stack backtrace */
pub const MAX_STACK_FRAMES: usize = 32;

/** Maximum length of a symbol name in bytes */
pub const SYMBOL_NAME_MAX_LEN: usize = 64;

/** Maximum length of a process name in bytes */
pub const PROCESS_NAME_MAX_LEN: usize = 32;

/** Tombstone file magic number ("TBSN" in little-endian) */
pub const TOMBSTONE_FILE_MAGIC: u32 = 0x5442_534E;

/** Tombstone binary format version */
pub const TOMBSTONE_FORMAT_VERSION: u32 = 1;

/** Maximum tombstone records in storage */
pub const TOMBSTONE_MAX_COUNT: u32 = 100;

/** Maximum single tombstone file size in bytes */
pub const TOMBSTONE_MAX_FILE_SIZE: u32 = 8192;

/** Default tombstone storage directory */
pub const TOMBSTONE_DEFAULT_DIR: &[u8; 18] = b"/data/tombstones/\0";

/** POSIX signal numbers for fatal signals */
pub const SIGSEGV: u8 = 11;
pub const SIGABRT: u8 = 6;
pub const SIGBUS: u8 = 7;
pub const SIGILL: u8 = 4;
pub const SIGFPE: u8 = 8;
pub const SIGTRAP: u8 = 5;

// ---------------------------------------------------------------------------
// CrashReason
// ---------------------------------------------------------------------------

/** Classification of why a process or task crashed */
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashReason {
    /** Terminated by a fatal signal (SIGABRT, etc.) */
    FatalSignal = 0,
    /** Illegal memory access (SIGSEGV, SIGBUS) */
    IllegalAccess = 1,
    /** Illegal or privileged instruction (SIGILL) */
    IllegalInstruction = 2,
    /** Floating-point exception (SIGFPE) */
    FpException = 3,
    /** Fault originating in kernel mode */
    KernelFault = 4,
    /** Task watchdog timeout */
    Watchdog = 5,
    /** Stack overflow detected */
    StackOverflow = 6,
    /** Unknown or unclassifiable crash */
    Unknown = 255,
}

impl CrashReason {
    /** Map a POSIX signal number to a CrashReason classification */
    pub fn from_signal(sig: u8) -> Self {
        match sig {
            SIGABRT => CrashReason::FatalSignal,
            SIGSEGV | SIGBUS => CrashReason::IllegalAccess,
            SIGILL => CrashReason::IllegalInstruction,
            SIGFPE => CrashReason::FpException,
            SIGTRAP => CrashReason::IllegalInstruction,
            _ => CrashReason::Unknown,
        }
    }
}

impl fmt::Display for CrashReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CrashReason::FatalSignal => write!(f, "FatalSignal"),
            CrashReason::IllegalAccess => write!(f, "IllegalAccess"),
            CrashReason::IllegalInstruction => write!(f, "IllegalInstruction"),
            CrashReason::FpException => write!(f, "FpException"),
            CrashReason::KernelFault => write!(f, "KernelFault"),
            CrashReason::Watchdog => write!(f, "Watchdog"),
            CrashReason::StackOverflow => write!(f, "StackOverflow"),
            CrashReason::Unknown => write!(f, "Unknown"),
        }
    }
}

// ---------------------------------------------------------------------------
// ArchId
// ---------------------------------------------------------------------------

/** Architecture identifier for cross-arch tombstone compatibility */
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchId {
    /** AArch64 / ARM64 */
    Arm64 = 0,
    /** x86-64 */
    X64 = 1,
    /** LoongArch64 */
    LoongArch64 = 2,
}

impl ArchId {
    /** Return the ArchId for the currently compiled target */
    pub fn current() -> Self {
        #[cfg(target_arch = "aarch64")]
        {
            ArchId::Arm64
        }
        #[cfg(target_arch = "x86_64")]
        {
            ArchId::X64
        }
        #[cfg(target_arch = "loongarch64")]
        {
            ArchId::LoongArch64
        }
    }
}

impl fmt::Display for ArchId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArchId::Arm64 => write!(f, "ARM64"),
            ArchId::X64 => write!(f, "X64"),
            ArchId::LoongArch64 => write!(f, "LoongArch64"),
        }
    }
}

// ---------------------------------------------------------------------------
// UnwindTruncateReason
// ---------------------------------------------------------------------------

/** Reason why a stack backtrace was truncated */
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnwindTruncateReason {
    /** Backtrace completed within the frame limit */
    None = 0,
    /** Stack memory appeared corrupted */
    CorruptStack = 1,
    /** Frame pointer was invalid */
    InvalidFp = 2,
    /** Stack frame pointed to unmapped memory */
    UnmappedMemory = 3,
    /** Reached the maximum frame count */
    MaxFrames = 4,
}

// ---------------------------------------------------------------------------
// TombstoneError
// ---------------------------------------------------------------------------

/** Errors that may occur during tombstone generation, storage, or query */
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TombstoneError {
    /** Memory allocation failed */
    OutOfMemory = 1,
    /** I/O error during file operation */
    IoError = 2,
    /** File system is not available */
    FsUnavailable = 3,
    /** Invalid parameter supplied */
    InvalidParam = 4,
    /** Caller lacks required capability */
    PermissionDenied = 5,
    /** Requested record not found */
    NotFound = 6,
    /** CPU context collection failed */
    ContextCollectionFailed = 7,
    /** Stack unwind failed */
    StackUnwindFailed = 8,
    /** Serialization or deserialization error */
    SerializeError = 9,
    /** Storage capacity exceeded */
    CapacityExceeded = 10,
    /** CRC32 checksum mismatch */
    ChecksumMismatch = 11,
    /** Tombstone subsystem not initialized */
    NotInitialized = 12,
}

impl fmt::Display for TombstoneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TombstoneError::OutOfMemory => write!(f, "OutOfMemory"),
            TombstoneError::IoError => write!(f, "IoError"),
            TombstoneError::FsUnavailable => write!(f, "FsUnavailable"),
            TombstoneError::InvalidParam => write!(f, "InvalidParam"),
            TombstoneError::PermissionDenied => write!(f, "PermissionDenied"),
            TombstoneError::NotFound => write!(f, "NotFound"),
            TombstoneError::ContextCollectionFailed => write!(f, "ContextCollectionFailed"),
            TombstoneError::StackUnwindFailed => write!(f, "StackUnwindFailed"),
            TombstoneError::SerializeError => write!(f, "SerializeError"),
            TombstoneError::CapacityExceeded => write!(f, "CapacityExceeded"),
            TombstoneError::ChecksumMismatch => write!(f, "ChecksumMismatch"),
            TombstoneError::NotInitialized => write!(f, "NotInitialized"),
        }
    }
}

impl TombstoneError {
    /** Convert to a negative errno-style integer for syscall return */
    pub fn to_errno(&self) -> i32 {
        -(*self as i32)
    }
}

// ---------------------------------------------------------------------------
// RegisterArray
// ---------------------------------------------------------------------------

/** Fixed-capacity array of general-purpose register values */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RegisterArray {
    /** Register values */
    pub regs: [u64; MAX_REGISTERS],
    /** Number of valid entries (0..=MAX_REGISTERS) */
    pub count: u8,
}

impl RegisterArray {
    /** Create an all-zero register array */
    pub const fn new() -> Self {
        RegisterArray {
            regs: [0u64; MAX_REGISTERS],
            count: 0,
        }
    }

    /** Set registers from a slice, clamping to MAX_REGISTERS */
    pub fn set_regs(&mut self, src: &[u64]) {
        let len = if src.len() > MAX_REGISTERS {
            MAX_REGISTERS
        } else {
            src.len()
        };
        self.regs[..len].copy_from_slice(&src[..len]);
        self.count = len as u8;
    }
}

impl Default for RegisterArray {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// StackFrame / StackFrameArray
// ---------------------------------------------------------------------------

/** A single frame in a stack backtrace */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StackFrame {
    /** Return address of this frame */
    pub return_addr: u64,
    /** Best-effort symbol name (zero-padded if unavailable) */
    pub symbol: [u8; SYMBOL_NAME_MAX_LEN],
    /** Whether the symbol name is valid */
    pub has_symbol: bool,
}

impl StackFrame {
    /** Create a StackFrame with only a return address and no symbol */
    pub const fn from_addr(addr: u64) -> Self {
        StackFrame {
            return_addr: addr,
            symbol: [0u8; SYMBOL_NAME_MAX_LEN],
            has_symbol: false,
        }
    }
}

/** Fixed-capacity array of stack frames */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StackFrameArray {
    /** Stack frames */
    pub frames: [StackFrame; MAX_STACK_FRAMES],
    /** Number of valid frames */
    pub count: u8,
    /** Reason the backtrace was truncated */
    pub truncate_reason: UnwindTruncateReason,
}

impl StackFrameArray {
    /** Create an empty StackFrameArray */
    pub const fn new() -> Self {
        StackFrameArray {
            frames: [StackFrame::from_addr(0); MAX_STACK_FRAMES],
            count: 0,
            truncate_reason: UnwindTruncateReason::None,
        }
    }

    /** Push a frame; returns false if capacity exceeded */
    pub fn push(&mut self, frame: StackFrame) -> bool {
        if (self.count as usize) >= MAX_STACK_FRAMES {
            self.truncate_reason = UnwindTruncateReason::MaxFrames;
            return false;
        }
        self.frames[self.count as usize] = frame;
        self.count += 1;
        true
    }
}

impl Default for StackFrameArray {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// TombstoneRecord
// ---------------------------------------------------------------------------

/** Complete tombstone crash record containing all captured crash context */
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TombstoneRecord {
    /** Format version of this record */
    pub version: u32,
    /** Crash timestamp in nanoseconds since boot */
    pub timestamp: u64,
    /** Process ID of the crashed process */
    pub pid: u32,
    /** Thread ID of the crashed thread */
    pub tid: u32,
    /** Process name (zero-padded) */
    pub process_name: [u8; PROCESS_NAME_MAX_LEN],
    /** Classification of why the crash occurred */
    pub crash_reason: CrashReason,
    /** Signal number if the crash was caused by a signal, 0 otherwise */
    pub signal_number: u8,
    /** Architecture on which the crash occurred */
    pub arch_id: ArchId,
    /** General-purpose register values at crash time */
    pub registers: RegisterArray,
    /** Stack pointer at crash time */
    pub sp: u64,
    /** Program counter at crash time */
    pub pc: u64,
    /** Faulting address (FAR/CR2/badaddr) */
    pub fault_addr: u64,
    /** Exception syndrome register (ESR/error code/ESTAT) */
    pub esr: u64,
    /** Processor state register (PSTATE/RFLAGS/CRMD) */
    pub pstate: u64,
    /** Stack backtrace frames */
    pub stack_frames: StackFrameArray,
    /** Whether the backtrace was truncated */
    pub truncated: bool,
    /** Whether CPU context collection was incomplete */
    pub context_incomplete: bool,
    /** Number of merged crashes (for deduplication) */
    pub crash_count: u32,
    /** CRC32 checksum of the serialized record body */
    pub checksum: u32,
}

impl TombstoneRecord {
    /** Create a minimal TombstoneRecord with default/zero values */
    pub fn new() -> Self {
        TombstoneRecord {
            version: TOMBSTONE_FORMAT_VERSION,
            timestamp: 0,
            pid: 0,
            tid: 0,
            process_name: [0u8; PROCESS_NAME_MAX_LEN],
            crash_reason: CrashReason::Unknown,
            signal_number: 0,
            arch_id: ArchId::current(),
            registers: RegisterArray::new(),
            sp: 0,
            pc: 0,
            fault_addr: 0,
            esr: 0,
            pstate: 0,
            stack_frames: StackFrameArray::new(),
            truncated: false,
            context_incomplete: false,
            crash_count: 1,
            checksum: 0,
        }
    }

    /** Validate the record: check version and format consistency */
    pub fn validate(&self) -> Result<(), TombstoneError> {
        if self.version != TOMBSTONE_FORMAT_VERSION {
            return Err(TombstoneError::SerializeError);
        }
        if self.registers.count as usize > MAX_REGISTERS {
            return Err(TombstoneError::SerializeError);
        }
        if self.stack_frames.count as usize > MAX_STACK_FRAMES {
            return Err(TombstoneError::SerializeError);
        }
        Ok(())
    }

    /** Compute CRC32 over the record body (excluding the checksum field itself) */
    pub fn compute_checksum(&self) -> u32 {
        let mut buf: [u8; 512] = [0u8; 512];
        let len = self.serialize_body_into(&mut buf);
        crc32(&buf[..len])
    }

    /** Serialize the record body (everything except magic/version/length/checksum)
     *  into the provided buffer. Returns the number of bytes written. */
    pub fn serialize_body_into(&self, buf: &mut [u8]) -> usize {
        let mut off: usize = 0;

        macro_rules! write_u64 {
            ($v:expr) => {{
                let v = $v.to_le_bytes();
                if off + 8 <= buf.len() {
                    buf[off..off + 8].copy_from_slice(&v);
                }
                off += 8;
            }};
        }
        macro_rules! write_u32 {
            ($v:expr) => {{
                let v = $v.to_le_bytes();
                if off + 4 <= buf.len() {
                    buf[off..off + 4].copy_from_slice(&v);
                }
                off += 4;
            }};
        }
        macro_rules! write_u8 {
            ($v:expr) => {{
                if off < buf.len() {
                    buf[off] = $v;
                }
                off += 1;
            }};
        }
        macro_rules! write_bytes {
            ($src:expr, $len:expr) => {{
                if off + $len <= buf.len() {
                    buf[off..off + $len].copy_from_slice(&$src[..$len]);
                }
                off += $len;
            }};
        }

        write_u64!(self.timestamp);
        write_u32!(self.pid);
        write_u32!(self.tid);
        write_bytes!(self.process_name, PROCESS_NAME_MAX_LEN);
        write_u8!(self.crash_reason as u8);
        write_u8!(self.signal_number);
        write_u8!(self.arch_id as u8);
        write_u8!(self.registers.count);
        for i in 0..MAX_REGISTERS {
            write_u64!(self.registers.regs[i]);
        }
        write_u64!(self.sp);
        write_u64!(self.pc);
        write_u64!(self.fault_addr);
        write_u64!(self.esr);
        write_u64!(self.pstate);
        write_u8!(self.stack_frames.count);
        write_u8!(self.stack_frames.truncate_reason as u8);
        for i in 0..MAX_STACK_FRAMES {
            write_u64!(self.stack_frames.frames[i].return_addr);
            write_u8!(self.stack_frames.frames[i].has_symbol as u8);
            write_bytes!(self.stack_frames.frames[i].symbol, SYMBOL_NAME_MAX_LEN);
        }
        write_u8!(self.truncated as u8);
        write_u8!(self.context_incomplete as u8);
        write_u32!(self.crash_count);

        off
    }

    /** Serialize the full tombstone record into buf.
     *  Format: magic(4) + version(4) + body_length(4) + body(N) + checksum(4)
     *  Returns the total number of bytes written, or 0 if buf is too small. */
    pub fn serialize_into(&self, buf: &mut [u8]) -> usize {
        let header_len: usize = 4 + 4 + 4;
        let trailer_len: usize = 4;
        let mut body_buf: [u8; 512] = [0u8; 512];
        let body_len = self.serialize_body_into(&mut body_buf);
        let total = header_len + body_len + trailer_len;

        if total > buf.len() {
            return 0;
        }

        let mut off: usize = 0;
        buf[off..off + 4].copy_from_slice(&TOMBSTONE_FILE_MAGIC.to_le_bytes());
        off += 4;
        buf[off..off + 4].copy_from_slice(&TOMBSTONE_FORMAT_VERSION.to_le_bytes());
        off += 4;
        buf[off..off + 4].copy_from_slice(&(body_len as u32).to_le_bytes());
        off += 4;
        buf[off..off + body_len].copy_from_slice(&body_buf[..body_len]);
        off += body_len;

        let cksum = crc32(&buf[..off]);
        buf[off..off + 4].copy_from_slice(&cksum.to_le_bytes());
        off += 4;

        off
    }

    /** Deserialize a TombstoneRecord from a byte slice.
     *  Validates magic, version, and CRC32 checksum. */
    pub fn deserialize_from(data: &[u8]) -> Result<Self, TombstoneError> {
        if data.len() < 12 {
            return Err(TombstoneError::SerializeError);
        }

        let magic = u32::from_le_bytes(
            data[0..4]
                .try_into()
                .map_err(|_| TombstoneError::SerializeError)?,
        );
        if magic != TOMBSTONE_FILE_MAGIC {
            return Err(TombstoneError::SerializeError);
        }

        let version = u32::from_le_bytes(
            data[4..8]
                .try_into()
                .map_err(|_| TombstoneError::SerializeError)?,
        );
        if version != TOMBSTONE_FORMAT_VERSION {
            return Err(TombstoneError::SerializeError);
        }

        let body_len = u32::from_le_bytes(
            data[8..12]
                .try_into()
                .map_err(|_| TombstoneError::SerializeError)?,
        ) as usize;
        let header_len: usize = 12;
        let trailer_len: usize = 4;
        let total = header_len + body_len + trailer_len;
        if data.len() < total {
            return Err(TombstoneError::SerializeError);
        }

        let stored_cksum = u32::from_le_bytes(
            data[total - 4..total]
                .try_into()
                .map_err(|_| TombstoneError::SerializeError)?,
        );
        let computed_cksum = crc32(&data[..total - 4]);
        if stored_cksum != computed_cksum {
            return Err(TombstoneError::ChecksumMismatch);
        }

        let body = &data[header_len..header_len + body_len];
        let mut rec = TombstoneRecord::new();
        let mut off: usize = 0;

        macro_rules! read_u64 {
            () => {{
                if off + 8 > body.len() {
                    return Err(TombstoneError::SerializeError);
                }
                let v = u64::from_le_bytes(
                    body[off..off + 8]
                        .try_into()
                        .map_err(|_| TombstoneError::SerializeError)?,
                );
                off += 8;
                v
            }};
        }
        macro_rules! read_u32 {
            () => {{
                if off + 4 > body.len() {
                    return Err(TombstoneError::SerializeError);
                }
                let v = u32::from_le_bytes(
                    body[off..off + 4]
                        .try_into()
                        .map_err(|_| TombstoneError::SerializeError)?,
                );
                off += 4;
                v
            }};
        }
        macro_rules! read_u8 {
            () => {{
                if off >= body.len() {
                    return Err(TombstoneError::SerializeError);
                }
                let v = body[off];
                off += 1;
                v
            }};
        }
        macro_rules! read_bytes {
            ($len:expr) => {{
                if off + $len > body.len() {
                    return Err(TombstoneError::SerializeError);
                }
                let mut arr = [0u8; 256];
                let copy_len = if $len > 256 { 256 } else { $len };
                arr[..copy_len].copy_from_slice(&body[off..off + copy_len]);
                off += $len;
                arr
            }};
        }

        rec.timestamp = read_u64!();
        rec.pid = read_u32!();
        rec.tid = read_u32!();
        {
            let pn = read_bytes!(PROCESS_NAME_MAX_LEN);
            rec.process_name
                .copy_from_slice(&pn[..PROCESS_NAME_MAX_LEN]);
        }
        rec.crash_reason = match read_u8!() {
            0 => CrashReason::FatalSignal,
            1 => CrashReason::IllegalAccess,
            2 => CrashReason::IllegalInstruction,
            3 => CrashReason::FpException,
            4 => CrashReason::KernelFault,
            5 => CrashReason::Watchdog,
            6 => CrashReason::StackOverflow,
            _ => CrashReason::Unknown,
        };
        rec.signal_number = read_u8!();
        rec.arch_id = match read_u8!() {
            0 => ArchId::Arm64,
            1 => ArchId::X64,
            _ => ArchId::LoongArch64,
        };
        rec.registers.count = read_u8!();
        for i in 0..MAX_REGISTERS {
            rec.registers.regs[i] = read_u64!();
        }
        rec.sp = read_u64!();
        rec.pc = read_u64!();
        rec.fault_addr = read_u64!();
        rec.esr = read_u64!();
        rec.pstate = read_u64!();
        rec.stack_frames.count = read_u8!();
        rec.stack_frames.truncate_reason = match read_u8!() {
            1 => UnwindTruncateReason::CorruptStack,
            2 => UnwindTruncateReason::InvalidFp,
            3 => UnwindTruncateReason::UnmappedMemory,
            4 => UnwindTruncateReason::MaxFrames,
            _ => UnwindTruncateReason::None,
        };
        for i in 0..MAX_STACK_FRAMES {
            rec.stack_frames.frames[i].return_addr = read_u64!();
            rec.stack_frames.frames[i].has_symbol = read_u8!() != 0;
            {
                let sym = read_bytes!(SYMBOL_NAME_MAX_LEN);
                rec.stack_frames.frames[i]
                    .symbol
                    .copy_from_slice(&sym[..SYMBOL_NAME_MAX_LEN]);
            }
        }
        rec.truncated = read_u8!() != 0;
        rec.context_incomplete = read_u8!() != 0;
        rec.crash_count = read_u32!();
        rec.checksum = stored_cksum;
        rec.version = version;

        Ok(rec)
    }
}

impl Default for TombstoneRecord {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// CRC32 (table-driven, no_std compatible)
// ---------------------------------------------------------------------------

/** CRC32 lookup table (IEEE 802.3 polynomial 0xEDB88320) */
static CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0u32;
    while i < 256 {
        let mut crc = i;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i as usize] = crc;
        i += 1;
    }
    table
};

/** Compute CRC32 over a byte slice using the IEEE polynomial */
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data.iter() {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[idx];
    }
    !crc
}

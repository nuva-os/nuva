/*
 * Nuva OS - Kernel - Core - Time
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
use crate::{pr_info};
/*
 * Nuva OS - Kernel - Time Management
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel time and clock management.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// Get current time in milliseconds
pub fn get_time_ms() -> u64 {
    // TODO: implement proper time retrieval from hardware timer
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Time Value (nanoseconds)
pub type Nsec = u64;
pub type Sec = u64;
pub type Usec = u64;
pub type Msec = u64;

/// Timespec
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Timespec {
    pub seconds: Sec,
    pub nanoseconds: Nsec,
}

impl Timespec {
    pub const fn new(sec: Sec, nsec: Nsec) -> Self {
        Timespec { seconds: sec, nanoseconds: nsec }
    }
    
    pub const fn zero() -> Self {
        Timespec { seconds: 0, nanoseconds: 0 }
    }
    
    /// Normalize (handle nsec overflow)
    pub fn normalize(&mut self) {
        while self.nanoseconds >= 1_000_000_000 {
            self.seconds += 1;
            self.nanoseconds -= 1_000_000_000;
        }
    }
    
    /// Convert to nanoseconds
    pub fn as_nanos(&self) -> Nsec {
        self.seconds * 1_000_000_000 + self.nanoseconds
    }
    
    /// Convert to microseconds
    pub fn as_micros(&self) -> Usec {
        self.seconds * 1_000_000 + self.nanoseconds / 1_000
    }
    
    /// Convert to milliseconds
    pub fn as_millis(&self) -> Msec {
        self.seconds * 1_000 + self.nanoseconds / 1_000_000
    }
    
    /// From nanoseconds
    pub fn from_nanos(nanos: Nsec) -> Self {
        Timespec {
            seconds: nanos / 1_000_000_000,
            nanoseconds: nanos % 1_000_000_000,
        }
    }
    
    /// From microseconds
    pub fn from_micros(micros: Usec) -> Self {
        Timespec {
            seconds: micros / 1_000_000,
            nanoseconds: (micros % 1_000_000) * 1_000,
        }
    }
    
    /// From milliseconds
    pub fn from_millis(millis: Msec) -> Self {
        Timespec {
            seconds: millis / 1_000,
            nanoseconds: (millis % 1_000) * 1_000_000,
        }
    }
    
    /// Add
    pub fn add(&self, other: &Timespec) -> Timespec {
        let mut result = Timespec {
            seconds: self.seconds + other.seconds,
            nanoseconds: self.nanoseconds + other.nanoseconds,
        };
        result.normalize();
        result
    }
    
    /// Subtract
    pub fn sub(&self, other: &Timespec) -> Timespec {
        let mut nsec = self.nanoseconds as i64 - other.nanoseconds as i64;
        let mut sec = self.seconds as i64 - other.seconds as i64;
        
        if nsec < 0 {
            sec -= 1;
            nsec += 1_000_000_000;
        }
        
        Timespec {
            seconds: sec as u64,
            nanoseconds: nsec as u64,
        }
    }
    
    /// Compare
    pub fn cmp(&self, other: &Timespec) -> core::cmp::Ordering {
        if self.seconds < other.seconds {
            core::cmp::Ordering::Less
        } else if self.seconds > other.seconds {
            core::cmp::Ordering::Greater
        } else if self.nanoseconds < other.nanoseconds {
            core::cmp::Ordering::Less
        } else if self.nanoseconds > other.nanoseconds {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }
}

/// Timeval
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Timeval {
    pub seconds: Sec,
    pub microseconds: Usec,
}

impl Timeval {
    pub fn from_timespec(ts: &Timespec) -> Self {
        Timeval {
            seconds: ts.seconds,
            microseconds: ts.nanoseconds / 1_000,
        }
    }
    
    pub fn to_timespec(&self) -> Timespec {
        Timespec {
            seconds: self.seconds,
            nanoseconds: self.microseconds * 1_000,
        }
    }
}

/// Clock Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockType {
    /// Real-time clock
    Realtime = 0,
    /// Monotonic clock
    Monotonic = 1,
    /// Process CPU time
    ProcessCputime = 2,
    /// Thread CPU time
    ThreadCputime = 3,
    /// Monotonic raw
    MonotonicRaw = 4,
    /// Boot time
    Boottime = 5,
    /// Real-time alarm
    RealtimeAlarm = 6,
    /// Boot time alarm
    BoottimeAlarm = 7,
    /// Tai clock
    Tai = 11,
}

/// Clock Source
pub struct ClockSource {
    /// Clock name
    pub name: [u8; 32],
    /// Clock type
    pub clock_type: ClockType,
    /// Read function
    pub read: Option<unsafe extern "C" fn() -> u64>,
    /// Mask
    pub mask: u64,
    /// Mult (for conversion)
    pub mult: u32,
    /// Shift (for conversion)
    pub shift: u32,
    /// Max cycles
    pub max_cycles: u64,
    /// Max idle ns
    pub max_idle_ns: u64,
    /// Flags
    pub flags: AtomicU32,
    /// Rating
    pub rating: u32,
    /// Enable count
    pub enable_count: AtomicU32,
}

/// Clock Source Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ClockSourceFlags: u32 {
        const CONTINUOUS = 1 << 0;
        const MUST_VERIFY = 1 << 1;
        const VDSO_TIMEGEN = 1 << 2;
        const UNSTABLE = 1 << 3;
        const VALID_FOR_HRES = 1 << 4;
    }
}

impl ClockSource {
    pub fn new(name: &[u8], clock_type: ClockType) -> Self {
        let mut name_arr = [0u8; 32];
        let len = name.len().min(31);
        name_arr[..len].copy_from_slice(&name[..len]);
        
        ClockSource {
            name: name_arr,
            clock_type,
            read: None,
            mask: u64::MAX,
            mult: 1,
            shift: 0,
            max_cycles: u64::MAX,
            max_idle_ns: 0,
            flags: AtomicU32::new(ClockSourceFlags::CONTINUOUS.bits()),
            rating: 0,
            enable_count: AtomicU32::new(0),
        }
    }
    
    /// Read clock
    pub fn read_clock(&self) -> u64 {
        if let Some(read) = self.read {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { read() }
        } else {
            0
        }
    }
    
    /// Cycles to nanoseconds
    pub fn cycles_to_ns(&self, cycles: u64) -> Nsec {
        ((cycles as u128 * self.mult as u128) >> self.shift) as u64
    }
    
    /// Nanoseconds to cycles
    pub fn ns_to_cycles(&self, ns: Nsec) -> u64 {
        (ns as u128 >> self.shift) as u64 / self.mult as u64
    }
}

/// Time Manager
pub struct TimeManager {
    /// Current time (monotonic ns)
    pub monotonic_time: AtomicU64,
    /// Current time (realtime ns)
    pub realtime_time: AtomicU64,
    /// Boot time (realtime ns at boot)
    pub boot_time: AtomicU64,
    /// Wall time offset
    pub wall_time_offset: AtomicU64,
    /// Clock sources
    pub clock_sources: *mut ClockSource,
    /// Best clock source
    pub best_clock: AtomicU32,
    /// Tick period (ns)
    pub tick_period: AtomicU64,
    /// Tick count
    pub tick_count: AtomicU64,
    /// HZ (ticks per second)
    pub hz: AtomicU32,
    /// NTP offset
    pub ntp_offset: AtomicU64,
    /// Statistics
    pub stats: TimeStats,
}

/// Time Statistics
pub struct TimeStats {
    pub total_ticks: AtomicU64,
    pub total_ns: AtomicU64,
    pub adj_count: AtomicU64,
}

impl TimeStats {
    pub const fn new() -> Self {
        TimeStats {
            total_ticks: AtomicU64::new(0),
            total_ns: AtomicU64::new(0),
            adj_count: AtomicU64::new(0),
        }
    }
}

impl TimeManager {
    pub const fn new() -> Self {
        TimeManager {
            monotonic_time: AtomicU64::new(0),
            realtime_time: AtomicU64::new(0),
            boot_time: AtomicU64::new(0),
            wall_time_offset: AtomicU64::new(0),
            clock_sources: core::ptr::null_mut(),
            best_clock: AtomicU32::new(0),
            tick_period: AtomicU64::new(1_000_000), // 1ms default
            tick_count: AtomicU64::new(0),
            hz: AtomicU32::new(1000), // 1000 Hz default
            ntp_offset: AtomicU64::new(0),
            stats: TimeStats::new(),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        // Set default tick period
        self.tick_period.store(1_000_000_000 / 1000, Ordering::Release);
        
        log_info!("Time manager initialized");
    }
    
    /// Get monotonic time
    pub fn get_monotonic(&self) -> Timespec {
        let ns = self.monotonic_time.load(Ordering::Acquire);
        Timespec::from_nanos(ns)
    }
    
    /// Get real time
    pub fn get_realtime(&self) -> Timespec {
        let ns = self.realtime_time.load(Ordering::Acquire);
        Timespec::from_nanos(ns)
    }
    
    /// Get boot time
    pub fn get_boottime(&self) -> Timespec {
        let ns = self.boot_time.load(Ordering::Acquire);
        Timespec::from_nanos(ns)
    }
    
    /// Set real time
    pub fn set_realtime(&mut self, ts: &Timespec) {
        let ns = ts.as_nanos();
        let mono = self.monotonic_time.load(Ordering::Acquire);
        
        self.wall_time_offset.store(ns.saturating_sub(mono), Ordering::Release);
        self.realtime_time.store(ns, Ordering::Release);
    }
    
    /// Tick handler
    pub fn tick(&mut self) {
        let period = self.tick_period.load(Ordering::Acquire);
        
        // Update monotonic time
        self.monotonic_time.fetch_add(period, Ordering::AcqRel);
        
        // Update real time
        self.realtime_time.fetch_add(period, Ordering::AcqRel);
        
        // Update tick count
        self.tick_count.fetch_add(1, Ordering::AcqRel);
        
        // Update stats
        self.stats.total_ticks.fetch_add(1, Ordering::AcqRel);
        self.stats.total_ns.fetch_add(period, Ordering::AcqRel);
    }
    
    /// Get time since boot
    pub fn uptime(&self) -> Timespec {
        self.get_monotonic()
    }
    
    /// Get jiffies
    pub fn jiffies(&self) -> u64 {
        self.tick_count.load(Ordering::Acquire)
    }
    
    /// Jiffies to milliseconds
    pub fn jiffies_to_msecs(&self, jiffies: u64) -> Msec {
        let hz = self.hz.load(Ordering::Acquire);
        jiffies * 1000 / hz as u64
    }
    
    /// Milliseconds to jiffies
    pub fn msecs_to_jiffies(&self, msecs: Msec) -> u64 {
        let hz = self.hz.load(Ordering::Acquire);
        msecs * hz as u64 / 1000
    }
    
    /// Register clock source
    pub fn register_clock(&mut self, clock: *mut ClockSource) -> i32 {
        if clock.is_null() {
            return Errno::Einval.to_ret_i32();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*clock).enable_count.fetch_add(1, Ordering::AcqRel);
        }
        
        // TODO: Add to list and select best
        
        0
    }
    
    /// Adjust time (NTP)
    pub fn adjtime(&mut self, delta: i64) {
        if delta > 0 {
            self.monotonic_time.fetch_add(delta as u64, Ordering::AcqRel);
            self.realtime_time.fetch_add(delta as u64, Ordering::AcqRel);
        } else {
            self.monotonic_time.fetch_sub((-delta) as u64, Ordering::AcqRel);
            self.realtime_time.fetch_sub((-delta) as u64, Ordering::AcqRel);
        }
        
        self.stats.adj_count.fetch_add(1, Ordering::AcqRel);
    }
}

/// Global time manager
static TIME_MANAGER: crate::sync_oncelock::OnceLock<TimeManager> = crate::sync_oncelock::OnceLock::new();

/// Get time manager
pub fn time_manager() -> &'static TimeManager {
    TIME_MANAGER.get_or_init(TimeManager::new)
}

pub fn init_time_manager() -> &'static TimeManager {
    TIME_MANAGER.get_or_init(TimeManager::new)
}

/// Initialize time
pub fn init_time() {
    let mgr = time_manager();
    mgr.init();
}

// Convenience functions

/// Get monotonic time
pub fn ktime_get() -> Timespec {
    time_manager().get_monotonic()
}

/// Get real time
pub fn ktime_get_real() -> Timespec {
    time_manager().get_realtime()
}

/// Get boot time
pub fn ktime_get_boottime() -> Timespec {
    time_manager().get_boottime()
}

/// Get nanoseconds
pub fn ktime_get_ns() -> Nsec {
    time_manager().monotonic_time.load(Ordering::Acquire)
}

/// Get microseconds
pub fn ktime_get_us() -> Usec {
    ktime_get_ns() / 1_000
}

/// Get milliseconds
pub fn ktime_get_ms() -> Msec {
    ktime_get_ns() / 1_000_000
}

/// Get seconds
pub fn ktime_get_seconds() -> Sec {
    ktime_get_ns() / 1_000_000_000
}

/// Get jiffies
pub fn get_jiffies() -> u64 {
    time_manager().jiffies()
}

/// Jiffies to milliseconds
pub fn jiffies_to_msecs(jiffies: u64) -> Msec {
    time_manager().jiffies_to_msecs(jiffies)
}

/// Milliseconds to jiffies
pub fn msecs_to_jiffies(msecs: Msec) -> u64 {
    time_manager().msecs_to_jiffies(msecs)
}

/// Timer Wheel
pub struct TimerWheel {
    /// Current time
    pub now: AtomicU64,
    /// Buckets
    pub buckets: [*mut Timer; 256],
    /// Current index
    pub index: AtomicU32,
    /// Timer count
    pub timer_count: AtomicU32,
}

/// Timer
pub struct Timer {
    /// Expiry time (jiffies)
    pub expires: u64,
    /// Callback
    pub callback: Option<unsafe extern "C" fn(*mut Timer)>,
    /// Data
    pub data: *mut core::ffi::c_void,
    /// Flags
    pub flags: AtomicU32,
    /// Next timer
    pub next: *mut Timer,
}

/// Timer Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct TimerFlags: u32 {
        const ACTIVE = 1 << 0;
        const PENDING = 1 << 1;
        const DEFERRABLE = 1 << 2;
        const PINNED = 1 << 3;
    }
}

impl Timer {
    pub fn new(callback: unsafe extern "C" fn(*mut Timer)) -> Self {
        Timer {
            expires: 0,
            callback: Some(callback),
            data: core::ptr::null_mut(),
            flags: AtomicU32::new(0),
            next: core::ptr::null_mut(),
        }
    }
    
    /// Mod timer
    pub fn mod_timer(&mut self, expires: u64) {
        self.expires = expires;
        self.flags.fetch_or(TimerFlags::PENDING.bits(), Ordering::AcqRel);
    }
    
    /// Add timer
    pub fn add_timer(&mut self) {
        self.flags.fetch_or(TimerFlags::ACTIVE.bits(), Ordering::AcqRel);
    }
    
    /// Del timer
    pub fn del_timer(&mut self) {
        self.flags.fetch_and(!TimerFlags::ACTIVE.bits(), Ordering::AcqRel);
    }
    
    /// Check if active
    pub fn is_active(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & TimerFlags::ACTIVE.bits()) != 0
    }
}

/// Delay functions

/// Busy wait for nanoseconds
pub fn ndelay(ns: Nsec) {
    // TODO: Use calibrated delay loop
    let _ = ns;
}

/// Busy wait for microseconds
pub fn udelay(us: Usec) {
    // TODO: Use calibrated delay loop
    let _ = us;
}

/// Busy wait for milliseconds
pub fn mdelay(ms: Msec) {
    // TODO: Use calibrated delay loop
    let _ = ms;
}

/// Time comparison macros
#[macro_export]
macro_rules! time_after {
    ($a:expr, $b:expr) => {
        ({$a} as i64 - {$b} as i64) > 0
    };
}

#[macro_export]
macro_rules! time_before {
    ($a:expr, $b:expr) => {
        ({$b} as i64 - {$a} as i64) > 0
    };
}

#[macro_export]
macro_rules! time_after_eq {
    ($a:expr, $b:expr) => {
        ({$a} as i64 - {$b} as i64) >= 0
    };
}

#[macro_export]
macro_rules! time_before_eq {
    ($a:expr, $b:expr) => {
        ({$b} as i64 - {$a} as i64) >= 0
    };
}

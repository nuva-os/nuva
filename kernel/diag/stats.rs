/*
 * Nuva OS - Kernel - Diag - Stats
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
 * Nuva OS - Kernel - Statistics
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel statistics and performance monitoring.
 */

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// CPU Statistics
#[repr(C)]
pub struct CpuStats {
    /// User time (jiffies)
    pub user: AtomicU64,
    /// Nice time
    pub nice: AtomicU64,
    /// System time
    pub system: AtomicU64,
    /// Idle time
    pub idle: AtomicU64,
    /// I/O wait time
    pub iowait: AtomicU64,
    /// IRQ time
    pub irq: AtomicU64,
    /// Soft IRQ time
    pub softirq: AtomicU64,
    /// Steal time
    pub steal: AtomicU64,
    /// Guest time
    pub guest: AtomicU64,
    /// Guest nice time
    pub guest_nice: AtomicU64,
}

impl Clone for CpuStats {
    fn clone(&self) -> Self {
        Self {
            user: AtomicU64::new(self.user.load(core::sync::atomic::Ordering::Relaxed)),
            nice: AtomicU64::new(self.nice.load(core::sync::atomic::Ordering::Relaxed)),
            system: AtomicU64::new(self.system.load(core::sync::atomic::Ordering::Relaxed)),
            idle: AtomicU64::new(self.idle.load(core::sync::atomic::Ordering::Relaxed)),
            iowait: AtomicU64::new(self.iowait.load(core::sync::atomic::Ordering::Relaxed)),
            irq: AtomicU64::new(self.irq.load(core::sync::atomic::Ordering::Relaxed)),
            softirq: AtomicU64::new(self.softirq.load(core::sync::atomic::Ordering::Relaxed)),
            steal: AtomicU64::new(self.steal.load(core::sync::atomic::Ordering::Relaxed)),
            guest: AtomicU64::new(self.guest.load(core::sync::atomic::Ordering::Relaxed)),
            guest_nice: AtomicU64::new(self.guest_nice.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl CpuStats {
    pub const fn new() -> Self {
        CpuStats {
            user: AtomicU64::new(0),
            nice: AtomicU64::new(0),
            system: AtomicU64::new(0),
            idle: AtomicU64::new(0),
            iowait: AtomicU64::new(0),
            irq: AtomicU64::new(0),
            softirq: AtomicU64::new(0),
            steal: AtomicU64::new(0),
            guest: AtomicU64::new(0),
            guest_nice: AtomicU64::new(0),
        }
    }
    
    /// Get total time
    pub fn total(&self) -> u64 {
        self.user.load(Ordering::Acquire)
            + self.nice.load(Ordering::Acquire)
            + self.system.load(Ordering::Acquire)
            + self.idle.load(Ordering::Acquire)
            + self.iowait.load(Ordering::Acquire)
            + self.irq.load(Ordering::Acquire)
            + self.softirq.load(Ordering::Acquire)
            + self.steal.load(Ordering::Acquire)
            + self.guest.load(Ordering::Acquire)
            + self.guest_nice.load(Ordering::Acquire)
    }
    
    /// Get usage percentage
    pub fn usage_percent(&self) -> u32 {
        let total = self.total();
        if total == 0 {
            return 0;
        }
        
        let idle = self.idle.load(Ordering::Acquire);
        let used = total.saturating_sub(idle);
        
        ((used * 100) / total) as u32
    }
}

/// Memory Statistics
#[repr(C)]
pub struct MemStats {
    /// Total memory (bytes)
    pub total: AtomicU64,
    /// Free memory
    pub free: AtomicU64,
    /// Available memory
    pub available: AtomicU64,
    /// Buffers
    pub buffers: AtomicU64,
    /// Cached
    pub cached: AtomicU64,
    /// Swap total
    pub swap_total: AtomicU64,
    /// Swap free
    pub swap_free: AtomicU64,
    /// Shared memory
    pub shared: AtomicU64,
    /// Slab memory
    pub slab: AtomicU64,
    /// Kernel stack
    pub kernel_stack: AtomicU64,
    /// Page tables
    pub page_tables: AtomicU64,
    /// Anonymous memory
    pub anon: AtomicU64,
    /// Committed memory
    pub committed: AtomicU64,
    /// Vmalloc total
    pub vmalloc_total: AtomicU64,
    /// Vmalloc used
    pub vmalloc_used: AtomicU64,
}

impl MemStats {
    pub const fn new() -> Self {
        MemStats {
            total: AtomicU64::new(0),
            free: AtomicU64::new(0),
            available: AtomicU64::new(0),
            buffers: AtomicU64::new(0),
            cached: AtomicU64::new(0),
            swap_total: AtomicU64::new(0),
            swap_free: AtomicU64::new(0),
            shared: AtomicU64::new(0),
            slab: AtomicU64::new(0),
            kernel_stack: AtomicU64::new(0),
            page_tables: AtomicU64::new(0),
            anon: AtomicU64::new(0),
            committed: AtomicU64::new(0),
            vmalloc_total: AtomicU64::new(0),
            vmalloc_used: AtomicU64::new(0),
        }
    }
    
    /// Get used memory
    pub fn used(&self) -> u64 {
        self.total.load(Ordering::Acquire)
            .saturating_sub(self.free.load(Ordering::Acquire))
    }
    
    /// Get usage percentage
    pub fn usage_percent(&self) -> u32 {
        let total = self.total.load(Ordering::Acquire);
        if total == 0 {
            return 0;
        }
        
        let used = self.used();
        ((used * 100) / total) as u32
    }
}

/// Process Statistics
#[repr(C)]
pub struct ProcStats {
    /// Total processes
    pub total: AtomicU64,
    /// Running processes
    pub running: AtomicU64,
    /// Sleeping processes
    pub sleeping: AtomicU64,
    /// Stopped processes
    pub stopped: AtomicU64,
    /// Zombie processes
    pub zombie: AtomicU64,
    /// Total threads
    pub threads: AtomicU64,
    /// Total file descriptors
    pub fd_total: AtomicU64,
    /// Context switches
    pub ctxt: AtomicU64,
    /// Processes created
    pub processes: AtomicU64,
    /// Processes running
    pub procs_running: AtomicU32,
    /// Processes blocked
    pub procs_blocked: AtomicU32,
}

impl ProcStats {
    pub const fn new() -> Self {
        ProcStats {
            total: AtomicU64::new(0),
            running: AtomicU64::new(0),
            sleeping: AtomicU64::new(0),
            stopped: AtomicU64::new(0),
            zombie: AtomicU64::new(0),
            threads: AtomicU64::new(0),
            fd_total: AtomicU64::new(0),
            ctxt: AtomicU64::new(0),
            processes: AtomicU64::new(0),
            procs_running: AtomicU32::new(0),
            procs_blocked: AtomicU32::new(0),
        }
    }
}

/// Interrupt Statistics
#[repr(C)]
pub struct IrqStats {
    /// Total interrupts
    pub total: AtomicU64,
    /// Timer interrupts
    pub timer: AtomicU64,
    /// IPI interrupts
    pub ipi: AtomicU64,
    /// Device interrupts
    pub device: AtomicU64,
    /// Spurious interrupts
    pub spurious: AtomicU64,
    /// Soft IRQs
    pub softirq: AtomicU64,
}

impl IrqStats {
    pub const fn new() -> Self {
        IrqStats {
            total: AtomicU64::new(0),
            timer: AtomicU64::new(0),
            ipi: AtomicU64::new(0),
            device: AtomicU64::new(0),
            spurious: AtomicU64::new(0),
            softirq: AtomicU64::new(0),
        }
    }
}

/// I/O Statistics
#[repr(C)]
pub struct IoStats {
    /// Read operations
    pub read_ops: AtomicU64,
    /// Write operations
    pub write_ops: AtomicU64,
    /// Bytes read
    pub read_bytes: AtomicU64,
    /// Bytes written
    pub write_bytes: AtomicU64,
    /// Read time (ms)
    pub read_time: AtomicU64,
    /// Write time (ms)
    pub write_time: AtomicU64,
    /// I/O wait time
    pub wait_time: AtomicU64,
}

impl IoStats {
    pub const fn new() -> Self {
        IoStats {
            read_ops: AtomicU64::new(0),
            write_ops: AtomicU64::new(0),
            read_bytes: AtomicU64::new(0),
            write_bytes: AtomicU64::new(0),
            read_time: AtomicU64::new(0),
            write_time: AtomicU64::new(0),
            wait_time: AtomicU64::new(0),
        }
    }
}

/// Network Statistics
#[repr(C)]
pub struct NetStats {
    /// Bytes received
    pub rx_bytes: AtomicU64,
    /// Packets received
    pub rx_packets: AtomicU64,
    /// Receive errors
    pub rx_errors: AtomicU64,
    /// Receive dropped
    pub rx_dropped: AtomicU64,
    /// Bytes transmitted
    pub tx_bytes: AtomicU64,
    /// Packets transmitted
    pub tx_packets: AtomicU64,
    /// Transmit errors
    pub tx_errors: AtomicU64,
    /// Transmit dropped
    pub tx_dropped: AtomicU64,
    /// Collisions
    pub collisions: AtomicU64,
}

impl NetStats {
    pub const fn new() -> Self {
        NetStats {
            rx_bytes: AtomicU64::new(0),
            rx_packets: AtomicU64::new(0),
            rx_errors: AtomicU64::new(0),
            rx_dropped: AtomicU64::new(0),
            tx_bytes: AtomicU64::new(0),
            tx_packets: AtomicU64::new(0),
            tx_errors: AtomicU64::new(0),
            tx_dropped: AtomicU64::new(0),
            collisions: AtomicU64::new(0),
        }
    }
}

/// Scheduler Statistics
#[repr(C)]
pub struct SchedStats {
    /// Schedule calls
    pub schedule_calls: AtomicU64,
    /// Context switches
    pub context_switches: AtomicU64,
    /// Preemptions
    pub preemptions: AtomicU64,
    /// Wakeups
    pub wakeups: AtomicU64,
    /// Load average (1 min)
    pub load_avg_1: AtomicU64,
    /// Load average (5 min)
    pub load_avg_5: AtomicU64,
    /// Load average (15 min)
    pub load_avg_15: AtomicU64,
    /// Run queue latency (ns)
    pub rq_latency: AtomicU64,
}

impl SchedStats {
    pub const fn new() -> Self {
        SchedStats {
            schedule_calls: AtomicU64::new(0),
            context_switches: AtomicU64::new(0),
            preemptions: AtomicU64::new(0),
            wakeups: AtomicU64::new(0),
            load_avg_1: AtomicU64::new(0),
            load_avg_5: AtomicU64::new(0),
            load_avg_15: AtomicU64::new(0),
            rq_latency: AtomicU64::new(0),
        }
    }
}

/// Kernel Statistics Manager
pub struct StatsManager {
    /// CPU stats per CPU
    pub cpu: [CpuStats; 256],
    /// Memory stats
    pub mem: MemStats,
    /// Process stats
    pub proc: ProcStats,
    /// IRQ stats
    pub irq: IrqStats,
    /// I/O stats
    pub io: IoStats,
    /// Network stats
    pub net: NetStats,
    /// Scheduler stats
    pub sched: SchedStats,
    /// Boot time
    pub boot_time: AtomicU64,
    /// Uptime (jiffies)
    pub uptime: AtomicU64,
}

impl StatsManager {
    pub const fn new() -> Self {
        StatsManager {
            cpu: [const { CpuStats::new() }; 256],
            mem: MemStats::new(),
            proc: ProcStats::new(),
            irq: IrqStats::new(),
            io: IoStats::new(),
            net: NetStats::new(),
            sched: SchedStats::new(),
            boot_time: AtomicU64::new(0),
            uptime: AtomicU64::new(0),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        // Set boot time
        self.boot_time.store(0, Ordering::Release);
        
        log_info!("Stats manager initialized");
    }
    
    /// Update uptime
    pub fn update_uptime(&self, jiffies: u64) {
        self.uptime.store(jiffies, Ordering::Release);
    }
    
    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        // Assuming HZ = 1000
        self.uptime.load(Ordering::Acquire) / 1000
    }
    
    /// Record context switch
    pub fn record_ctxt(&self) {
        self.proc.ctxt.fetch_add(1, Ordering::AcqRel);
        self.sched.context_switches.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Record schedule call
    pub fn record_schedule(&self) {
        self.sched.schedule_calls.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Record preemption
    pub fn record_preempt(&self) {
        self.sched.preemptions.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Record wakeup
    pub fn record_wakeup(&self) {
        self.sched.wakeups.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Record interrupt
    pub fn record_irq(&self) {
        self.irq.total.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Record timer interrupt
    pub fn record_timer_irq(&self) {
        self.irq.timer.fetch_add(1, Ordering::AcqRel);
        self.record_irq();
    }
    
    /// Record soft IRQ
    pub fn record_softirq(&self) {
        self.irq.softirq.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Record I/O read
    pub fn record_read(&self, bytes: u64) {
        self.io.read_ops.fetch_add(1, Ordering::AcqRel);
        self.io.read_bytes.fetch_add(bytes, Ordering::AcqRel);
    }
    
    /// Record I/O write
    pub fn record_write(&self, bytes: u64) {
        self.io.write_ops.fetch_add(1, Ordering::AcqRel);
        self.io.write_bytes.fetch_add(bytes, Ordering::AcqRel);
    }
    
    /// Record network receive
    pub fn record_net_rx(&self, bytes: u64) {
        self.net.rx_packets.fetch_add(1, Ordering::AcqRel);
        self.net.rx_bytes.fetch_add(bytes, Ordering::AcqRel);
    }
    
    /// Record network transmit
    pub fn record_net_tx(&self, bytes: u64) {
        self.net.tx_packets.fetch_add(1, Ordering::AcqRel);
        self.net.tx_bytes.fetch_add(bytes, Ordering::AcqRel);
    }
    
    /// Get CPU stats
    pub fn get_cpu_stats(&self, cpu: u32) -> &CpuStats {
        &self.cpu[cpu as usize]
    }
    
    /// Get total CPU stats
    pub fn get_total_cpu_stats(&self) -> CpuStats {
        let mut total = CpuStats::new();
        
        for cpu in &self.cpu {
            total.user.fetch_add(cpu.user.load(Ordering::Acquire), Ordering::AcqRel);
            total.nice.fetch_add(cpu.nice.load(Ordering::Acquire), Ordering::AcqRel);
            total.system.fetch_add(cpu.system.load(Ordering::Acquire), Ordering::AcqRel);
            total.idle.fetch_add(cpu.idle.load(Ordering::Acquire), Ordering::AcqRel);
            total.iowait.fetch_add(cpu.iowait.load(Ordering::Acquire), Ordering::AcqRel);
            total.irq.fetch_add(cpu.irq.load(Ordering::Acquire), Ordering::AcqRel);
            total.softirq.fetch_add(cpu.softirq.load(Ordering::Acquire), Ordering::AcqRel);
        }
        
        total
    }
    
    /// Dump stats
    pub fn dump(&self) {
        log_info!("Kernel Statistics:");
        log_info!("  Uptime: {} seconds", self.uptime_secs());
        log_info!("  Memory: {}% used", self.mem.usage_percent());
        log_info!("  Processes: {}", self.proc.total.load(Ordering::Acquire));
        log_info!("  Threads: {}", self.proc.threads.load(Ordering::Acquire));
        log_info!("  Context switches: {}", self.proc.ctxt.load(Ordering::Acquire));
        log_info!("  Interrupts: {}", self.irq.total.load(Ordering::Acquire));
        log_info!("  Network RX: {} bytes", self.net.rx_bytes.load(Ordering::Acquire));
        log_info!("  Network TX: {} bytes", self.net.tx_bytes.load(Ordering::Acquire));
    }
}

/// Global stats manager
static STATS_MANAGER: core::sync::OnceLock<StatsManager> = core::sync::OnceLock::new();

/// Get stats manager
pub fn stats_manager() -> &'static StatsManager {
    STATS_MANAGER.get_or_init(StatsManager::new)
}

pub fn init_stats_manager() -> &'static StatsManager {
    STATS_MANAGER.get_or_init(StatsManager::new)
}

/// Initialize stats
pub fn init_stats() {
    let mgr = stats_manager();
    mgr.init();
}

// Convenience functions

/// Record context switch
pub fn stats_ctxt() {
    stats_manager().record_ctxt();
}

/// Record schedule
pub fn stats_schedule() {
    stats_manager().record_schedule();
}

/// Record interrupt
pub fn stats_irq() {
    stats_manager().record_irq();
}

/// Record I/O read
pub fn stats_read(bytes: u64) {
    stats_manager().record_read(bytes);
}

/// Record I/O write
pub fn stats_write(bytes: u64) {
    stats_manager().record_write(bytes);
}

/// Record network RX
pub fn stats_net_rx(bytes: u64) {
    stats_manager().record_net_rx(bytes);
}

/// Record network TX
pub fn stats_net_tx(bytes: u64) {
    stats_manager().record_net_tx(bytes);
}

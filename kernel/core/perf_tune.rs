use crate::{pr_info};
/*
 * Nuva OS - Kernel - Performance Tuning
 * 
 * System performance optimization and tuning.
 * 
 * Copyright (C) 2026 Nuva OS Team
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

/// Performance profile
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfProfile {
    /// Power saving mode
    PowerSaver = 0,
    /// Balanced mode
    Balanced = 1,
    /// Performance mode
    Performance = 2,
    /// Maximum performance
    MaxPerformance = 3,
}

impl Default for PerfProfile {
    fn default() -> Self { Self::Balanced }
}

/// CPU governor
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuGovernor {
    /// On-demand scaling
    OnDemand = 0,
    /// Performance governor
    Performance = 1,
    /// Power-save governor
    PowerSave = 2,
    /// Conservative governor
    Conservative = 3,
    /// Userspace governor
    Userspace = 4,
    /// Scheduler governor
    SchedUtil = 5,
}

/// I/O scheduler
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoScheduler {
    /// No-op scheduler
    Noop = 0,
    /// Deadline scheduler
    Deadline = 1,
    /// CFQ scheduler
    Cfq = 2,
    /// Budget fair queueing
    Bfq = 3,
    /// Multi-queue
    MqDeadline = 4,
    /// Kyber scheduler
    Kyber = 5,
}

/// Performance configuration
#[repr(C)]
pub struct PerfConfig {
    /// Current profile
    pub profile: AtomicU32,
    /// CPU governor
    pub cpu_governor: AtomicU32,
    /// I/O scheduler
    pub io_scheduler: AtomicU32,
    /// CPU frequency scaling
    pub cpu_scaling: AtomicBool,
    /// Turbo boost
    pub turbo_boost: AtomicBool,
    /// Hyper-threading
    pub hyper_threading: AtomicBool,
    /// C-state enabled
    pub cstate_enabled: AtomicBool,
    /// Max C-state
    pub max_cstate: AtomicU32,
    /// CPU affinity optimization
    pub cpu_affinity_opt: AtomicBool,
    /// NUMA optimization
    pub numa_opt: AtomicBool,
    /// Memory prefetch
    pub mem_prefetch: AtomicBool,
    /// Huge pages
    pub huge_pages: AtomicBool,
    /// Transparent huge pages
    pub transparent_huge: AtomicBool,
    /// Read-ahead size (KB)
    pub readahead_kb: AtomicU32,
    /// Dirty ratio (%)
    pub dirty_ratio: AtomicU32,
    /// Dirty background ratio (%)
    pub dirty_bg_ratio: AtomicU32,
    /// Swappiness
    pub swappiness: AtomicU32,
    /// VFS cache pressure
    pub vfs_cache_pressure: AtomicU32,
}

impl PerfConfig {
    pub const fn new() -> Self {
        PerfConfig {
            profile: AtomicU32::new(PerfProfile::Balanced as u32),
            cpu_governor: AtomicU32::new(CpuGovernor::OnDemand as u32),
            io_scheduler: AtomicU32::new(IoScheduler::Cfq as u32),
            cpu_scaling: AtomicBool::new(true),
            turbo_boost: AtomicBool::new(true),
            hyper_threading: AtomicBool::new(true),
            cstate_enabled: AtomicBool::new(true),
            max_cstate: AtomicU32::new(3),
            cpu_affinity_opt: AtomicBool::new(true),
            numa_opt: AtomicBool::new(true),
            mem_prefetch: AtomicBool::new(true),
            huge_pages: AtomicBool::new(false),
            transparent_huge: AtomicBool::new(true),
            readahead_kb: AtomicU32::new(128),
            dirty_ratio: AtomicU32::new(20),
            dirty_bg_ratio: AtomicU32::new(10),
            swappiness: AtomicU32::new(60),
            vfs_cache_pressure: AtomicU32::new(100),
        }
    }
    
    /// Apply performance profile
    pub fn apply_profile(&self, profile: PerfProfile) {
        self.profile.store(profile as u32, Ordering::Release);
        
        match profile {
            PerfProfile::PowerSaver => {
                self.cpu_governor.store(CpuGovernor::PowerSave as u32, Ordering::Release);
                self.turbo_boost.store(false, Ordering::Release);
                self.max_cstate.store(6, Ordering::Release);
                self.readahead_kb.store(64, Ordering::Release);
                self.dirty_ratio.store(10, Ordering::Release);
            }
            PerfProfile::Balanced => {
                self.cpu_governor.store(CpuGovernor::OnDemand as u32, Ordering::Release);
                self.turbo_boost.store(true, Ordering::Release);
                self.max_cstate.store(3, Ordering::Release);
                self.readahead_kb.store(128, Ordering::Release);
                self.dirty_ratio.store(20, Ordering::Release);
            }
            PerfProfile::Performance => {
                self.cpu_governor.store(CpuGovernor::Performance as u32, Ordering::Release);
                self.turbo_boost.store(true, Ordering::Release);
                self.max_cstate.store(1, Ordering::Release);
                self.readahead_kb.store(256, Ordering::Release);
                self.dirty_ratio.store(30, Ordering::Release);
            }
            PerfProfile::MaxPerformance => {
                self.cpu_governor.store(CpuGovernor::Performance as u32, Ordering::Release);
                self.turbo_boost.store(true, Ordering::Release);
                self.cstate_enabled.store(false, Ordering::Release);
                self.max_cstate.store(0, Ordering::Release);
                self.readahead_kb.store(512, Ordering::Release);
                self.dirty_ratio.store(40, Ordering::Release);
            }
        }
    }
}

/// Performance statistics
#[repr(C)]
pub struct PerfStats {
    /// CPU utilization (%)
    pub cpu_util: AtomicU32,
    /// Memory utilization (%)
    pub mem_util: AtomicU32,
    /// I/O utilization (%)
    pub io_util: AtomicU32,
    /// Context switches/sec
    pub ctx_switches: AtomicU64,
    /// Interrupts/sec
    pub interrupts: AtomicU64,
    /// Cache hits
    pub cache_hits: AtomicU64,
    /// Cache misses
    pub cache_misses: AtomicU64,
    /// Page faults
    pub page_faults: AtomicU64,
    /// Major page faults
    pub major_faults: AtomicU64,
    /// CPU migrations
    pub cpu_migrations: AtomicU64,
    /// Alignment faults
    pub align_faults: AtomicU64,
    /// EMULATION faults
    pub emul_faults: AtomicU64,
}

impl PerfStats {
    pub const fn new() -> Self {
        PerfStats {
            cpu_util: AtomicU32::new(0),
            mem_util: AtomicU32::new(0),
            io_util: AtomicU32::new(0),
            ctx_switches: AtomicU64::new(0),
            interrupts: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            page_faults: AtomicU64::new(0),
            major_faults: AtomicU64::new(0),
            cpu_migrations: AtomicU64::new(0),
            align_faults: AtomicU64::new(0),
            emul_faults: AtomicU64::new(0),
        }
    }
    
    /// Get cache hit ratio
    pub fn cache_hit_ratio(&self) -> u32 {
        let hits = self.cache_hits.load(Ordering::Acquire);
        let misses = self.cache_misses.load(Ordering::Acquire);
        let total = hits + misses;
        if total == 0 { 0 } else { ((hits * 100) / total) as u32 }
    }
}

/// Performance tuner
pub struct PerfTuner {
    /// Configuration
    pub config: PerfConfig,
    /// Statistics
    pub stats: PerfStats,
    /// Auto-tune enabled
    auto_tune: AtomicBool,
    /// Tune interval (ms)
    tune_interval: AtomicU32,
}

impl PerfTuner {
    pub const fn new() -> Self {
        PerfTuner {
            config: PerfConfig::new(),
            stats: PerfStats::new(),
            auto_tune: AtomicBool::new(true),
            tune_interval: AtomicU32::new(1000),
        }
    }
    
    /// Initialize performance tuner
    pub fn init(&mut self) {
        log_info!("Performance tuner initialized");
        
        // Apply default profile
        self.config.apply_profile(PerfProfile::Balanced);
        
        // Start auto-tune
        if self.auto_tune.load(Ordering::Acquire) {
            self.start_auto_tune();
        }
    }
    
    /// Start auto-tune
    fn start_auto_tune(&self) {
        log_info!("Starting auto-tune...");
        extern "C" fn perf_tune_thread_entry() {
            let tuner = get_perf_tuner();
            let mut cycle_count: u64 = 0;
            loop {
                tuner.auto_tune_cycle();
                cycle_count += 1;
                tuner.stats.cpu_util.store(
                    crate::kernel::sched::get_scheduler().nr_running.load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );
                if cycle_count % 100 == 0 {
                    let pgo = crate::kernel::perf::pgo::get_pgo_profile();
                    let opt_count = pgo.apply_feedback();
                    if opt_count > 0 {
                        crate::log_debug!("perf_tune: PGO applied {} optimizations at cycle {}", opt_count, cycle_count);
                    }
                }
                crate::kernel::sched::schedule();
            }
        }
        fn _perf_tune_entry() { perf_tune_thread_entry(); }
        // Spawn a kernel thread that periodically calls auto_tune_cycle()
        crate::kernel::process::kernel_thread_create(
            "perf_auto_tune",
            _perf_tune_entry,
            0, // default priority
        );
    }
    
    /// Run auto-tune cycle
    pub fn auto_tune_cycle(&self) {
        let cpu_util = self.stats.cpu_util.load(Ordering::Acquire);
        let mem_util = self.stats.mem_util.load(Ordering::Acquire);
        let cache_ratio = self.stats.cache_hit_ratio();
        
        // Adjust based on utilization
        if cpu_util > 80 {
            // High CPU usage - switch to performance mode
            let current = self.config.profile.load(Ordering::Acquire);
            if current != PerfProfile::Performance as u32 && current != PerfProfile::MaxPerformance as u32 {
                self.config.apply_profile(PerfProfile::Performance);
            }
        } else if cpu_util < 20 && mem_util < 50 {
            // Low usage - can use power saver
            let current = self.config.profile.load(Ordering::Acquire);
            if current == PerfProfile::Performance as u32 {
                self.config.apply_profile(PerfProfile::Balanced);
            }
        }
        
        // Adjust cache settings based on hit ratio
        if cache_ratio < 70 {
            // Poor cache performance - increase read-ahead
            let ra = self.config.readahead_kb.load(Ordering::Acquire);
            if ra < 512 {
                self.config.readahead_kb.store(ra * 2, Ordering::Release);
            }
        }
    }
    
    /// Set performance profile
    pub fn set_profile(&self, profile: PerfProfile) {
        self.config.apply_profile(profile);
        log_info!("Performance profile set to {:?}", profile);
    }
    
    /// Get current profile
    pub fn get_profile(&self) -> PerfProfile {
        match self.config.profile.load(Ordering::Acquire) {
            0 => PerfProfile::PowerSaver,
            1 => PerfProfile::Balanced,
            2 => PerfProfile::Performance,
            3 => PerfProfile::MaxPerformance,
            _ => PerfProfile::Balanced,
        }
    }
    
    /// Enable/disable turbo boost
    pub fn set_turbo_boost(&self, enable: bool) {
        self.config.turbo_boost.store(enable, Ordering::Release);
        // Write to IA32_MISC_ENABLE MSR (bit 38) on x86_64 to enable/disable turbo
        crate::hal::cpu::write_msr(0x1A0, if enable { 0 } else { 1 << 38 });
    }
    
    /// Set CPU governor
    pub fn set_cpu_governor(&self, governor: CpuGovernor) {
        self.config.cpu_governor.store(governor as u32, Ordering::Release);
        // Apply the governor policy to the CPUFreq subsystem
        crate::hal::cpu::dvfs::set_governor(governor as u32);
    }
    
    /// Set I/O scheduler
    pub fn set_io_scheduler(&self, scheduler: IoScheduler) {
        self.config.io_scheduler.store(scheduler as u32, Ordering::Release);
        // Apply the I/O scheduler to all block devices
        crate::kernel::driver::block::set_io_scheduler(scheduler as u32);
    }
    
    /// Optimize for workload
    pub fn optimize_for_workload(&self, workload: WorkloadType) {
        match workload {
            WorkloadType::Desktop => {
                self.config.apply_profile(PerfProfile::Balanced);
                self.config.swappiness.store(60, Ordering::Release);
            }
            WorkloadType::Server => {
                self.config.apply_profile(PerfProfile::Performance);
                self.config.swappiness.store(10, Ordering::Release);
                self.config.huge_pages.store(true, Ordering::Release);
            }
            WorkloadType::Gaming => {
                self.config.apply_profile(PerfProfile::MaxPerformance);
                self.config.swappiness.store(10, Ordering::Release);
            }
            WorkloadType::Battery => {
                self.config.apply_profile(PerfProfile::PowerSaver);
                self.config.swappiness.store(100, Ordering::Release);
            }
            WorkloadType::LatencySensitive => {
                self.config.apply_profile(PerfProfile::MaxPerformance);
                self.config.cstate_enabled.store(false, Ordering::Release);
            }
            WorkloadType::Throughput => {
                self.config.apply_profile(PerfProfile::Performance);
                self.config.readahead_kb.store(1024, Ordering::Release);
            }
        }
    }
    pub fn auto_tune_thread_fn(&self) {}
}

/// Workload type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadType {
    Desktop = 0,
    Server = 1,
    Gaming = 2,
    Battery = 3,
    LatencySensitive = 4,
    Throughput = 5,
}

impl Default for PerfTuner {
    fn default() -> Self { Self::new() }
}

/// Global performance tuner
static PERF_TUNER: core::sync::OnceLock<PerfTuner> = core::sync::OnceLock::new();

/// Get performance tuner
pub fn perf_tuner() -> &'static PerfTuner {
    PERF_TUNER.get_or_init(PerfTuner::new)
}

/// Initialize performance tuning
pub fn init_perf_tune() {
    let tuner = get_perf_tuner();
    tuner.init();
}

/// Quick performance check
pub fn perf_check() -> u32 {
    let tuner = get_perf_tuner();
    let cpu = tuner.stats.cpu_util.load(Ordering::Acquire);
    let mem = tuner.stats.mem_util.load(Ordering::Acquire);
    let io = tuner.stats.io_util.load(Ordering::Acquire);
    
    // Return overall performance score (0-100)
    (cpu + mem + io) / 3
}

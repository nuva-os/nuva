/*
 * Nuva OS - Kernel - OOM Killer
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

//! Out-of-Memory (OOM) Killer implementation.
/*!*/
//! When the system runs out of memory, the OOM killer selects the best
//! candidate process to terminate in order to free memory. The selection
//! is based on oom_score which considers:
//! - Process RSS (resident set size) relative to total memory
//! - Swap usage relative to total swap
//! - Process nice value (lower nice = higher priority = less likely killed)
//! - Admin override via oom_score_adj
//! - Whether the process is privileged (root processes are protected)

use crate::{pr_err, pr_info, pr_warn};
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// Maximum OOM score.
pub const OOM_SCORE_MAX: i16 = 1000;

/// Minimum OOM score adjustment.
pub const OOM_SCORE_ADJ_MIN: i16 = -1000;

/// Maximum OOM score adjustment.
pub const OOM_SCORE_ADJ_MAX: i16 = 1000;

/// OOM kill constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OomConstraint {
    /// No constraint.
    None,
    /// Constrain to processes in the same memory cgroup.
    Cgroup,
    /// Constrain to processes in the same memory policy.
    MemoryPolicy,
}

/// OOM kill result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OomResult {
    /// A process was selected and killed.
    Killed(u32),
    /// No process could be found to kill.
    NoCandidate,
    /// OOM killer is disabled.
    Disabled,
    /// Not enough memory to proceed with kill.
    Critical,
}

/// Per-process OOM information.
pub struct OomProcessInfo {
    /// Process ID.
    pub pid: u32,
    /// Process RSS in pages.
    pub rss_pages: u64,
    /// Process swap usage in pages.
    pub swap_pages: u64,
    /// Total memory in pages.
    pub total_pages: u64,
    /// Total swap in pages.
    pub total_swap_pages: u64,
    /// Process nice value (-20 to +19).
    pub nice: i32,
    /// Whether the process is privileged (uid 0).
    pub is_privileged: bool,
    /// Admin OOM score adjustment (-1000 to +1000).
    pub oom_score_adj: i16,
    /// Whether the process is an OOM victim (already selected).
    pub is_oom_victim: bool,
    /// Number of child processes.
    pub child_count: u32,
    /// Process uptime in seconds.
    pub uptime_secs: u64,
}

impl OomProcessInfo {
    /// Create a new OOM process info.
    pub const fn new() -> Self {
        OomProcessInfo {
            pid: 0,
            rss_pages: 0,
            swap_pages: 0,
            total_pages: 0,
            total_swap_pages: 0,
            nice: 0,
            is_privileged: false,
            oom_score_adj: 0,
            is_oom_victim: false,
            child_count: 0,
            uptime_secs: 0,
        }
    }

    /// Calculate the OOM score for this process.
    /// Score formula (enhanced):
    ///   rss_score = (rss / total_memory) * 1000
    ///   swap_score = (swap / total_swap) * 1000
    ///   prio_score = nice * 10 (lower nice = -20 -> -200, RT = -500)
    ///   runtime_score = -min(uptime_secs, 200) (long-running protected)
    ///   oom_score = clamp(rss_score + swap_score + prio_score
    ///                     + runtime_score + oom_score_adj, -1000, 1000)
    /// Higher score = more likely to be killed.
    /// RT tasks (nice < 0 beyond normal range) are strongly protected.
    pub fn calculate_oom_score(&self) -> i16 {
        let mut score: i64 = 0;

        // RSS contribution (0-1000)
        if self.total_pages > 0 {
            let rss_score = (self.rss_pages * 1000) / self.total_pages;
            score += rss_score as i64;
        }

        // Swap contribution (0-1000)
        if self.total_swap_pages > 0 {
            let swap_score = (self.swap_pages * 1000) / self.total_swap_pages;
            score += swap_score as i64;
        }

        // Priority score: RT tasks get -500, normal tasks get nice*10
        let prio_score: i64 = if self.nice < -20 {
            // Real-time task: strong protection
            -500
        } else {
            (self.nice as i64) * 10
        };
        score += prio_score;

        // Runtime score: long-running processes are more valuable
        let runtime_score = -core::cmp::min(self.uptime_secs as i64, 200);
        score += runtime_score;

        // Apply admin adjustment
        score += self.oom_score_adj as i64;

        // Clamp to valid range [-1000, 1000]
        if score < -1000 {
            score = -1000;
        }
        if score > 1000 {
            score = 1000;
        }

        score as i16
    }

    /// Check if this process should be protected from OOM kill.
    pub fn is_protected(&self) -> bool {
        // Processes with oom_score_adj of -1000 are never killed
        self.oom_score_adj == OOM_SCORE_ADJ_MIN
    }
}

/// OOM Killer state.
pub struct OomKiller {
    /// Minimum free pages threshold to trigger OOM.
    threshold_pages: AtomicU64,
    /// Whether OOM killer is enabled.
    enabled: AtomicBool,
    /// Number of OOM kills performed.
    kill_count: AtomicU64,
    /// Last killed PID.
    last_killed_pid: AtomicU32,
    /// OOM constraint.
    constraint: OomConstraint,
    /// Panic on OOM (instead of killing).
    panic_on_oom: AtomicBool,
}

impl OomKiller {
    /// Create a new OOM killer.
    pub const fn new() -> Self {
        OomKiller {
            threshold_pages: AtomicU64::new(256), // 1MB with 4KB pages
            enabled: AtomicBool::new(true),
            kill_count: AtomicU64::new(0),
            last_killed_pid: AtomicU32::new(0),
            constraint: OomConstraint::None,
            panic_on_oom: AtomicBool::new(false),
        }
    }

    /// Initialize the OOM killer.
    pub fn init(&self) {
        log_info!(
            "OOM: Killer initialized (threshold: {} pages)",
            self.threshold_pages.load(Ordering::Relaxed)
        );
    }

    /// Check if OOM condition is triggered.
    /// Returns true if free memory is below the OOM threshold.
    pub fn check_oom(&self, free_pages: u64) -> bool {
        if !self.enabled.load(Ordering::Relaxed) {
            return false;
        }
        free_pages < self.threshold_pages.load(Ordering::Relaxed)
    }

    /// Select the best process to kill.
    /// The process with the highest OOM score is selected.
    /// Protected processes (oom_score_adj = -1000) are skipped.
    /// Privileged processes are given a score penalty.
    pub fn select_victim(&self, processes: &[OomProcessInfo]) -> Option<u32> {
        let mut best_pid: u32 = 0;
        let mut best_score: i16 = -1;

        for proc in processes {
            // Skip protected processes
            if proc.is_protected() {
                continue;
            }

            // Skip already-oom-victim processes
            if proc.is_oom_victim {
                continue;
            }

            // Skip PID 0 (kernel idle) and PID 1 (init)
            if proc.pid <= 1 {
                continue;
            }

            let mut score = proc.calculate_oom_score();

            // Penalize privileged processes (reduce score by 30%)
            if proc.is_privileged {
                score = (score as i64 * 7 / 10) as i16;
            }

            // Favor killing processes with many children (orphan cleanup)
            if proc.child_count > 10 {
                score = (score as i64 + 50).min(OOM_SCORE_MAX as i64) as i16;
            }

            // Favor killing short-lived processes
            if proc.uptime_secs < 60 {
                score = (score as i64 + 30).min(OOM_SCORE_MAX as i64) as i16;
            }

            if score > best_score {
                best_score = score;
                best_pid = proc.pid;
            }
        }

        if best_pid > 0 {
            log_info!(
                "OOM: Selected PID {} for kill (score={})",
                best_pid,
                best_score
            );
            Some(best_pid)
        } else {
            log_warn!("OOM: No suitable victim found");
            None
        }
    }

    /// Kill the selected process.
    /// Sends SIGKILL (signal 9) to the victim process.
    pub fn kill(&self, pid: u32) -> OomResult {
        if pid == 0 {
            return OomResult::NoCandidate;
        }

        log_warn!("OOM: Killing process {} to free memory", pid);

        // Send SIGKILL to the victim process
        let handler = crate::kernel::process::signal::get_signal_handler();
        let info = crate::kernel::process::signal::SigInfo {
            signo: crate::kernel::process::signal::signal::SIGKILL as i32,
            errno: 0,
            code: 0, // SI_KERNEL
            pid: 0,
            uid: 0,
            value: crate::kernel::process::signal::SigVal { sival_int: 0 },
            addr: 0,
        };
        let _ = handler.send_signal(pid, crate::kernel::process::signal::signal::SIGKILL, &info);

        self.kill_count.fetch_add(1, Ordering::Relaxed);
        self.last_killed_pid.store(pid, Ordering::Relaxed);

        OomResult::Killed(pid)
    }

    /// Handle an OOM situation.
    /// Called when the page allocator fails to allocate memory.
    /// Tries to reclaim memory, then invokes the OOM killer if necessary.
    pub fn handle_out_of_memory(&self, free_pages: u64, processes: &[OomProcessInfo]) -> OomResult {
        if !self.enabled.load(Ordering::Relaxed) {
            return OomResult::Disabled;
        }

        if self.panic_on_oom.load(Ordering::Relaxed) {
            log_error!("OOM: panic_on_oom is set, system will panic!");
            // In real implementation: panic!("Out of memory");
            return OomResult::Critical;
        }

        log_warn!(
            "OOM: Out of memory! Free pages: {}, threshold: {}",
            free_pages,
            self.threshold_pages.load(Ordering::Relaxed)
        );

        // Step 1: Try to reclaim memory (drop caches, compact, shrink slabs)
        // In real implementation: call memory reclaim

        // Step 2: Select victim and kill
        match self.select_victim(processes) {
            Some(pid) => self.kill(pid),
            None => {
                log_error!("OOM: No process to kill! System may become unstable.");
                OomResult::NoCandidate
            }
        }
    }

    /// Set the OOM threshold (minimum free pages before triggering).
    pub fn set_threshold(&self, pages: u64) {
        self.threshold_pages.store(pages, Ordering::Release);
    }

    /// Enable or disable the OOM killer.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
    }

    /// Set panic_on_oom flag.
    pub fn set_panic_on_oom(&self, panic: bool) {
        self.panic_on_oom.store(panic, Ordering::Release);
    }

    /// Get the number of OOM kills.
    pub fn get_kill_count(&self) -> u64 {
        self.kill_count.load(Ordering::Relaxed)
    }
}

/// Global OOM killer.
static OOM_KILLER: OomKiller = OomKiller::new();

/// Get the global OOM killer.
pub fn get_oom_killer() -> &'static OomKiller {
    &OOM_KILLER
}

/// Initialize the OOM killer.
pub fn init_oom() {
    get_oom_killer().init();
}

/// Check if OOM condition exists.
pub fn check_oom(free_pages: u64) -> bool {
    get_oom_killer().check_oom(free_pages)
}

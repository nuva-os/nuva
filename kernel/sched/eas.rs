/*
 * Nuva OS - Kernel - Energy Aware Scheduling (EAS)
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

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

/// Maximum number of performance domains
pub const MAX_NR_PERF_DOMAINS: usize = 4;

/// Maximum number of CPUs per performance domain
pub const MAX_CPUS_PER_DOMAIN: usize = 8;

/// Performance state (P-state)
/// Represents a CPU frequency/voltage operating point
#[derive(Clone)]
pub struct PerfState {
    /// Frequency in kHz
    pub frequency: u32,
    
    /// Voltage in microvolts
    pub voltage: u32,
    
    /// Power consumption in microwatts
    pub power: u32,
    
    /// Cost coefficient for this state
    pub cost: u64,
}

impl PerfState {
    pub const fn new(frequency: u32, voltage: u32, power: u32) -> Self {
        // Cost = power / frequency (simplified)
        let cost = if frequency > 0 {
            (power as u64 * 1000000) / (frequency as u64)
        } else {
            0
        };
        
        PerfState {
            frequency,
            voltage,
            power,
            cost,
        }
    }
}

/// Performance domain
/// A group of CPUs that share the same frequency/voltage domain
pub struct PerfDomain {
    /// CPUs in this domain
    pub cpus: u64,  // CPU mask
    
    /// Number of CPUs
    pub nr_cpus: u32,
    
    /// Performance states (sorted by frequency)
    pub states: [Option<PerfState>; 16],
    
    /// Number of performance states
    pub nr_states: u32,
    
    /// Current performance state index
    pub current_state: AtomicU32,
    
    /// Domain name
    pub name: [u8; 16],
}

impl Clone for PerfDomain {
    fn clone(&self) -> Self {
        Self {
            cpus: self.cpus.clone(),
            nr_cpus: self.nr_cpus.clone(),
            states: self.states.clone(),
            nr_states: self.nr_states.clone(),
            current_state: AtomicU32::new(self.current_state.load(core::sync::atomic::Ordering::Relaxed)),
            name: self.name.clone(),
        }
    }
}

impl PerfDomain {
    pub const fn new() -> Self {
        PerfDomain {
            cpus: 0,
            nr_cpus: 0,
            states: [const { None }; 16],
            nr_states: 0,
            current_state: AtomicU32::new(0),
            name: [0; 16],
        }
    }
    
    /// Get current frequency
    pub fn get_current_frequency(&self) -> u32 {
        let idx = self.current_state.load(Ordering::Acquire) as usize;
        if idx < self.nr_states as usize {
            if let Some(ref state) = self.states[idx] {
                return state.frequency;
            }
        }
        0
    }
    
    /// Get current power consumption
    pub fn get_current_power(&self) -> u32 {
        let idx = self.current_state.load(Ordering::Acquire) as usize;
        if idx < self.nr_states as usize {
            if let Some(ref state) = self.states[idx] {
                return state.power;
            }
        }
        0
    }
    
    /// Set performance state
    pub fn set_state(&self, idx: u32) -> bool {
        if idx >= self.nr_states {
            return false;
        }
        self.current_state.store(idx, Ordering::Release);
        true
    }
    
    /// Find performance state for a given frequency
    pub fn find_state_for_frequency(&self, frequency: u32) -> u32 {
        for i in 0..self.nr_states as usize {
            if let Some(ref state) = self.states[i] {
                if state.frequency >= frequency {
                    return i as u32;
                }
            }
        }
        // Return highest state if not found
        if self.nr_states > 0 {
            self.nr_states - 1
        } else {
            0
        }
    }
}

/// Energy model
/// Contains all performance domains and their energy characteristics
pub struct EnergyModel {
    /// Performance domains
    pub domains: [Option<PerfDomain>; MAX_NR_PERF_DOMAINS],
    
    /// Number of performance domains
    pub nr_domains: u32,
    
    /// Total system capacity
    pub total_capacity: AtomicU64,
    
    /// EAS enabled flag
    pub enabled: AtomicBool,
    
    /// Overutilization threshold
    pub overutilization_threshold: AtomicU32,
}

impl EnergyModel {
    pub const fn new() -> Self {
        EnergyModel {
            domains: [const { None }; MAX_NR_PERF_DOMAINS],
            nr_domains: 0,
            total_capacity: AtomicU64::new(0),
            enabled: AtomicBool::new(false),
            overutilization_threshold: AtomicU32::new(80),  // 80%
        }
    }
    
    /// Initialize energy model
    pub fn init(&mut self) {
        // Create default performance domains
        // This would normally be populated from device tree or ACPI
        
        // Example: Create a big.LITTLE style configuration
        // Domain 0: LITTLE cores (4 CPUs, low frequency)
        let mut little_domain = PerfDomain::new();
        little_domain.cpus = 0x0F;  // CPUs 0-3
        little_domain.nr_cpus = 4;
        little_domain.name = *b"LITTLE\0\0\0\0\0\0\0\0\0\0";
        
        // Add performance states for LITTLE cores
        little_domain.states[0] = Some(PerfState::new(300000, 800000, 100000));   // 300MHz
        little_domain.states[1] = Some(PerfState::new(600000, 850000, 200000));   // 600MHz
        little_domain.states[2] = Some(PerfState::new(900000, 900000, 350000));   // 900MHz
        little_domain.states[3] = Some(PerfState::new(1200000, 950000, 550000));  // 1.2GHz
        little_domain.states[4] = Some(PerfState::new(1500000, 1000000, 800000)); // 1.5GHz
        little_domain.nr_states = 5;
        
        self.domains[0] = Some(little_domain);
        
        // Domain 1: big cores (4 CPUs, high frequency)
        let mut big_domain = PerfDomain::new();
        big_domain.cpus = 0xF0;  // CPUs 4-7
        big_domain.nr_cpus = 4;
        big_domain.name = *b"big\0\0\0\0\0\0\0\0\0\0\0\0\0";
        
        // Add performance states for big cores
        big_domain.states[0] = Some(PerfState::new(500000, 900000, 300000));    // 500MHz
        big_domain.states[1] = Some(PerfState::new(1000000, 950000, 600000));   // 1GHz
        big_domain.states[2] = Some(PerfState::new(1500000, 1000000, 1000000)); // 1.5GHz
        big_domain.states[3] = Some(PerfState::new(2000000, 1050000, 1500000)); // 2GHz
        big_domain.states[4] = Some(PerfState::new(2500000, 1100000, 2200000)); // 2.5GHz
        big_domain.nr_states = 5;
        
        self.domains[1] = Some(big_domain);
        
        self.nr_domains = 2;
        self.total_capacity.store(1024 * 8, Ordering::Release);  // 8 CPUs * 1024 capacity
        self.enabled.store(true, Ordering::Release);
    }
    
    /// Get performance domain for a CPU
    pub fn get_domain_for_cpu(&self, cpu: usize) -> Option<&PerfDomain> {
        for i in 0..self.nr_domains as usize {
            if let Some(ref domain) = self.domains[i] {
                if (domain.cpus & (1 << cpu)) != 0 {
                    return Some(domain);
                }
            }
        }
        None
    }
    
    /// Calculate energy for a given CPU utilization
    pub fn compute_energy(&self, cpu: usize, utilization: u32) -> u64 {
        if let Some(domain) = self.get_domain_for_cpu(cpu) {
            // Find the performance state that can handle this utilization
            let freq = self.util_to_freq(utilization);
            let state_idx = domain.find_state_for_frequency(freq);
            
            if let Some(ref state) = domain.states[state_idx as usize] {
                // Energy = power * time
                // For simplicity, assume unit time
                return state.power as u64;
            }
        }
        0
    }
    
    /// Convert utilization to frequency
    fn util_to_freq(&self, utilization: u32) -> u32 {
        // Simplified: assume max frequency is 2.5GHz
        // utilization is 0-1024, map to 0-2500000kHz
        (utilization as u64 * 2500000 / 1024) as u32
    }
    
    /// Check if system is overutilized
    pub fn is_overutilized(&self, total_util: u64, total_capacity: u64) -> bool {
        if total_capacity == 0 {
            return false;
        }
        
        let util_pct = (total_util * 100) / total_capacity;
        util_pct > self.overutilization_threshold.load(Ordering::Acquire) as u64
    }
    
    /// Find the most energy-efficient CPU for a task
    pub fn find_energy_efficient_cpu(&self, task_util: u32, prev_cpu: usize) -> usize {
        if !self.enabled.load(Ordering::Acquire) {
            return prev_cpu;
        }
        
        let mut best_cpu = prev_cpu;
        let mut min_energy = u64::MAX;
        
        // Evaluate each CPU
        for cpu in 0..64 {
            // Check if CPU exists
            let mut cpu_exists = false;
            for i in 0..self.nr_domains as usize {
                if let Some(ref domain) = self.domains[i] {
                    if (domain.cpus & (1 << cpu)) != 0 {
                        cpu_exists = true;
                        break;
                    }
                }
            }
            
            if !cpu_exists {
                continue;
            }
            
            // Calculate energy for placing task on this CPU
            let energy = self.compute_energy(cpu, task_util);
            
            // Consider migration cost
            let migration_cost = if cpu == prev_cpu { 0 } else { 100000 };  // 100mW penalty
            
            let total_energy = energy + migration_cost;
            
            if total_energy < min_energy {
                min_energy = total_energy;
                best_cpu = cpu;
            }
        }
        
        best_cpu
    }
}

/// Energy-aware scheduler data
pub struct EasData {
    /// Energy model
    pub energy_model: EnergyModel,
    
    /// EAS enabled flag
    pub enabled: AtomicBool,
    
    /// Statistics
    pub stats: EasStats,
}

/// EAS statistics
pub struct EasStats {
    /// Number of EAS wakeups
    pub eas_wakeups: AtomicU64,
    
    /// Number of EAS migrations
    pub eas_migrations: AtomicU64,
    
    /// Number of fallback to CFS
    pub eas_fallbacks: AtomicU64,
    
    /// Energy saved (estimated)
    pub energy_saved: AtomicU64,
}

impl EasData {
    pub const fn new() -> Self {
        EasData {
            energy_model: EnergyModel::new(),
            enabled: AtomicBool::new(false),
            stats: EasStats {
                eas_wakeups: AtomicU64::new(0),
                eas_migrations: AtomicU64::new(0),
                eas_fallbacks: AtomicU64::new(0),
                energy_saved: AtomicU64::new(0),
            },
        }
    }
    
    /// Initialize EAS
    pub fn init(&mut self) {
        self.energy_model.init();
        self.enabled.store(true, Ordering::Release);
    }
    
    /// Select target CPU for a waking task
    pub fn select_task_rq(&mut self, task_util: u32, prev_cpu: usize, _sync: bool) -> usize {
        if !self.enabled.load(Ordering::Acquire) {
            return prev_cpu;
        }
        
        self.stats.eas_wakeups.fetch_add(1, Ordering::Relaxed);
        
        // Use energy model to find best CPU
        let best_cpu = self.energy_model.find_energy_efficient_cpu(task_util, prev_cpu);
        
        if best_cpu != prev_cpu {
            self.stats.eas_migrations.fetch_add(1, Ordering::Relaxed);
        }
        
        best_cpu
    }
}

/// Global EAS data
static EAS_DATA: core::sync::OnceLock<EasData> = core::sync::OnceLock::new();

/// Get EAS data
pub fn eas_data() -> &'static EasData {
    EAS_DATA.get_or_init(EasData::new)
}

/// Initialize EAS
pub fn init_eas() {
    // SAFETY: init is called once during boot, before multi-core scheduling starts
    let eas: &mut EasData = unsafe {
        &mut *core::ptr::from_ref(EAS_DATA.get_or_init(EasData::new)).cast_mut()
    };
    eas.init();
}

/// Check if EAS is enabled
pub fn is_eas_enabled() -> bool {
    eas_data().enabled.load(Ordering::Acquire)
}

/// Select target CPU for a task (EAS entry point)
pub fn eas_select_task_rq(task_util: u32, prev_cpu: usize, sync: bool) -> usize {
    // SAFETY: select_task_rq uses atomic operations internally;
    // mutable aliasing during boot is safe because only one CPU runs init
    let eas: &mut EasData = unsafe {
        &mut *core::ptr::from_ref(EAS_DATA.get_or_init(EasData::new)).cast_mut()
    };
    eas.select_task_rq(task_util, prev_cpu, sync)
}

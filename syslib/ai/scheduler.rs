/*
 * Nuva OS - Syslib - Ai - Scheduler
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
 * Intelligent Scheduler - AI-Driven Task Scheduling
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module implements intelligent task scheduling using
 * AI/ML to predict task behavior and optimize scheduling decisions.
 */

use core::fmt;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use spin::RwLock;

/// Intelligent scheduler
/// Uses AI to optimize task scheduling by:
/// - Predicting task behavior
/// - Adjusting task priorities
/// - Optimizing CPU affinity
/// - Balancing load across cores
pub struct IntelligentScheduler {
    /// Behavior prediction model
    behavior_model: Arc<RwLock<Box<dyn BehaviorModel>>>,

    /// Scheduling policy
    policy: RwLock<SchedulingPolicy>,

    /// Task history
    history: RwLock<TaskHistory>,

    /// Scheduler statistics
    stats: RwLock<SchedulerStats>,

    /// Enable prediction
    enable_prediction: bool,

    /// Number of CPUs
    num_cpus: usize,
}

impl IntelligentScheduler {
    /// Create new intelligent scheduler
    /// @param config: Scheduler configuration
    pub fn new(config: SchedulerConfig) -> Self {
        let behavior_model = Arc::new(RwLock::new(config.behavior_model));
        let num_cpus = config.num_cpus;
        let enable_prediction = config.enable_prediction;
        Self {
            behavior_model,
            policy: RwLock::new(SchedulingPolicy::default()),
            history: RwLock::new(TaskHistory::new()),
            stats: RwLock::new(SchedulerStats::default()),
            enable_prediction,
            num_cpus,
        }
    }

    /// Predict task behavior
    /// @param task: Task information
    /// @return: Predicted behavior
    pub fn predict_behavior(&self, task: &TaskInfo) -> Result<TaskPrediction, SchedulerError> {
        let model = self.behavior_model.read();
        model.predict(task)
    }

    /// Schedule task
    /// @param task: Task to schedule
    /// @return: Scheduling decision
    pub fn schedule(&self, task: &mut TaskInfo) -> Result<SchedulingDecision, SchedulerError> {
        // Predict task behavior
        let prediction = self.predict_behavior(task)?;

        // Get current policy
        let policy = self.policy.read();

        // Make scheduling decision
        let decision = self.make_decision(task, &prediction, &policy)?;

        // Apply decision
        self.apply_decision(task, &decision)?;

        // Update statistics
        let mut stats = self.stats.write();
        stats.total_scheduled += 1;

        Ok(decision)
    }

    /// Adjust task priority
    /// @param task: Task to adjust
    /// @param prediction: Task prediction
    pub fn adjust_priority(&self, task: &mut TaskInfo, prediction: &TaskPrediction) {
        // Base priority adjustment on predicted behavior
        let priority_adjustment = match prediction.task_type {
            PredictedTaskType::CpuIntensive => -5,  // Lower priority
            PredictedTaskType::IoIntensive => 5,    // Higher priority
            PredictedTaskType::Mixed => 0,
            PredictedTaskType::Interactive => 10,   // Highest priority
            PredictedTaskType::Background => -10,   // Lowest priority
        };

        // Apply adjustment
        task.priority = (task.priority as i32 + priority_adjustment).max(0).min(100) as u8;
    }

    /// Optimize CPU affinity
    /// @param task: Task to optimize
    /// @param prediction: Task prediction
    /// @return: Optimal CPU mask
    pub fn optimize_affinity(&self, task: &TaskInfo, prediction: &TaskPrediction) -> Result<CpuMask, SchedulerError> {
        // Get number of CPUs
        let num_cpus = self.num_cpus;

        // Determine optimal CPU set based on prediction
        let affinity = match prediction.task_type {
            PredictedTaskType::CpuIntensive => {
                // Spread across all CPUs
                CpuMask::all(num_cpus)
            }
            PredictedTaskType::IoIntensive => {
                // Pin to single CPU to reduce migration
                CpuMask::single(0)
            }
            PredictedTaskType::Interactive => {
                // Use high-performance CPUs
                CpuMask::high_performance(num_cpus)
            }
            _ => {
                // Default: all CPUs
                CpuMask::all(num_cpus)
            }
        };

        Ok(affinity)
    }

    /// Balance load across CPUs
    /// @param tasks: All tasks
    /// @return: Load balancing decisions
    pub fn balance_load(&self, tasks: &mut [TaskInfo]) -> Result<Vec<LoadBalanceDecision>, SchedulerError> {
        let mut decisions = Vec::new();

        // Calculate current load per CPU
        let cpu_loads = self.calculate_cpu_loads(tasks)?;

        // Find overloaded and underloaded CPUs
        let avg_load = cpu_loads.iter().sum::<f32>() / cpu_loads.len() as f32;

        for (cpu, &load) in cpu_loads.iter().enumerate() {
            if load > avg_load * 1.2 {
                // Overloaded: migrate tasks
                let tasks_to_migrate = self.find_migratable_tasks(tasks, cpu)?;
                for task in tasks_to_migrate {
                    let target_cpu = self.find_underloaded_cpu(&cpu_loads)?;
                    decisions.push(LoadBalanceDecision {
                        task_id: task.id,
                        from_cpu: cpu,
                        to_cpu: target_cpu,
                    });
                }
            }
        }

        Ok(decisions)
    }

    /// Update scheduling policy
    /// @param policy: New policy
    pub fn update_policy(&self, policy: SchedulingPolicy) {
        let mut current = self.policy.write();
        *current = policy;
    }

    /// Get scheduler statistics
    pub fn get_stats(&self) -> SchedulerStats {
        let stats = self.stats.read();
        stats.clone()
    }

    // Private helper methods

    /// Make scheduling decision
    fn make_decision(
        &self,
        task: &TaskInfo,
        prediction: &TaskPrediction,
        policy: &SchedulingPolicy,
    ) -> Result<SchedulingDecision, SchedulerError> {
        // Determine priority
        let priority = self.calculate_priority(task, prediction, policy);

        // Determine CPU affinity
        let affinity = self.optimize_affinity(task, prediction)?;

        // Determine time slice
        let time_slice = self.calculate_time_slice(task, prediction, policy);

        Ok(SchedulingDecision {
            priority,
            affinity,
            time_slice,
            preemptible: prediction.preemptible,
        })
    }

    /// Apply scheduling decision
    fn apply_decision(&self, task: &mut TaskInfo, decision: &SchedulingDecision) -> Result<(), SchedulerError> {
        task.priority = decision.priority;
        task.affinity = decision.affinity.clone();
        task.time_slice = decision.time_slice;
        task.preemptible = decision.preemptible;
        Ok(())
    }

    /// Calculate task priority
    fn calculate_priority(&self, task: &TaskInfo, prediction: &TaskPrediction, policy: &SchedulingPolicy) -> u8 {
        // Base priority
        let mut priority = task.priority as i32;

        // Adjust based on prediction
        priority += prediction.priority_adjustment;

        // Apply policy
        priority = priority.max(policy.min_priority as i32).min(policy.max_priority as i32);

        priority as u8
    }

    /// Calculate time slice
    fn calculate_time_slice(&self, task: &TaskInfo, prediction: &TaskPrediction, policy: &SchedulingPolicy) -> u64 {
        // Base time slice
        let mut time_slice = policy.base_time_slice;

        // Adjust based on prediction
        time_slice = match prediction.task_type {
            PredictedTaskType::CpuIntensive => time_slice * 2,
            PredictedTaskType::IoIntensive => time_slice / 2,
            PredictedTaskType::Interactive => time_slice / 4,
            _ => time_slice,
        };

        time_slice.max(policy.min_time_slice).min(policy.max_time_slice)
    }

    /// Calculate CPU loads
    fn calculate_cpu_loads(&self, tasks: &[TaskInfo]) -> Result<Vec<f32>, SchedulerError> {
        let mut loads = vec![0.0f32; self.num_cpus];

        for task in tasks {
            for cpu in task.affinity.cpus() {
                if cpu < loads.len() {
                    loads[cpu] += task.cpu_usage;
                }
            }
        }

        Ok(loads)
    }

    /// Find migratable tasks
    fn find_migratable_tasks(&self, tasks: &[TaskInfo], cpu: usize) -> Result<Vec<TaskInfo>, SchedulerError> {
        Ok(tasks
            .iter()
            .filter(|t| t.affinity.has_cpu(cpu) && t.migratable)
            .cloned()
            .collect())
    }

    /// Find underloaded CPU
    fn find_underloaded_cpu(&self, loads: &[f32]) -> Result<usize, SchedulerError> {
        loads
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .ok_or(SchedulerError::NoCpuAvailable)
    }
}

/// Task information
#[derive(Debug, Clone)]
pub struct TaskInfo {
    /// Task ID
    pub id: u64,

    /// Task name
    pub name: String,

    /// Current priority
    pub priority: u8,

    /// CPU affinity
    pub affinity: CpuMask,

    /// Time slice (us)
    pub time_slice: u64,

    /// CPU usage (0-100%)
    pub cpu_usage: f32,

    /// Memory usage (bytes)
    pub memory_usage: u64,

    /// Is preemptible
    pub preemptible: bool,

    /// Is migratable
    pub migratable: bool,
}

/// Task prediction
#[derive(Debug, Clone)]
pub struct TaskPrediction {
    /// Predicted task type
    pub task_type: PredictedTaskType,

    /// Predicted CPU usage
    pub predicted_cpu_usage: f32,

    /// Predicted memory usage
    pub predicted_memory_usage: u64,

    /// Predicted duration (us)
    pub predicted_duration: u64,

    /// Priority adjustment
    pub priority_adjustment: i32,

    /// Is preemptible
    pub preemptible: bool,
}

/// Predicted task type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictedTaskType {
    /// CPU-intensive task
    CpuIntensive,

    /// I/O-intensive task
    IoIntensive,

    /// Mixed task
    Mixed,

    /// Interactive task
    Interactive,

    /// Background task
    Background,
}

/// Scheduling decision
#[derive(Debug, Clone)]
pub struct SchedulingDecision {
    /// Assigned priority
    pub priority: u8,

    /// CPU affinity
    pub affinity: CpuMask,

    /// Time slice (us)
    pub time_slice: u64,

    /// Is preemptible
    pub preemptible: bool,
}

/// Load balance decision
#[derive(Debug, Clone)]
pub struct LoadBalanceDecision {
    /// Task ID
    pub task_id: u64,

    /// Source CPU
    pub from_cpu: usize,

    /// Target CPU
    pub to_cpu: usize,
}

/// CPU mask
#[derive(Debug, Clone)]
pub struct CpuMask {
    /// Bitmask of CPUs
    mask: Vec<u64>,
}

impl CpuMask {
    pub fn all(num_cpus: usize) -> Self {
        let num_words = (num_cpus + 63) / 64;
        Self {
            mask: vec![u64::MAX; num_words],
        }
    }

    pub fn single(cpu: usize) -> Self {
        let num_words = (cpu + 64) / 64;
        let mut mask = vec![0u64; num_words];
        mask[cpu / 64] = 1 << (cpu % 64);
        Self { mask }
    }

    pub fn high_performance(num_cpus: usize) -> Self {
        // TODO: Identify high-performance CPUs
        Self::all(num_cpus)
    }

    pub fn has_cpu(&self, cpu: usize) -> bool {
        let word = cpu / 64;
        let bit = cpu % 64;
        word < self.mask.len() && (self.mask[word] & (1 << bit)) != 0
    }

    pub fn cpus(&self) -> Vec<usize> {
        let mut cpus = Vec::new();
        for (word, &mask) in self.mask.iter().enumerate() {
            for bit in 0..64 {
                if (mask & (1 << bit)) != 0 {
                    cpus.push(word * 64 + bit);
                }
            }
        }
        cpus
    }
}

/// Scheduling policy
#[derive(Debug, Clone)]
pub struct SchedulingPolicy {
    /// Minimum priority
    pub min_priority: u8,

    /// Maximum priority
    pub max_priority: u8,

    /// Base time slice (us)
    pub base_time_slice: u64,

    /// Minimum time slice (us)
    pub min_time_slice: u64,

    /// Maximum time slice (us)
    pub max_time_slice: u64,

    /// Enable load balancing
    pub enable_load_balance: bool,
}

impl Default for SchedulingPolicy {
    fn default() -> Self {
        Self {
            min_priority: 0,
            max_priority: 100,
            base_time_slice: 10000, // 10ms
            min_time_slice: 1000,   // 1ms
            max_time_slice: 100000, // 100ms
            enable_load_balance: true,
        }
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    /// Total tasks scheduled
    pub total_scheduled: u64,

    /// Total migrations
    pub total_migrations: u64,

    /// Average scheduling latency (us)
    pub avg_scheduling_latency: u64,
}

/// Task history
struct TaskHistory {
    entries: Vec<TaskHistoryEntry>,
    max_entries: usize,
}

impl TaskHistory {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 1000,
        }
    }
}

/// Task history entry
struct TaskHistoryEntry {
    task_id: u64,
    timestamp: u64,
    prediction: TaskPrediction,
    decision: SchedulingDecision,
}

/// Behavior model trait
pub trait BehaviorModel: Send + Sync {
    fn predict(&self, task: &TaskInfo) -> Result<TaskPrediction, SchedulerError>;
}

/// Scheduler configuration
pub struct SchedulerConfig {
    /// Number of CPUs
    pub num_cpus: usize,

    /// Behavior prediction model
    pub behavior_model: Box<dyn BehaviorModel>,

    /// Enable prediction
    pub enable_prediction: bool,
}

/// Scheduler error
#[derive(Debug, Clone)]
pub enum SchedulerError {
    /// Model error
    ModelError(String),

    /// No CPU available
    NoCpuAvailable,

    /// Task not found
    TaskNotFound(u64),

    /// Not supported
    NotSupported,
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModelError(msg) => write!(f, "Model error: {}", msg),
            Self::NoCpuAvailable => write!(f, "No CPU available"),
            Self::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            Self::NotSupported => write!(f, "Not supported"),
        }
    }
}

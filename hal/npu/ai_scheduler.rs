/*
 * Nuva OS - HAL - AI Scheduler
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

//! AI Scheduler
/*!*/
//! AI-driven task scheduling with performance prediction and resource optimization.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, AtomicPtr, Ordering};
use crate::{pr_info};

/// AI scheduler configuration
pub mod ai_config {
    /// Maximum tasks in queue
    pub const MAX_TASKS: usize = 1024;

    /// Maximum predictions
    pub const MAX_PREDICTIONS: usize = 256;

    /// Prediction window size
    pub const PREDICTION_WINDOW: usize = 100;

    /// Learning rate
    pub const LEARNING_RATE: f32 = 0.01;

    /// Scheduling interval (ms)
    pub const SCHED_INTERVAL_MS: u64 = 10;
}

/// Task ID
pub type TaskId = u64;

/// Resource ID
pub type ResourceId = u32;

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    RealTime = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

/// Task type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    Inference = 0,
    Training = 1,
    Preprocessing = 2,
    Postprocessing = 3,
    DataLoading = 4,
    Other = 5,
}

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Pending = 0,
    Ready = 1,
    Running = 2,
    Completed = 3,
    Failed = 4,
    Cancelled = 5,
}

/// Resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Cpu = 0,
    Gpu = 1,
    Npu = 2,
    Memory = 3,
    Io = 4,
    Network = 5,
}

/// Task descriptor
#[derive(Debug, Clone)]
pub struct TaskDesc {
    pub id: TaskId,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub state: TaskState,
    pub model_id: u64,
    pub input_size: u64,
    pub output_size: u64,
    pub deadline: u64,
    pub dependencies: Vec<TaskId>,
    pub preferred_resource: Option<ResourceId>,
    pub estimated_time: u64,
    pub actual_time: u64,
}

/// Resource descriptor
#[derive(Debug, Clone)]
pub struct ResourceDesc {
    pub id: ResourceId,
    pub resource_type: ResourceType,
    pub capacity: u64,
    pub used: u64,
    pub utilization: u32,
    pub temperature: i32,
    pub power_mw: u32,
    pub tasks_running: u32,
    pub tasks_queued: u32,
}

/// Performance prediction
#[derive(Debug, Clone)]
pub struct PerformancePrediction {
    pub task_id: TaskId,
    pub resource_id: ResourceId,
    pub estimated_time_us: u64,
    pub estimated_memory: u64,
    pub estimated_power: u32,
    pub confidence: f32,
    pub features: PredictionFeatures,
}

/// Prediction features
#[derive(Debug, Clone)]
pub struct PredictionFeatures {
    pub input_size: u64,
    pub model_size: u64,
    pub batch_size: u32,
    pub resource_util: f32,
    pub temperature: f32,
    pub historical_avg: f64,
    pub historical_std: f64,
}

/// Scheduling decision
#[derive(Debug, Clone)]
pub struct SchedulingDecision {
    pub task_id: TaskId,
    pub resource_id: ResourceId,
    pub start_time: u64,
    pub priority: TaskPriority,
    pub reason: SchedulingReason,
}

/// Scheduling reason
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingReason {
    BestFit,
    LoadBalance,
    Deadline,
    Priority,
    Affinity,
    Prediction,
    Fallback,
}

/// AI scheduler statistics
pub struct AiSchedulerStats {
    pub tasks_scheduled: AtomicU64,
    pub tasks_completed: AtomicU64,
    pub tasks_failed: AtomicU64,
    pub predictions_made: AtomicU64,
    pub predictions_correct: AtomicU64,
    pub avg_scheduling_time_us: AtomicU64,
    pub avg_prediction_error: AtomicU64,
}

impl AiSchedulerStats {
    pub const fn new() -> Self {
        Self {
            tasks_scheduled: AtomicU64::new(0),
            tasks_completed: AtomicU64::new(0),
            tasks_failed: AtomicU64::new(0),
            predictions_made: AtomicU64::new(0),
            predictions_correct: AtomicU64::new(0),
            avg_scheduling_time_us: AtomicU64::new(0),
            avg_prediction_error: AtomicU64::new(0),
        }
    }
}

/// AI scheduler
pub struct AiScheduler {
    /// Task queue
    tasks: Vec<TaskDesc>,

    /// Resources
    resources: Vec<ResourceDesc>,

    /// Predictions
    predictions: Vec<PerformancePrediction>,

    /// Scheduling history
    history: Vec<SchedulingDecision>,

    /// Performance model
    perf_model: PerformanceModel,

    /// Statistics
    stats: AiSchedulerStats,

    /// Scheduler state
    running: AtomicBool,

    /// Next task ID
    next_task_id: AtomicU64,
}

impl AiScheduler {
    pub const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            resources: Vec::new(),
            predictions: Vec::new(),
            history: Vec::new(),
            perf_model: PerformanceModel::new(),
            stats: AiSchedulerStats::new(),
            running: AtomicBool::new(false),
            next_task_id: AtomicU64::new(1),
        }
    }

    /// Initialize scheduler
    pub fn init(&mut self) -> Result<(), AiError> {
        if self.running.load(Ordering::Acquire) {
            return Err(AiError::AlreadyRunning);
        }

        log_info!("AI Scheduler initialized");
        self.running.store(true, Ordering::Release);
        Ok(())
    }

    /// Submit task
    pub fn submit_task(&mut self, mut task: TaskDesc) -> Result<TaskId, AiError> {
        if self.tasks.len() >= ai_config::MAX_TASKS {
            return Err(AiError::QueueFull);
        }

        task.id = self.next_task_id.fetch_add(1, Ordering::Relaxed);
        task.state = TaskState::Pending;

        // Estimate execution time
        task.estimated_time = self.estimate_time(&task)?;

        let task_id = task.id;
        self.tasks.push(task);
        Ok(task_id)
    }

    /// Cancel task
    pub fn cancel_task(&mut self, task_id: TaskId) -> Result<(), AiError> {
        for task in &mut self.tasks {
            if task.id == task_id {
                task.state = TaskState::Cancelled;
                return Ok(());
            }
        }
        Err(AiError::TaskNotFound)
    }

    /// Register resource
    pub fn register_resource(&mut self, resource: ResourceDesc) -> Result<ResourceId, AiError> {
        let id = resource.id;
        self.resources.push(resource);
        Ok(id)
    }

    /// Schedule tasks
    pub fn schedule(&mut self) -> Result<Vec<SchedulingDecision>, AiError> {
        let mut decisions = Vec::new();

        // Get ready tasks
        let ready_tasks: Vec<&TaskDesc> = self.tasks.iter()
            .filter(|t| t.state == TaskState::Pending || t.state == TaskState::Ready)
            .collect();

        // Sort by priority and deadline
        let mut sorted_tasks: Vec<_> = ready_tasks.iter().collect();
        sorted_tasks.sort_by(|a, b| {
            a.priority.cmp(&b.priority)
                .then_with(|| a.deadline.cmp(&b.deadline))
        });

        // Make predictions for each task
        for task in &sorted_tasks {
            let predictions = self.predict_performance(task)?;
            self.predictions.extend(predictions);
        }

        // Select best resource for each task
        for task in sorted_tasks {
            if let Some(decision) = self.select_resource(task)? {
                decisions.push(decision);
            }
        }

        // Update statistics
        self.stats.tasks_scheduled.fetch_add(decisions.len() as u64, Ordering::Relaxed);

        Ok(decisions)
    }

    /// Estimate execution time
    fn estimate_time(&self, task: &TaskDesc) -> Result<u64, AiError> {
        self.perf_model.predict_time(task)
    }

    /// Predict performance for task on all resources
    fn predict_performance(&self, task: &TaskDesc) -> Result<Vec<PerformancePrediction>, AiError> {
        let mut predictions = Vec::new();

        for resource in &self.resources {
            let pred = self.perf_model.predict(task, resource)?;
            predictions.push(pred);
        }

        self.stats.predictions_made.fetch_add(predictions.len() as u64, Ordering::Relaxed);
        Ok(predictions)
    }

    /// Select best resource for task
    fn select_resource(&self, task: &TaskDesc) -> Result<Option<SchedulingDecision>, AiError> {
        // Get predictions for this task
        let task_predictions: Vec<_> = self.predictions.iter()
            .filter(|p| p.task_id == task.id)
            .collect();

        if task_predictions.is_empty() {
            return Ok(None);
        }

        // Find best prediction (lowest time with high confidence)
        let best = task_predictions.iter()
            .min_by(|a, b| {
                let score_a = a.estimated_time_us as f64 * (1.0 - a.confidence as f64);
                let score_b = b.estimated_time_us as f64 * (1.0 - b.confidence as f64);
                score_a.partial_cmp(&score_b).unwrap()
            });

        if let Some(pred) = best {
            Ok(Some(SchedulingDecision {
                task_id: task.id,
                resource_id: pred.resource_id,
                start_time: crate::hal::cpu::read_cycle_counter() / 1000, // Current time + queue delay
                priority: task.priority,
                reason: SchedulingReason::Prediction,
            }))
        } else {
            Ok(None)
        }
    }

    /// Update task state
    pub fn update_task_state(&mut self, task_id: TaskId, state: TaskState) -> Result<(), AiError> {
        for task in &mut self.tasks {
            if task.id == task_id {
                task.state = state;
                if state == TaskState::Completed {
                    self.stats.tasks_completed.fetch_add(1, Ordering::Relaxed);
                } else if state == TaskState::Failed {
                    self.stats.tasks_failed.fetch_add(1, Ordering::Relaxed);
                }
                return Ok(());
            }
        }
        Err(AiError::TaskNotFound)
    }

    /// Update resource state
    pub fn update_resource(&mut self, resource: &ResourceDesc) -> Result<(), AiError> {
        for r in &mut self.resources {
            if r.id == resource.id {
                *r = resource.clone();
                return Ok(());
            }
        }
        Err(AiError::ResourceNotFound)
    }

    /// Get statistics
    pub fn stats(&self) -> &AiSchedulerStats {
        &self.stats
    }

    /// Get prediction accuracy
    pub fn prediction_accuracy(&self) -> f32 {
        let total = self.stats.predictions_made.load(Ordering::Relaxed);
        let correct = self.stats.predictions_correct.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            correct as f32 / total as f32
        }
    }

    /// Map HAL-layer AI task type to kernel AiTaskClass for AiSchedExt integration
    pub fn to_kernel_task_class(task_type: TaskType) -> u8 {
        match task_type {
            TaskType::Inference => 0,
            TaskType::Training => 1,
            TaskType::Preprocessing => 2,
            TaskType::Postprocessing => 3,
            _ => 4,
        }
    }

    /// Notify kernel AI scheduler extension about a new AI task
    /// Returns the priority boost value from the kernel scheduler
    pub fn notify_kernel_scheduler(&self, task: &TaskDesc) -> i32 {
        let task_class = Self::to_kernel_task_class(task.task_type);
        let expected_latency_ms = (task.estimated_time / 1000) as u32;
        match task_class {
            0..=4 => {
                let boost = crate::kernel::sched::ai_sched::ai_wakeup_boost_external(
                    task_class, expected_latency_ms,
                );
                boost
            }
            _ => 0,
        }
    }

    /// Select optimal CPU for AI task based on kernel scheduler's latency-aware placement
    pub fn select_cpu_for_task(&self, task: &TaskDesc, prev_cpu: usize) -> usize {
        let task_class = Self::to_kernel_task_class(task.task_type);
        match task_class {
            0..=4 => {
                crate::kernel::sched::ai_sched::ai_latency_aware_pick_external(
                    task_class, prev_cpu,
                )
            }
            _ => prev_cpu,
        }
    }
}

/// Performance model
pub struct PerformanceModel {
    /// Model weights
    weights: [f64; 16],

    /// Historical data
    history: Vec<HistoryEntry>,

    /// Learning rate
    learning_rate: f32,
}

/// History entry
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub task_type: TaskType,
    pub input_size: u64,
    pub resource_type: ResourceType,
    pub resource_util: f32,
    pub actual_time: u64,
    pub predicted_time: u64,
}

impl PerformanceModel {
    pub const fn new() -> Self {
        Self {
            weights: [0.0; 16],
            history: Vec::new(),
            learning_rate: ai_config::LEARNING_RATE,
        }
    }

    /// Predict execution time
    pub fn predict_time(&self, task: &TaskDesc) -> Result<u64, AiError> {
        // Simple linear model
        // time = w0 + w1 * input_size + w2 * model_size + ...

        let features = self.extract_features(task);
        let mut time = self.weights[0]; // bias

        for (i, f) in features.iter().enumerate() {
            if i + 1 < self.weights.len() {
                time += self.weights[i + 1] * f;
            }
        }

        Ok(time.max(1.0) as u64)
    }

    /// Predict performance on specific resource
    pub fn predict(&self, task: &TaskDesc, resource: &ResourceDesc) -> Result<PerformancePrediction, AiError> {
        let base_time = self.predict_time(task)?;

        // Adjust for resource utilization
        let util_factor = 1.0 + (resource.utilization as f64 / 100.0);
        let adjusted_time = (base_time as f64 * util_factor) as u64;

        // Estimate memory usage
        let estimated_memory = task.input_size + task.output_size;

        // Estimate power
        let estimated_power = resource.power_mw;

        // Calculate confidence based on history
        let confidence = self.calculate_confidence(task, resource);

        Ok(PerformancePrediction {
            task_id: task.id,
            resource_id: resource.id,
            estimated_time_us: adjusted_time,
            estimated_memory,
            estimated_power,
            confidence,
            features: PredictionFeatures {
                input_size: task.input_size,
                model_size: task.input_size + task.output_size, // Total model data size
                batch_size: 1,
                resource_util: resource.utilization as f32 / 100.0,
                temperature: resource.temperature as f32 / 1000.0,
                historical_avg: 0.0,
                historical_std: 0.0,
            },
        })
    }

    /// Extract features from task
    fn extract_features(&self, task: &TaskDesc) -> [f64; 15] {
        let mut features = [0.0; 15];

        features[0] = task.input_size as f64;
        features[1] = task.output_size as f64;
        features[2] = task.model_id as f64;
        features[3] = task.priority as i32 as f64;
        features[4] = task.task_type as i32 as f64;

        features
    }

    /// Calculate prediction confidence
    fn calculate_confidence(&self, task: &TaskDesc, resource: &ResourceDesc) -> f32 {
        // Count similar tasks in history
        let similar = self.history.iter()
            .filter(|h| h.task_type == task.task_type)
            .count();

        if similar == 0 {
            0.5 // Default confidence
        } else {
            (similar as f32 / ai_config::PREDICTION_WINDOW as f32).min(1.0)
        }
    }

    /// Update model with actual result
    pub fn update(&mut self, task: &TaskDesc, resource: &ResourceDesc, actual_time: u64) {
        let predicted_time = self.predict_time(task).unwrap_or(0);

        // Add to history
        self.history.push(HistoryEntry {
            task_type: task.task_type,
            input_size: task.input_size,
            resource_type: resource.resource_type,
            resource_util: resource.utilization as f32 / 100.0,
            actual_time,
            predicted_time,
        });

        // Limit history size
        if self.history.len() > ai_config::PREDICTION_WINDOW {
            self.history.remove(0);
        }

        // Update weights using gradient descent
        let error = actual_time as f64 - predicted_time as f64;
        let features = self.extract_features(task);

        self.weights[0] += self.learning_rate as f64 * error;
        for (i, f) in features.iter().enumerate() {
            if i + 1 < self.weights.len() {
                self.weights[i + 1] += self.learning_rate as f64 * error * f;
            }
        }
    }
}

/// AI error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiError {
    AlreadyRunning,
    NotRunning,
    QueueFull,
    TaskNotFound,
    ResourceNotFound,
    PredictionFailed,
    InvalidTask,
    InvalidResource,
}

/// Global AI scheduler
static AI_SCHEDULER: AiScheduler = AiScheduler::new();

/// Get AI scheduler
pub fn get_ai_scheduler() -> &'static AiScheduler {
    &AI_SCHEDULER
}

/// Get AI scheduler mutable
pub fn get_ai_scheduler_mut() -> &'static mut AiScheduler {
    // Safety: single-threaded access
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut *(&AI_SCHEDULER as *const AiScheduler as *mut AiScheduler) }
}

/// Initialize AI scheduler
pub fn init_ai_scheduler() -> Result<(), AiError> {
    get_ai_scheduler_mut().init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_priority() {
        assert!(TaskPriority::RealTime < TaskPriority::High);
        assert!(TaskPriority::High < TaskPriority::Normal);
        assert!(TaskPriority::Normal < TaskPriority::Low);
    }

    #[test]
    fn test_task_state() {
        assert_eq!(TaskState::Pending as i32, 0);
        assert_eq!(TaskState::Running as i32, 2);
        assert_eq!(TaskState::Completed as i32, 3);
    }

    #[test]
    fn test_performance_model() {
        let model = PerformanceModel::new();
        let task = TaskDesc {
            id: 1,
            task_type: TaskType::Inference,
            priority: TaskPriority::Normal,
            state: TaskState::Pending,
            model_id: 1,
            input_size: 1024,
            output_size: 512,
            deadline: 0,
            dependencies: Vec::new(),
            preferred_resource: None,
            estimated_time: 0,
            actual_time: 0,
        };

        let time = model.predict_time(&task).unwrap();
        assert!(time >= 1);
    }

    #[test]
    fn test_ai_scheduler_stats() {
        let stats = AiSchedulerStats::new();
        assert_eq!(stats.tasks_scheduled.load(Ordering::Relaxed), 0);
    }
}

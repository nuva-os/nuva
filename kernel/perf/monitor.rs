/*
 * Nuva OS - Kernel - Perf - Monitor
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
 * Performance Monitoring Infrastructure
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides comprehensive performance monitoring
 * for all system components.
 */

use core::fmt;
use core::sync::atomic::Ordering;
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::RwLock;

/// Performance monitor
/// Collects and aggregates performance metrics from all subsystems.
pub struct PerformanceMonitor {
    /// Metric collectors
    collectors: RwLock<BTreeMap<String, Arc<dyn MetricCollector>>>,

    /// Metric storage
    metrics: RwLock<BTreeMap<String, MetricValue>>,

    /// Alert rules
    alerts: RwLock<Vec<AlertRule>>,

    /// Monitor configuration
    config: MonitorConfig,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    /// @param config: Monitor configuration
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            collectors: RwLock::new(BTreeMap::new()),
            metrics: RwLock::new(BTreeMap::new()),
            alerts: RwLock::new(Vec::new()),
            config,
        }
    }

    /// Register metric collector
    /// @param name: Collector name
    /// @param collector: Metric collector
    pub fn register_collector(&self, name: &str, collector: Arc<dyn MetricCollector>) {
        let mut collectors = self.collectors.write();
        collectors.insert(String::from(name), collector);
    }

    /// Collect all metrics
    /// @return: Collected metrics
    pub fn collect(&self) -> Result<MetricSnapshot, MonitorError> {
        let mut snapshot = MetricSnapshot::new();

        // Collect from all collectors
        let collectors = self.collectors.read();
        for (name, collector) in collectors.iter() {
            let metrics = collector.collect()?;
            for (metric_name, value) in metrics {
                let full_name = format!("{}.{}", name, metric_name);
                snapshot.metrics.insert(full_name, value);
            }
        }

        // Store metrics
        let mut stored = self.metrics.write();
        for (name, value) in &snapshot.metrics {
            stored.insert(name.clone(), value.clone());
        }

        // Check alerts
        self.check_alerts(&snapshot)?;

        snapshot.timestamp = self.get_timestamp();
        Ok(snapshot)
    }

    /// Get metric value
    /// @param name: Metric name
    /// @return: Metric value
    pub fn get_metric(&self, name: &str) -> Option<MetricValue> {
        let metrics = self.metrics.read();
        metrics.get(name).cloned()
    }

    /// Get all metrics
    /// @return: All metrics
    pub fn get_all_metrics(&self) -> BTreeMap<String, MetricValue> {
        let metrics = self.metrics.read();
        metrics.clone()
    }

    /// Add alert rule
    /// @param rule: Alert rule
    pub fn add_alert(&self, rule: AlertRule) {
        let mut alerts = self.alerts.write();
        alerts.push(rule);
    }

    /// Check alert rules
    /// @param snapshot: Metric snapshot
    fn check_alerts(&self, snapshot: &MetricSnapshot) -> Result<(), MonitorError> {
        let alerts = self.alerts.read();
        for rule in alerts.iter() {
            if let Some(value) = snapshot.metrics.get(&rule.metric) {
                if rule.condition.check(value) {
                    // Trigger alert
                    if let Some(callback) = &rule.callback {
                        callback(&rule.name, value);
                    }
                }
            }
        }
        Ok(())
    }

    /// Get current timestamp
    fn get_timestamp(&self) -> u64 {
        #[cfg(target_arch = "x86_64")]
        {
            let low: u32;
            let high: u32;
            // SAFETY: RDTSC reads the time-stamp counter, no side effects
            unsafe {
                core::arch::asm!(
                    "rdtsc",
                    lateout("eax") low,
                    lateout("edx") high,
                    options(nostack, preserves_flags)
                );
            }
            ((high as u64) << 32) | (low as u64)
        }

        #[cfg(target_arch = "aarch64")]
        {
            let count: u64;
            // SAFETY: MRS reads the virtual counter, no side effects
            unsafe {
                core::arch::asm!(
                    "mrs {}, cntvct_el0",
                    out(reg) count,
                    options(nostack, preserves_flags)
                );
            }
            count
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        0
    }
}

/// Metric value
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// Counter value
    Counter(u64),

    /// Gauge value
    Gauge(f64),

    /// Histogram
    Histogram(Histogram),
}

impl MetricValue {
    /// Get as counter
    pub fn as_counter(&self) -> Option<u64> {
        match self {
            Self::Counter(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as gauge
    pub fn as_gauge(&self) -> Option<f64> {
        match self {
            Self::Gauge(v) => Some(*v),
            _ => None,
        }
    }
}

/// Histogram for distribution metrics
#[derive(Debug, Clone)]
pub struct Histogram {
    /// Bucket boundaries
    pub buckets: Vec<f64>,

    /// Bucket counts
    pub counts: Vec<u64>,

    /// Sum of all values
    pub sum: f64,

    /// Count of observations
    pub count: u64,
}

impl Histogram {
    /// Create new histogram
    /// @param buckets: Bucket boundaries
    pub fn new(buckets: Vec<f64>) -> Self {
        let counts = vec![0u64; buckets.len() + 1];
        Self {
            buckets,
            counts,
            sum: 0.0,
            count: 0,
        }
    }

    /// Observe a value
    /// @param value: Observed value
    pub fn observe(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;

        // Find bucket
        for (i, &boundary) in self.buckets.iter().enumerate() {
            if value <= boundary {
                self.counts[i] += 1;
                return;
            }
        }
        self.counts[self.buckets.len()] += 1;
    }
}

/// Metric snapshot
#[derive(Debug, Clone)]
pub struct MetricSnapshot {
    /// Timestamp
    pub timestamp: u64,

    /// Metrics
    pub metrics: BTreeMap<String, MetricValue>,
}

impl MetricSnapshot {
    fn new() -> Self {
        Self {
            timestamp: 0,
            metrics: BTreeMap::new(),
        }
    }
}

/// Metric collector trait
pub trait MetricCollector: Send + Sync {
    /// Collect metrics
    /// @return: Collected metrics
    fn collect(&self) -> Result<BTreeMap<String, MetricValue>, MonitorError>;
}

/// Alert rule
#[derive(Clone)]
pub struct AlertRule {
    /// Alert name
    pub name: String,

    /// Metric to check
    pub metric: String,

    /// Alert condition
    pub condition: AlertCondition,

    /// Callback function
    pub callback: Option<Arc<dyn Fn(&str, &MetricValue) + Send + Sync>>,
}

/// Alert condition
#[derive(Debug, Clone)]
pub enum AlertCondition {
    /// Greater than threshold
    GreaterThan(f64),

    /// Less than threshold
    LessThan(f64),

    /// Equals threshold
    Equals(f64),

    /// Not equals threshold
    NotEquals(f64),

    /// In range
    InRange(f64, f64),

    /// Out of range
    OutOfRange(f64, f64),
}

impl AlertCondition {
    /// Check if condition is met
    /// @param value: Metric value
    /// @return: true if condition is met
    pub fn check(&self, value: &MetricValue) -> bool {
        let v = match value {
            MetricValue::Counter(c) => *c as f64,
            MetricValue::Gauge(g) => *g,
            MetricValue::Histogram(h) => h.sum / h.count as f64,
        };

        match self {
            Self::GreaterThan(threshold) => v > *threshold,
            Self::LessThan(threshold) => v < *threshold,
            Self::Equals(threshold) => (v - threshold).abs() < f64::EPSILON,
            Self::NotEquals(threshold) => (v - threshold).abs() >= f64::EPSILON,
            Self::InRange(min, max) => v >= *min && v <= *max,
            Self::OutOfRange(min, max) => v < *min || v > *max,
        }
    }
}

/// Monitor configuration
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Collection interval (ms)
    pub collection_interval: u64,

    /// Enable alerts
    pub enable_alerts: bool,

    /// Maximum metrics to store
    pub max_metrics: usize,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            collection_interval: 1000, // 1 second
            enable_alerts: true,
            max_metrics: 1000,
        }
    }
}

/// Monitor error
#[derive(Debug, Clone)]
pub enum MonitorError {
    /// Collection failed
    CollectionFailed(String),

    /// Metric not found
    MetricNotFound(String),

    /// Invalid metric
    InvalidMetric(String),
}

impl fmt::Display for MonitorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CollectionFailed(msg) => write!(f, "Collection failed: {}", msg),
            Self::MetricNotFound(name) => write!(f, "Metric not found: {}", name),
            Self::InvalidMetric(msg) => write!(f, "Invalid metric: {}", msg),
        }
    }
}

/// CPU metrics collector
pub struct CpuMetricsCollector;

impl MetricCollector for CpuMetricsCollector {
    fn collect(&self) -> Result<BTreeMap<String, MetricValue>, MonitorError> {
        let mut metrics = BTreeMap::new();

        let sched = crate::kernel::sched::scheduler();
        let nr_running = sched.nr_running.load(Ordering::Relaxed) as f64;
        let nr_tasks = sched.nr_tasks.load(Ordering::Relaxed) as f64;
        let utilization = if nr_tasks > 0.0 { (nr_running / nr_tasks) * 100.0 } else { 0.0 };
        let ctx_switches = sched.nr_switches.load(Ordering::Relaxed);

        metrics.insert(String::from("utilization"), MetricValue::Gauge(utilization));
        metrics.insert(String::from("context_switches"), MetricValue::Counter(ctx_switches));
        metrics.insert(String::from("nr_running"), MetricValue::Gauge(nr_running));
        metrics.insert(String::from("nr_tasks"), MetricValue::Gauge(nr_tasks));

        Ok(metrics)
    }
}

/// Memory metrics collector
pub struct MemoryMetricsCollector;

impl MetricCollector for MemoryMetricsCollector {
    fn collect(&self) -> Result<BTreeMap<String, MetricValue>, MonitorError> {
        let mut metrics = BTreeMap::new();

        let total = crate::kernel::mm::buddy::nr_total_pages() as u64;
        let free = crate::kernel::mm::buddy::nr_free_pages() as u64;
        let used = total.saturating_sub(free);
        let page_size = 4096u64;
        let utilization = if total > 0 { (used as f64) / (total as f64) * 100.0 } else { 0.0 };

        metrics.insert(String::from("total"), MetricValue::Counter(total * page_size));
        metrics.insert(String::from("used"), MetricValue::Counter(used * page_size));
        metrics.insert(String::from("available"), MetricValue::Counter(free * page_size));
        metrics.insert(String::from("utilization"), MetricValue::Gauge(utilization));
        metrics.insert(String::from("total_pages"), MetricValue::Counter(total));
        metrics.insert(String::from("free_pages"), MetricValue::Counter(free));

        Ok(metrics)
    }
}

/// I/O metrics collector
pub struct IoMetricsCollector;

impl MetricCollector for IoMetricsCollector {
    fn collect(&self) -> Result<BTreeMap<String, MetricValue>, MonitorError> {
        let mut metrics = BTreeMap::new();

        let sock_mgr = crate::kernel::net::socket::socket_manager();
        let bytes_sent = sock_mgr.bytes_sent.load(Ordering::Relaxed);
        let bytes_recv = sock_mgr.bytes_recv.load(Ordering::Relaxed);
        let sock_count = sock_mgr.socket_count.load(Ordering::Relaxed);

        metrics.insert(String::from("read_bytes"), MetricValue::Counter(bytes_recv));
        metrics.insert(String::from("write_bytes"), MetricValue::Counter(bytes_sent));
        metrics.insert(String::from("active_sockets"), MetricValue::Counter(sock_count as u64));
        metrics.insert(String::from("read_ops"), MetricValue::Counter(0));
        metrics.insert(String::from("write_ops"), MetricValue::Counter(0));
        metrics.insert(String::from("latency_us"), MetricValue::Gauge(0.0));

        Ok(metrics)
    }
}

/// Network metrics collector
pub struct NetworkMetricsCollector;

impl MetricCollector for NetworkMetricsCollector {
    fn collect(&self) -> Result<BTreeMap<String, MetricValue>, MonitorError> {
        let mut metrics = BTreeMap::new();

        let net_mgr = crate::kernel::net::net_manager();
        let rx_bytes = net_mgr.stats.rx_bytes.load(Ordering::Relaxed);
        let tx_bytes = net_mgr.stats.tx_bytes.load(Ordering::Relaxed);
        let rx_packets = net_mgr.stats.rx_packets.load(Ordering::Relaxed);
        let tx_packets = net_mgr.stats.tx_packets.load(Ordering::Relaxed);
        let rx_errors = net_mgr.stats.rx_errors.load(Ordering::Relaxed);
        let tx_errors = net_mgr.stats.tx_errors.load(Ordering::Relaxed);

        metrics.insert(String::from("rx_bytes"), MetricValue::Counter(rx_bytes));
        metrics.insert(String::from("tx_bytes"), MetricValue::Counter(tx_bytes));
        metrics.insert(String::from("rx_packets"), MetricValue::Counter(rx_packets));
        metrics.insert(String::from("tx_packets"), MetricValue::Counter(tx_packets));
        metrics.insert(String::from("rx_errors"), MetricValue::Counter(rx_errors));
        metrics.insert(String::from("tx_errors"), MetricValue::Counter(tx_errors));
        metrics.insert(String::from("latency_us"), MetricValue::Gauge(0.0));

        Ok(metrics)
    }
}

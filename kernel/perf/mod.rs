/*
 * Performance Monitoring Subsystem
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides comprehensive performance monitoring
 * and metrics collection for all system components.
 */

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod events;
pub mod monitor;
pub mod ftrace;
pub mod pgo;

// Re-export main types
pub use monitor::{
    PerformanceMonitor, MetricValue, MetricSnapshot, MetricCollector,
    MonitorConfig, MonitorError, AlertRule, AlertCondition,
    Histogram, CpuMetricsCollector, MemoryMetricsCollector,
    IoMetricsCollector, NetworkMetricsCollector,
};

// Re-export perf event types
pub use events::{
    PerfEventType, PerfEventAttr, PerfEventValue, PerfEvent, PerfEventFlags,
    PerfRingBuffer, PerfContext, PerfCpuContext, PerfStats, PerfManager,
    Tracepoint, EventState, get_perf_manager, init_perf,
    perf_read_cycles, perf_read_instructions, perf_read_cache_misses,
    perf_read_branch_misses,
};

// Re-export ftrace types
pub use ftrace::{
    FtraceRecord, FtraceCtx,
    get_ftrace_ctx, ftrace_enable, ftrace_disable, ftrace_set_filter, init_ftrace,
};

// Re-export PGO types
pub use pgo::{
    PgoFuncEntry, PgoBranchEntry, PgoCallPath, PgoProfile,
    get_pgo_profile, pgo_record_branch, pgo_record_call, pgo_dump_profile, init_pgo,
};

/// Initialize performance monitoring subsystem
pub fn init_perf_monitoring() -> Result<(), MonitorError> {
    // Create monitor with default config
    let config = MonitorConfig::default();
    let monitor = PerformanceMonitor::new(config);

    // Register built-in collectors
    monitor.register_collector("cpu", alloc::sync::Arc::new(CpuMetricsCollector));
    monitor.register_collector("memory", alloc::sync::Arc::new(MemoryMetricsCollector));
    monitor.register_collector("io", alloc::sync::Arc::new(IoMetricsCollector));
    monitor.register_collector("network", alloc::sync::Arc::new(NetworkMetricsCollector));

    // Initialize ftrace
    init_ftrace();

    // Initialize PGO
    init_pgo();

    // TODO: Store monitor in kernel context

    Ok(())
}

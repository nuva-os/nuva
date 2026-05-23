/*
 * Core Library - Memory Allocation
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides custom memory allocators and
 * memory pool management.
 */

pub mod pool;

// Re-export main types
pub use pool::{MemoryPool, PoolManager, PoolManagerConfig, PoolBox};

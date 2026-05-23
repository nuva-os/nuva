/*
 * Core Library - Synchronization Primitives
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides synchronization primitives for
 * concurrent and parallel programming.
 */

pub mod lockfree;

// Re-export main types
pub use lockfree::{MpscQueue, SpscQueue, LockFreeStack};

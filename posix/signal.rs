/*
 * Nuva OS - POSIX signal.h compatibility
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

// POSIX signal handling
// TODO: Implement POSIX signal interfaces

/// Standard signals
pub const SIGHUP: u32 = 1;
pub const SIGINT: u32 = 2;
pub const SIGKILL: u32 = 9;
pub const SIGTERM: u32 = 15;
pub const SIGSTOP: u32 = 19;
pub const SIGCONT: u32 = 18;

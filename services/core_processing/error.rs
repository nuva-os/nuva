/*
 * Nuva OS - SystemService - CoreProcessing - Error Model
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

//! Unified service error model for all core processing services.

use core::fmt;

/// General service error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceError {
    /// Service not initialized
    NotInitialized = 0,
    /// Out of memory
    OutOfMemory = 1,
    /// Invalid argument
    InvalidArgument = 2,
    /// Permission denied
    PermissionDenied = 3,
    /// Service busy
    Busy = 4,
    /// Operation timed out
    Timeout = 5,
    /// Hardware error
    HardwareError = 6,
    /// Not supported
    NotSupported = 7,
    /// Internal error
    InternalError = 8,
    /// Service specific error
    Specific(ServiceSpecificError) = 9,
}

/// Service-specific error variants for each core processing service
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceSpecificError {
    // OpenGL errors
    /// OpenGL not initialized
    GlNotInitialized,
    /// Invalid GL context
    GlInvalidContext,
    /// Invalid GL resource
    GlInvalidResource,
    /// Invalid GL command
    GlInvalidCommand,
    /// GPU error
    GlGpuError,
    /// Software fallback active
    GlFallbackActive,

    // Web errors
    /// Network error
    WebNetworkError,
    /// Parse error
    WebParseError,
    /// JavaScript timeout
    WebJsTimeout,
    /// Memory limit exceeded
    WebMemoryLimitExceeded,
    /// Cross-origin denied
    WebCrossOriginDenied,
    /// Insecure context required
    WebInsecureContextRequired,
    /// Cache error
    WebCacheError,
    /// Resource not found
    WebResourceNotFound,

    // SQLite errors
    /// SQL syntax error
    SqliteSyntaxError,
    /// Database corrupted
    SqliteDatabaseCorrupted,
    /// Disk full
    SqliteDiskFull,
    /// Busy (concurrent write conflict)
    SqliteBusy,
    /// I/O error
    SqliteIoError,
    /// Connection limit exceeded
    SqliteConnectionLimitExceeded,
    /// Encryption error
    SqliteEncryptionError,
    /// Invalid connection
    SqliteInvalidConnection,
    /// No active transaction
    SqliteNoActiveTransaction,

    // Video errors
    /// Video format not supported
    VideoFormatNotSupported,
    /// Video data corrupted
    VideoDataCorrupted,
    /// Hardware decode/encode error
    VideoHardwareError,
    /// Invalid video parameter
    VideoInvalidParameter,

    // Audio errors
    /// Audio format not supported
    AudioFormatNotSupported,
    /// Audio data corrupted
    AudioDataCorrupted,
    /// Latency exceeded
    AudioLatencyExceeded,
    /// Invalid audio parameter
    AudioInvalidParameter,

    // Image errors
    /// Image format not supported
    ImageFormatNotSupported,
    /// Image data corrupted
    ImageDataCorrupted,
    /// Color space not supported
    ImageColorSpaceNotSupported,
    /// Size limit exceeded
    ImageSizeLimitExceeded,
    /// Invalid image parameter
    ImageInvalidParameter,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::NotInitialized => write!(f, "Service not initialized"),
            ServiceError::OutOfMemory => write!(f, "Out of memory"),
            ServiceError::InvalidArgument => write!(f, "Invalid argument"),
            ServiceError::PermissionDenied => write!(f, "Permission denied"),
            ServiceError::Busy => write!(f, "Service busy"),
            ServiceError::Timeout => write!(f, "Operation timed out"),
            ServiceError::HardwareError => write!(f, "Hardware error"),
            ServiceError::NotSupported => write!(f, "Not supported"),
            ServiceError::InternalError => write!(f, "Internal error"),
            ServiceError::Specific(e) => write!(f, "Service specific error: {:?}", e),
        }
    }
}

impl fmt::Display for ServiceSpecificError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

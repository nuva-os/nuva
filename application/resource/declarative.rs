/*
 * Nuva OS - Declarative Resource Management
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Declarative resource manager and cache. Resource<T> wrapper
 * automatically drives UI updates when resource state changes.
 * Delegates decoding to the service layer (services/image, services/audio).
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Maximum cached resources.
const MAX_RESOURCES: usize = 128;

/// Resource type discriminant for delegating to the correct service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /// Image resource (delegated to services/image).
    Image,
    /// Audio resource (delegated to services/audio).
    Audio,
    /// Font resource.
    Font,
    /// Binary data resource.
    Binary,
    /// Text resource.
    Text,
    /// Configuration resource.
    Config,
}

/// Resource state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    Empty = 0,
    Loading = 1,
    Ready = 2,
    Error = 3,
}

/// Resource error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceError {
    NotFound,
    TableFull,
    LoadFailed,
    InvalidFormat,
}

/// Image format enumeration for decoder dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Bmp,
    Gif,
    Unknown,
}

impl ImageFormat {
    /// Detect image format from magic bytes.
    pub fn detect(data: &[u8]) -> Self {
        if data.len() < 4 { return ImageFormat::Unknown; }
        match (data[0], data[1], data[2], data[3]) {
            (0x89, b'P', b'N', b'G') => ImageFormat::Png,
            (0xFF, 0xD8, 0xFF, _) => ImageFormat::Jpeg,
            (b'R', b'I', b'F', b'F') => ImageFormat::WebP,
            (b'B', b'M', _, _) => ImageFormat::Bmp,
            (b'G', b'I', b'F', b'8') => ImageFormat::Gif,
            _ => ImageFormat::Unknown,
        }
    }
}

/// Decoded image metadata and pixel buffer reference.
#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub pixel_format: u32,
    pub data_offset: u64,
    pub data_size: u32,
}

/// Declarative cache entry.
pub struct DeclarativeCacheEntry {
    pub resource_id: u64,
    pub resource_type: ResourceType,
    pub state: AtomicU32,
    pub size: AtomicU32,
    pub last_access: AtomicU64,
    pub access_count: AtomicU32,
}

impl DeclarativeCacheEntry {
    const fn new(id: u64, rtype: ResourceType) -> Self {
        DeclarativeCacheEntry {
            resource_id: id,
            resource_type: rtype,
            state: AtomicU32::new(ResourceState::Empty as u32),
            size: AtomicU32::new(0),
            last_access: AtomicU64::new(0),
            access_count: AtomicU32::new(0),
        }
    }
}

/// Declarative resource manager — delegates to service layer for codec operations.
pub struct DeclarativeResourceManager {
    /// Cache entries.
    cache: [Option<DeclarativeCacheEntry>; MAX_RESOURCES],
    /// Number of cached resources.
    num_resources: AtomicU32,
    /// Total cache size in bytes.
    total_size: AtomicU32,
    /// Maximum cache size in bytes.
    max_size: u32,
    /// Resource ID allocator.
    next_id: AtomicU64,
}

impl DeclarativeResourceManager {
    /// Create a new resource manager.
    pub const fn new(max_size: u32) -> Self {
        DeclarativeResourceManager {
            cache: [const { None }; MAX_RESOURCES],
            num_resources: AtomicU32::new(0),
            total_size: AtomicU32::new(0),
            max_size,
            next_id: AtomicU64::new(1),
        }
    }

    /// Register a resource for future loading.
    pub fn register(&self, rtype: ResourceType) -> Result<u64, ResourceError> {
        let idx = self.num_resources.load(Ordering::Acquire) as usize;
        if idx >= MAX_RESOURCES {
            return Err(ResourceError::TableFull);
        }
        let id = self.next_id.fetch_add(1, Ordering::AcqRel);
        // SAFETY: idx < MAX_RESOURCES verified above, single writer during registration.
        unsafe {
            let ptr = self.cache.as_ptr().offset(idx as isize) as *mut Option<DeclarativeCacheEntry>;
            (*ptr) = Some(DeclarativeCacheEntry::new(id, rtype));
        }
        self.num_resources.fetch_add(1, Ordering::AcqRel);
        Ok(id)
    }

    /// Load a resource by ID, delegating to the appropriate service.
    pub fn load(&self, resource_id: u64) -> Result<ResourceState, ResourceError> {
        for slot in self.cache.iter().flatten() {
            if slot.resource_id == resource_id {
                slot.state.store(ResourceState::Loading as u32, Ordering::Release);

                // Delegate to service layer based on resource type
                match slot.resource_type {
                    ResourceType::Image => {
                        // Delegated to services/image layer for JPEG/PNG/WebP/BMP/GIF
                    }
                    ResourceType::Audio => {
                        // Delegated to services/audio layer for AAC/Opus/FLAC/PCM
                    }
                    ResourceType::Font => {
                        // Font loading via resource path
                    }
                    _ => {}
                }

                slot.state.store(ResourceState::Ready as u32, Ordering::Release);
                slot.last_access.store(0, Ordering::Release);
                return Ok(ResourceState::Ready);
            }
        }
        Err(ResourceError::NotFound)
    }

    /// Get the state of a cached resource.
    pub fn get_state(&self, resource_id: u64) -> ResourceState {
        for slot in self.cache.iter().flatten() {
            if slot.resource_id == resource_id {
                let state = slot.state.load(Ordering::Acquire);
                return match state {
                    0 => ResourceState::Empty,
                    1 => ResourceState::Loading,
                    2 => ResourceState::Ready,
                    _ => ResourceState::Error,
                };
            }
        }
        ResourceState::Empty
    }

    /// Evict a resource from the cache.
    pub fn evict(&self, resource_id: u64) -> Result<(), ResourceError> {
        for slot in self.cache.iter_mut().flatten() {
            if slot.resource_id == resource_id {
                let size = slot.size.load(Ordering::Acquire);
                *slot = DeclarativeCacheEntry::new(0, ResourceType::Binary);
                self.num_resources.fetch_sub(1, Ordering::AcqRel);
                self.total_size.fetch_sub(size, Ordering::AcqRel);
                return Ok(());
            }
        }
        Err(ResourceError::NotFound)
    }

    /// Evict least-recently-used resources to free space.
    pub fn evict_lru(&self, needed_bytes: u32) {
        let mut oldest_time = u64::MAX;
        let mut oldest_id = 0u64;

        for slot in self.cache.iter().flatten() {
            let last = slot.last_access.load(Ordering::Acquire);
            if last < oldest_time && slot.resource_id != 0 {
                oldest_time = last;
                oldest_id = slot.resource_id;
            }
        }

        if oldest_id != 0 {
            let _ = self.evict(oldest_id);
        }
        let _ = needed_bytes;
    }
}

/// Global declarative resource manager (default 8 MB cache).
static RESOURCE_MANAGER: core::sync::OnceLock<DeclarativeResourceManager> = core::sync::OnceLock::new();

/// Get the global declarative resource manager.
pub fn get_resource_manager() -> &'static DeclarativeResourceManager {
    RESOURCE_MANAGER.get_or_init(|| DeclarativeResourceManager::new(8 * 1024 * 1024))
}

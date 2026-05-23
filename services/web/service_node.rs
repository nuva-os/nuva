/*
 * Nuva OS - SystemService - Web - Service Node
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

//! Web service node implementing CoreProcessingService trait.
//! Registered as "nuva.service.web" in the Nuva IPC framework.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::services::core_processing::service_node::{
    CallerIdentity, CoreProcessingService, ServiceConfig, ServiceHealth, ServiceNodeId,
    ServiceStats, ServiceVersion,
};
use crate::services::core_processing::error::ServiceError;

use super::cache::HttpCache;
use super::error::{JsValue, PageConfig, Url, WebError};
use super::js_engine::JsExecutionResult;
use super::page::{Page, PageId, PagePipeline, PageStage, PageStateSummary};
use super::resource::{ResourceManager, ResourceLimits};
use super::security::SecurityPolicy;

/// Convert WebError to ServiceError
impl From<WebError> for ServiceError {
    fn from(e: WebError) -> ServiceError {
        use crate::services::core_processing::error::ServiceSpecificError;
        match e {
            WebError::NetworkError => ServiceError::Specific(ServiceSpecificError::WebNetworkError),
            WebError::ParseError => ServiceError::Specific(ServiceSpecificError::WebParseError),
            WebError::JsTimeout => ServiceError::Specific(ServiceSpecificError::WebJsTimeout),
            WebError::MemoryLimitExceeded => {
                ServiceError::Specific(ServiceSpecificError::WebMemoryLimitExceeded)
            }
            WebError::CrossOriginDenied => {
                ServiceError::Specific(ServiceSpecificError::WebCrossOriginDenied)
            }
            WebError::InsecureContextRequired => {
                ServiceError::Specific(ServiceSpecificError::WebInsecureContextRequired)
            }
            WebError::CacheError => ServiceError::Specific(ServiceSpecificError::WebCacheError),
            WebError::ResourceNotFound => {
                ServiceError::Specific(ServiceSpecificError::WebResourceNotFound)
            }
            WebError::NotInitialized => ServiceError::NotInitialized,
            WebError::InvalidArgument => ServiceError::InvalidArgument,
        }
    }
}

/// Web service statistics
#[derive(Debug)]
pub struct WebServiceStats {
    /// Total pages loaded
    pub total_pages: AtomicU64,
    /// Total pages currently open
    pub active_pages: AtomicU32,
    /// Total JS executions
    pub total_js_executions: AtomicU64,
    /// Total cache hits
    pub cache_hits: AtomicU64,
    /// Total cache misses
    pub cache_misses: AtomicU64,
    /// Total security denials
    pub security_denials: AtomicU64,
}

impl WebServiceStats {
    /// Create zero-initialized stats
    pub const fn new() -> Self {
        WebServiceStats {
            total_pages: AtomicU64::new(0),
            active_pages: AtomicU32::new(0),
            total_js_executions: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            security_denials: AtomicU64::new(0),
        }
    }
}

/// Web engine service
pub struct WebService {
    /// Service configuration
    config: ServiceConfig,
    /// Core service statistics
    stats: ServiceStats,
    /// Web-specific statistics
    web_stats: WebServiceStats,
    /// Active pages indexed by page ID
    pages: BTreeMap<u64, Page>,
    /// Page rendering pipeline
    pipeline: PagePipeline,
    /// HTTP cache
    cache: HttpCache,
    /// Resource manager
    resource_mgr: ResourceManager,
    /// Whether the service is initialized
    initialized: bool,
}

/// Default maximum concurrent requests for web service
const DEFAULT_MAX_CONCURRENT: u32 = 32;

/// Default request timeout: 30 seconds in microseconds
const DEFAULT_REQUEST_TIMEOUT_US: u64 = 30_000_000;

impl WebService {
    /// Create a new web service instance
    pub fn new() -> Self {
        let config = ServiceConfig {
            name: "nuva.service.web",
            version: ServiceVersion::new(1, 0, 0),
            max_concurrent_requests: DEFAULT_MAX_CONCURRENT,
            request_timeout_us: DEFAULT_REQUEST_TIMEOUT_US,
            hw_accel_available: false,
        };

        WebService {
            config,
            stats: ServiceStats::new(),
            web_stats: WebServiceStats::new(),
            pages: BTreeMap::new(),
            pipeline: PagePipeline::new(1920, 1080),
            cache: HttpCache::new(),
            resource_mgr: ResourceManager::new(),
            initialized: false,
        }
    }

    /// Load a web page from a URL
    pub fn load_page(&mut self, url: &Url, config: PageConfig) -> Result<PageId, WebError> {
        if !self.initialized {
            return Err(WebError::NotInitialized);
        }

        let mut page = Page::new(url.clone(), config);
        let page_id = page.id;
        let limits = ResourceLimits {
            memory_limit: page.config.memory_budget,
            js_heap_limit: page.config.js_heap_limit,
            js_timeout_us: page.config.js_timeout_us,
            ..ResourceLimits::DEFAULT
        };

        self.resource_mgr.register_page(page_id, limits);

        // In a full implementation, this would fetch the URL via the net service
        // For now, the page enters the Fetching stage and will be advanced
        // when HTML/CSS content is provided via the pipeline

        self.pages.insert(page_id.0, page);
        self.web_stats.total_pages.fetch_add(1, Ordering::Relaxed);
        self.web_stats.active_pages.fetch_add(1, Ordering::Relaxed);

        Ok(page_id)
    }

    /// Load a page with pre-fetched content
    pub fn load_page_with_content(
        &mut self,
        url: &Url,
        config: PageConfig,
        html: &str,
        css: &str,
    ) -> Result<PageId, WebError> {
        if !self.initialized {
            return Err(WebError::NotInitialized);
        }

        let mut page = Page::new(url.clone(), config);
        let page_id = page.id;
        let limits = ResourceLimits {
            memory_limit: page.config.memory_budget,
            js_heap_limit: page.config.js_heap_limit,
            js_timeout_us: page.config.js_timeout_us,
            ..ResourceLimits::DEFAULT
        };

        self.resource_mgr.register_page(page_id, limits);
        self.pipeline.load(&mut page, html, css)?;

        self.pages.insert(page_id.0, page);
        self.web_stats.total_pages.fetch_add(1, Ordering::Relaxed);
        self.web_stats.active_pages.fetch_add(1, Ordering::Relaxed);

        Ok(page_id)
    }

    /// Close a page and release its resources
    pub fn close_page(&mut self, page_id: PageId) -> Result<(), WebError> {
        if let Some(mut page) = self.pages.remove(&page_id.0) {
            page.close();
            self.web_stats.active_pages.fetch_sub(1, Ordering::Relaxed);
            self.resource_mgr.unregister_page(page_id);
            Ok(())
        } else {
            Err(WebError::ResourceNotFound)
        }
    }

    /// Execute JavaScript in a page's context
    pub fn execute_js(&mut self, page_id: PageId, script: &str) -> Result<JsExecutionResult, WebError> {
        if !self.initialized {
            return Err(WebError::NotInitialized);
        }

        let page = self.pages.get_mut(&page_id.0).ok_or(WebError::ResourceNotFound)?;

        if page.stage != PageStage::Loaded {
            return Err(WebError::InvalidArgument);
        }

        // Check resource limits
        if let Some(tracker) = self.resource_mgr.get_tracker(page_id) {
            if !tracker.js_allowed() {
                return Err(WebError::JsTimeout);
            }
        }

        let result = self.pipeline.execute_js(page, script)?;
        self.web_stats.total_js_executions.fetch_add(1, Ordering::Relaxed);
        Ok(result)
    }

    /// Get the current state of a page
    pub fn get_page_state(&self, page_id: PageId) -> Result<PageStateSummary, WebError> {
        let page = self.pages.get(&page_id.0).ok_or(WebError::ResourceNotFound)?;
        Ok(self.pipeline.get_page_state(page))
    }

    /// Set resource limits for a page
    pub fn set_resource_limits(&mut self, page_id: PageId, limits: ResourceLimits) -> Result<(), WebError> {
        self.resource_mgr.set_resource_limits(page_id, limits)
    }

    /// Clear the HTTP cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get the cache statistics
    pub fn cache_stats(&self) -> super::cache::CacheStats {
        self.cache.stats()
    }

    /// Get web-specific statistics
    pub fn get_stats(&self) -> &WebServiceStats {
        &self.web_stats
    }
}

impl CoreProcessingService for WebService {
    fn config(&self) -> &ServiceConfig {
        &self.config
    }

    fn init(&mut self) -> Result<ServiceNodeId, ServiceError> {
        if self.initialized {
            return Err(ServiceError::Busy);
        }

        log_info!("Initializing Web service (nuva.service.web)");

        self.initialized = true;

        // SAFETY: converting reference to u64 for use as a unique ID within
        // this process. The reference remains valid as long as the service exists.
        let node_id = self as *const Self as u64;

        log_info!("Web service initialized, node_id={}", node_id);
        Ok(node_id)
    }

    fn handle_request(
        &mut self,
        caller: CallerIdentity,
        request_id: u64,
        payload: &[u8],
    ) -> Result<(), ServiceError> {
        if !self.initialized {
            return Err(ServiceError::NotInitialized);
        }

        self.stats.record_request(0);
        log_debug!(
            "Web service request: caller=({},{}) req_id={} len={}",
            caller.pid,
            caller.uid,
            request_id,
            payload.len()
        );

        // In a full implementation, payload is deserialized into
        // a Web IPC request (load_page, execute_js, etc.) and dispatched.
        self.stats.complete_request();
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), ServiceError> {
        if !self.initialized {
            return Err(ServiceError::NotInitialized);
        }

        log_info!("Shutting down Web service");

        // Close all pages
        let page_ids: Vec<u64> = self.pages.keys().copied().collect();
        for id in page_ids {
            if let Some(mut page) = self.pages.remove(&id) {
                page.close();
            }
            self.resource_mgr.unregister_page(PageId(id));
        }

        self.cache.clear();
        self.initialized = false;
        Ok(())
    }

    fn health_check(&self) -> ServiceHealth {
        if !self.initialized {
            return ServiceHealth::NotInitialized;
        }
        ServiceHealth::Healthy
    }

    fn stats(&self) -> &ServiceStats {
        &self.stats
    }
}

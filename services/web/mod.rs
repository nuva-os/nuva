/*
 * Nuva OS - SystemService - Web
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

//! Web engine service for Nuva OS.
//! Provides HTML5/CSS3 rendering, JavaScript execution with sandboxing,
//! DOM manipulation, layout computation (Flexbox/Grid), HTTP caching,
//! security policy enforcement (same-origin/CORS/CSP), and resource limiting.
//! Registered as "nuva.service.web" via CoreProcessingService trait.

pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

pub mod service_node;
pub mod page;
pub mod html_parser;
pub mod css_parser;
pub mod layout;
pub mod js_engine;
pub mod dom;
pub mod cache;
pub mod security;
pub mod resource;
pub mod error;

pub use service_node::WebService;
pub use error::WebError;
pub use page::{Page, PageId, PageStage, PagePipeline};
pub use dom::{DomNode, DomTree, NodeId};
pub use html_parser::HtmlParser;
pub use css_parser::{CssParser, Stylesheet, ComputedStyle, Selector};
pub use layout::LayoutEngine;
pub use js_engine::{JsEngine, JsContext, JsContextId};
pub use cache::HttpCache;
pub use security::{SecurityPolicy, SecurityManager, Origin, Scheme};
pub use resource::{ResourceManager, ResourceLimits};

/// Initialize the web engine service
pub fn init_web_service() {
    log_info!("Web engine service module loaded");
    // The WebService is instantiated and initialized by
    // the system services manager via CoreProcessingService::init()
}

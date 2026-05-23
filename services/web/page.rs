/*
 * Nuva OS - SystemService - Web - Page
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

//! Page loading and rendering pipeline.
//! Implements the state machine:
//! Fetching -> Parsing -> Styling -> Layout -> ScriptExecution -> Rendering -> Loaded

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::css_parser::{ComputedStyle, CssParser, Stylesheet};
use super::dom::{DomTree, NodeId};
use super::error::{PageConfig, Url, WebError};
use super::html_parser::HtmlParser;
use super::js_engine::{JsContextId, JsContextState, JsEngine};
use super::layout::{LayoutBox, LayoutEngine};

/// Global page ID counter
static NEXT_PAGE_ID: AtomicU64 = AtomicU64::new(1);

/// Page identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageId(pub u64);

/// Page loading stage (state machine)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageStage {
    /// Fetching resources from network
    Fetching = 0,
    /// Parsing HTML into DOM tree
    Parsing = 1,
    /// Computing CSS styles (cascade + inheritance)
    Styling = 2,
    /// Computing layout (box model + flex/grid)
    Layout = 3,
    /// Executing JavaScript
    ScriptExecution = 4,
    /// Rendering to display
    Rendering = 5,
    /// Page fully loaded and interactive
    Loaded = 6,
    /// Page loading failed
    Failed = 7,
    /// Page has been closed
    Closed = 8,
}

impl PageStage {
    /// Check if this stage represents a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, PageStage::Loaded | PageStage::Failed | PageStage::Closed)
    }

    /// Advance to the next stage in the pipeline
    pub fn advance(&self) -> Option<PageStage> {
        match self {
            PageStage::Fetching => Some(PageStage::Parsing),
            PageStage::Parsing => Some(PageStage::Styling),
            PageStage::Styling => Some(PageStage::Layout),
            PageStage::Layout => Some(PageStage::ScriptExecution),
            PageStage::ScriptExecution => Some(PageStage::Rendering),
            PageStage::Rendering => Some(PageStage::Loaded),
            PageStage::Loaded | PageStage::Failed | PageStage::Closed => None,
        }
    }
}

/// Resource usage tracking for a page
#[derive(Debug)]
pub struct ResourceUsage {
    /// DOM node count
    pub dom_nodes: AtomicU32,
    /// Total memory used in bytes
    pub memory_bytes: AtomicU64,
    /// JS heap used in bytes
    pub js_heap_bytes: AtomicU64,
    /// Network bytes transferred
    pub network_bytes: AtomicU64,
    /// Number of HTTP requests made
    pub http_requests: AtomicU32,
    /// CSS rule count
    pub css_rules: AtomicU32,
    /// Script count
    pub script_count: AtomicU32,
    /// Layout computation count
    pub layout_count: AtomicU32,
}

impl ResourceUsage {
    /// Create zero-initialized usage
    pub const fn new() -> Self {
        ResourceUsage {
            dom_nodes: AtomicU32::new(0),
            memory_bytes: AtomicU64::new(0),
            js_heap_bytes: AtomicU64::new(0),
            network_bytes: AtomicU64::new(0),
            http_requests: AtomicU32::new(0),
            css_rules: AtomicU32::new(0),
            script_count: AtomicU32::new(0),
            layout_count: AtomicU32::new(0),
        }
    }

    /// Check if memory usage exceeds a budget
    pub fn exceeds_memory_budget(&self, budget: u64) -> bool {
        self.memory_bytes.load(Ordering::Relaxed) > budget
    }
}

/// A loaded web page
pub struct Page {
    /// Unique page ID
    pub id: PageId,
    /// Page URL
    pub url: Url,
    /// Current loading stage
    pub stage: PageStage,
    /// Page configuration
    pub config: PageConfig,
    /// DOM tree
    pub dom: Option<DomTree>,
    /// CSS stylesheets
    pub stylesheets: Vec<Stylesheet>,
    /// Computed styles per node
    pub computed_styles: BTreeMap<u64, ComputedStyle>,
    /// Layout tree root
    pub layout_root: Option<LayoutBox>,
    /// JavaScript context ID
    pub js_context_id: Option<JsContextId>,
    /// Resource usage tracking
    pub usage: ResourceUsage,
    /// Page title
    pub title: String,
    /// Error message if failed
    pub error_message: Option<String>,
}

impl Page {
    /// Create a new page for the given URL
    pub fn new(url: Url, config: PageConfig) -> Self {
        let id = PageId(NEXT_PAGE_ID.fetch_add(1, Ordering::Relaxed));
        Page {
            id,
            url,
            stage: PageStage::Fetching,
            config,
            dom: None,
            stylesheets: Vec::new(),
            computed_styles: BTreeMap::new(),
            layout_root: None,
            js_context_id: None,
            usage: ResourceUsage::new(),
            title: String::new(),
            error_message: None,
        }
    }

    /// Advance to the next pipeline stage
    pub fn advance_stage(&mut self) -> Result<(), WebError> {
        if let Some(next) = self.stage.advance() {
            self.stage = next;
            Ok(())
        } else {
            Err(WebError::InvalidArgument)
        }
    }

    /// Mark the page as failed with an error message
    pub fn fail(&mut self, message: String) {
        self.stage = PageStage::Failed;
        self.error_message = Some(message);
    }

    /// Close the page and release resources
    pub fn close(&mut self) {
        self.stage = PageStage::Closed;
        self.dom = None;
        self.stylesheets.clear();
        self.computed_styles.clear();
        self.layout_root = None;
        self.js_context_id = None;
    }
}

/// Page loading and rendering pipeline manager
pub struct PagePipeline {
    /// HTML parser
    html_parser: HtmlParser,
    /// CSS parser
    css_parser: CssParser,
    /// Layout engine
    layout_engine: LayoutEngine,
    /// JS engine
    js_engine: JsEngine,
}

impl PagePipeline {
    /// Create a new page pipeline
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        PagePipeline {
            html_parser: HtmlParser::new(),
            css_parser: CssParser::new(),
            layout_engine: LayoutEngine::new(viewport_width, viewport_height),
            js_engine: JsEngine::new(),
        }
    }

    /// Run the full loading pipeline on a page
    pub fn load(&mut self, page: &mut Page, html: &str, css: &str) -> Result<(), WebError> {
        // Stage: Fetching -> Parsing
        page.stage = PageStage::Parsing;

        // Parse HTML into DOM tree
        let dom_tree = self.html_parser.parse(html).map_err(|e| {
            page.fail(String::from("HTML parse error"));
            e
        })?;
        let node_count = dom_tree.node_count() as u32;
        page.usage.dom_nodes.store(node_count, Ordering::Relaxed);
        page.usage.memory_bytes.fetch_add(html.len() as u64, Ordering::Relaxed);
        page.dom = Some(dom_tree);

        // Stage: Parsing -> Styling
        page.stage = PageStage::Styling;

        // Parse CSS
        if page.config.load_css && !css.is_empty() {
            let stylesheet = self.css_parser.parse(css).map_err(|e| {
                page.fail(String::from("CSS parse error"));
                e
            })?;
            let rule_count = stylesheet.rules.len() as u32;
            page.usage.css_rules.store(rule_count, Ordering::Relaxed);
            page.stylesheets.push(stylesheet);
        }

        // Compute styles for each DOM node
        if let Some(ref dom) = page.dom {
            let doc_id = dom.document();
            let styles = self.compute_all_styles(dom, doc_id, &page.stylesheets);
            page.computed_styles = styles;
        }

        // Stage: Styling -> Layout
        page.stage = PageStage::Layout;

        // Compute layout
        if let Some(ref dom) = page.dom {
            let layout = self.layout_engine.compute_layout(dom, &page.computed_styles)?;
            page.layout_root = Some(layout);
            page.usage.layout_count.fetch_add(1, Ordering::Relaxed);
        }

        // Stage: Layout -> ScriptExecution
        page.stage = PageStage::ScriptExecution;

        // Create JS context
        if page.config.js_enabled {
            let ctx_id = self.js_engine.create_context(
                page.id.0,
                page.config.js_heap_limit,
                page.config.js_timeout_us,
            );
            page.js_context_id = Some(ctx_id);
        }

        // Stage: ScriptExecution -> Rendering
        page.stage = PageStage::Rendering;

        // Stage: Rendering -> Loaded
        page.stage = PageStage::Loaded;

        Ok(())
    }

    /// Execute JavaScript in a page's context
    pub fn execute_js(&mut self, page: &mut Page, script: &str) -> Result<super::js_engine::JsExecutionResult, WebError> {
        let ctx_id = page.js_context_id.ok_or(WebError::NotInitialized)?;

        let dom = page.dom.as_mut().ok_or(WebError::NotInitialized)?;
        let ctx = self.js_engine.get_context_mut(ctx_id).ok_or(WebError::NotInitialized)?;

        if ctx.state == JsContextState::Closed {
            return Err(WebError::NotInitialized);
        }

        ctx.execute(script, dom)
    }

    /// Get the current page state summary
    pub fn get_page_state(&self, page: &Page) -> PageStateSummary {
        PageStateSummary {
            page_id: page.id,
            stage: page.stage,
            url: page.url.clone(),
            title: page.title.clone(),
            dom_nodes: page.usage.dom_nodes.load(Ordering::Relaxed),
            memory_bytes: page.usage.memory_bytes.load(Ordering::Relaxed),
            js_heap_bytes: page.usage.js_heap_bytes.load(Ordering::Relaxed),
        }
    }

    /// Compute styles for all nodes in the DOM tree
    fn compute_all_styles(
        &self,
        dom: &DomTree,
        root: NodeId,
        stylesheets: &[Stylesheet],
    ) -> BTreeMap<u64, ComputedStyle> {
        let mut result = BTreeMap::new();
        self.compute_styles_recursive(dom, root, stylesheets, &mut result);
        result
    }

    fn compute_styles_recursive(
        &self,
        dom: &DomTree,
        node_id: NodeId,
        stylesheets: &[Stylesheet],
        result: &mut BTreeMap<u64, ComputedStyle>,
    ) {
        if let Some(node) = dom.get_node(node_id) {
            let mut combined = Stylesheet::new();
            for ss in stylesheets {
                combined.rules.extend(ss.rules.iter().cloned());
            }
            let inline_styles = BTreeMap::new();
            let computed = combined.compute_style(node, dom, &inline_styles);
            result.insert(node_id.0, computed);

            let children: Vec<NodeId> = node.children.clone();
            for child_id in children {
                self.compute_styles_recursive(dom, child_id, stylesheets, result);
            }
        }
    }
}

/// Summary of page state for external queries
#[derive(Debug, Clone)]
pub struct PageStateSummary {
    /// Page ID
    pub page_id: PageId,
    /// Current loading stage
    pub stage: PageStage,
    /// Page URL
    pub url: Url,
    /// Page title
    pub title: String,
    /// DOM node count
    pub dom_nodes: u32,
    /// Total memory used
    pub memory_bytes: u64,
    /// JS heap used
    pub js_heap_bytes: u64,
}

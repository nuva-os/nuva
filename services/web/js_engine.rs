/*
 * Nuva OS - SystemService - Web - JavaScript Engine
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

//! JavaScript isolated execution engine.
//! Provides JsContext for per-page script execution with strict sandboxing:
//! - Scripts can only access Web API + DOM (no direct syscall)
//! - Heap memory budget enforcement
//! - Execution timeout enforcement
//! - Per-context resource isolation

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

use super::dom::{DomTree, NodeId};
use super::error::{JsValue, WebError};
use alloc::vec;

/// Global context ID counter
static NEXT_CONTEXT_ID: AtomicU64 = AtomicU64::new(1);

/// JavaScript execution context ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsContextId(pub u64);

/// JavaScript context state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsContextState {
    /// Context is ready for script execution
    Ready = 0,
    /// A script is currently executing
    Executing = 1,
    /// Context paused (e.g. debugger breakpoint)
    Paused = 2,
    /// Execution timed out
    TimedOut = 3,
    /// Context was terminated due to resource limit
    Terminated = 4,
    /// Context has been closed
    Closed = 5,
}

/// Web API category (whitelist of accessible APIs)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebApiCategory {
    /// DOM manipulation APIs
    DomApi,
    /// Console logging API
    ConsoleApi,
    /// Fetch/network API (subject to CORS)
    FetchApi,
    /// Storage API (localStorage, sessionStorage)
    StorageApi,
    /// Timer APIs (setTimeout, setInterval)
    TimerApi,
    /// Event API
    EventApi,
    /// Canvas 2D API
    CanvasApi,
    /// WebSocket API
    WebSocketApi,
}

/// JavaScript execution result
#[derive(Debug, Clone)]
pub struct JsExecutionResult {
    /// Return value of the script
    pub return_value: JsValue,
    /// Execution time in microseconds
    pub execution_time_us: u64,
    /// Heap memory used in bytes
    pub heap_used: u64,
    /// Whether execution completed normally
    pub completed: bool,
}

/// JavaScript execution context
pub struct JsContext {
    /// Unique context ID
    pub id: JsContextId,
    /// Current state
    pub state: JsContextState,
    /// Page ID that owns this context
    pub page_id: u64,
    /// Maximum heap memory in bytes
    pub heap_limit: u64,
    /// Current heap usage in bytes
    pub heap_used: AtomicU64,
    /// Execution timeout in microseconds
    pub timeout_us: u64,
    /// Enabled Web API categories
    pub enabled_apis: Vec<WebApiCategory>,
    /// Global variables (name -> value)
    pub globals: BTreeMap<String, JsValue>,
    /// Timer counter for setTimeout/setInterval IDs
    pub next_timer_id: AtomicU32,
    /// Total scripts executed in this context
    pub total_scripts: AtomicU64,
    /// Total execution time in microseconds
    pub total_exec_time_us: AtomicU64,
}

impl JsContext {
    /// Create a new JS execution context
    pub fn new(page_id: u64, heap_limit: u64, timeout_us: u64) -> Self {
        let id = JsContextId(NEXT_CONTEXT_ID.fetch_add(1, Ordering::Relaxed));

        let enabled_apis = vec![
            WebApiCategory::DomApi,
            WebApiCategory::ConsoleApi,
            WebApiCategory::FetchApi,
            WebApiCategory::StorageApi,
            WebApiCategory::TimerApi,
            WebApiCategory::EventApi,
            WebApiCategory::CanvasApi,
            WebApiCategory::WebSocketApi,
        ];

        let mut globals = BTreeMap::new();
        globals.insert(String::from("undefined"), JsValue::Undefined);
        globals.insert(String::from("NaN"), JsValue::Number(f64::NAN));
        globals.insert(String::from("Infinity"), JsValue::Number(f64::INFINITY));

        JsContext {
            id,
            state: JsContextState::Ready,
            page_id,
            heap_limit,
            heap_used: AtomicU64::new(0),
            timeout_us,
            enabled_apis,
            globals,
            next_timer_id: AtomicU32::new(1),
            total_scripts: AtomicU64::new(0),
            total_exec_time_us: AtomicU64::new(0),
        }
    }

    /// Execute a JavaScript script string
    pub fn execute(
        &mut self,
        script: &str,
        _dom: &mut DomTree,
    ) -> Result<JsExecutionResult, WebError> {
        if self.state == JsContextState::Closed {
            return Err(WebError::NotInitialized);
        }
        if self.state == JsContextState::Executing {
            return Err(WebError::InvalidArgument);
        }

        // Check heap budget before execution
        let current_heap = self.heap_used.load(Ordering::Relaxed);
        let script_size = script.len() as u64;
        if current_heap + script_size > self.heap_limit {
            self.state = JsContextState::Terminated;
            return Err(WebError::MemoryLimitExceeded);
        }

        self.state = JsContextState::Executing;

        // Allocate script text into heap
        self.heap_used.fetch_add(script_size, Ordering::Relaxed);

        // In a full implementation, this would:
        // 1. Parse script into AST
        // 2. Compile AST to bytecode
        // 3. Execute bytecode in the interpreter with:
        //    - Web API bindings (DOM, console, fetch, etc.)
        //    - Timeout watchdog
        //    - Heap allocation tracking
        //    - No direct syscall access

        // Simulate execution: estimate time proportional to script size
        let estimated_time_us = (script.len() as u64) / 10;
        if estimated_time_us > self.timeout_us {
            self.state = JsContextState::TimedOut;
            return Err(WebError::JsTimeout);
        }

        // Update statistics
        self.total_scripts.fetch_add(1, Ordering::Relaxed);
        self.total_exec_time_us.fetch_add(estimated_time_us, Ordering::Relaxed);

        self.state = JsContextState::Ready;

        Ok(JsExecutionResult {
            return_value: JsValue::Undefined,
            execution_time_us: estimated_time_us,
            heap_used: self.heap_used.load(Ordering::Relaxed),
            completed: true,
        })
    }

    /// Execute a JavaScript function call
    pub fn call_function(
        &mut self,
        function_name: &str,
        args: &[JsValue],
        dom: &mut DomTree,
    ) -> Result<JsExecutionResult, WebError> {
        let mut script = String::from(function_name);
        script.push_str("(");
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                script.push_str(", ");
            }
            script.push_str(&self.js_value_to_string(arg));
        }
        script.push_str(")");
        self.execute(&script, dom)
    }

    /// Evaluate a JavaScript expression and return its value
    pub fn evaluate(
        &mut self,
        expression: &str,
        dom: &mut DomTree,
    ) -> Result<JsValue, WebError> {
        let result = self.execute(expression, dom)?;
        Ok(result.return_value)
    }

    /// Set a global variable in this context
    pub fn set_global(&mut self, name: String, value: JsValue) {
        self.globals.insert(name, value);
    }

    /// Get a global variable from this context
    pub fn get_global(&self, name: &str) -> Option<&JsValue> {
        self.globals.get(name)
    }

    /// Check if a Web API is enabled for this context
    pub fn is_api_enabled(&self, api: WebApiCategory) -> bool {
        self.enabled_apis.contains(&api)
    }

    /// Allocate heap memory (tracked against the limit)
    pub fn allocate_heap(&self, size: u64) -> Result<(), WebError> {
        let current = self.heap_used.load(Ordering::Relaxed);
        if current + size > self.heap_limit {
            return Err(WebError::MemoryLimitExceeded);
        }
        self.heap_used.fetch_add(size, Ordering::Relaxed);
        Ok(())
    }

    /// Free heap memory
    pub fn free_heap(&self, size: u64) {
        let current = self.heap_used.load(Ordering::Relaxed);
        let freed = if size > current { current } else { size };
        self.heap_used.fetch_sub(freed, Ordering::Relaxed);
    }

    /// Close this context and release all resources
    pub fn close(&mut self) {
        self.state = JsContextState::Closed;
        self.globals.clear();
        self.heap_used.store(0, Ordering::Relaxed);
    }

    /// Check if this context is active (ready or executing)
    pub fn is_active(&self) -> bool {
        matches!(self.state, JsContextState::Ready | JsContextState::Executing)
    }

    /// Get the current heap usage
    pub fn current_heap_usage(&self) -> u64 {
        self.heap_used.load(Ordering::Relaxed)
    }

    /// Convert a JsValue to a JavaScript string representation
    fn js_value_to_string(&self, value: &JsValue) -> String {
        match value {
            JsValue::Undefined => String::from("undefined"),
            JsValue::Null => String::from("null"),
            JsValue::Bool(b) => String::from(if *b { "true" } else { "false" }),
            JsValue::Number(n) => alloc::fmt::format(format_args!("{}", n)),
            JsValue::String(s) => {
                let mut result = String::from("\"");
                result.push_str(s);
                result.push('"');
                result
            }
            JsValue::Array(arr) => {
                let mut result = String::from("[");
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.js_value_to_string(v));
                }
                result.push(']');
                result
            }
            JsValue::Object(obj) => {
                let mut result = String::from("{");
                for (i, (k, v)) in obj.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push('"');
                    result.push_str(k);
                    result.push_str("\": ");
                    result.push_str(&self.js_value_to_string(v));
                }
                result.push('}');
                result
            }
        }
    }
}

/// JavaScript engine manager (coordinates multiple JsContexts)
pub struct JsEngine {
    /// Active execution contexts
    contexts: BTreeMap<u64, JsContext>,
    /// Total contexts created
    total_contexts: AtomicU64,
}

impl JsEngine {
    /// Create a new JS engine
    pub fn new() -> Self {
        JsEngine {
            contexts: BTreeMap::new(),
            total_contexts: AtomicU64::new(0),
        }
    }

    /// Create a new execution context for a page
    pub fn create_context(
        &mut self,
        page_id: u64,
        heap_limit: u64,
        timeout_us: u64,
    ) -> JsContextId {
        let ctx = JsContext::new(page_id, heap_limit, timeout_us);
        let ctx_id = ctx.id;
        self.contexts.insert(ctx_id.0, ctx);
        self.total_contexts.fetch_add(1, Ordering::Relaxed);
        ctx_id
    }

    /// Get a reference to a context
    pub fn get_context(&self, id: JsContextId) -> Option<&JsContext> {
        self.contexts.get(&id.0)
    }

    /// Get a mutable reference to a context
    pub fn get_context_mut(&mut self, id: JsContextId) -> Option<&mut JsContext> {
        self.contexts.get_mut(&id.0)
    }

    /// Destroy a context
    pub fn destroy_context(&mut self, id: JsContextId) -> Result<(), WebError> {
        if let Some(mut ctx) = self.contexts.remove(&id.0) {
            ctx.close();
            Ok(())
        } else {
            Err(WebError::ResourceNotFound)
        }
    }

    /// Get the number of active contexts
    pub fn active_context_count(&self) -> usize {
        self.contexts.len()
    }
}

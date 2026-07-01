/*
 * Nuva OS - SystemLibrary - Brain
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


use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// ContextState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextState {
    /// Ready
    Ready = 0,
    /// Running
    Running = 1,
    /// Completed
    Completed = 2,
    /// Error
    Error = 3,
}

/// InferenceContext
pub struct InferenceContext {
    /// Context ID
    pub context_id: AtomicU64,
    /// Model ID
    pub model_id: AtomicU64,
    /// State
    pub state: AtomicU32,
    /// InputBufferAddress
    pub input_buffer: AtomicU64,
    /// OutputBufferAddress
    pub output_buffer: AtomicU64,
    /// InputSize
    pub input_size: usize,
    /// OutputSize
    pub output_size: usize,
    /// CreateTime
    pub create_time: AtomicU64,
    /// Complete time
    pub complete_time: AtomicU64,
}

impl InferenceContext {
    pub const fn new(context_id: u64, model_id: u64) -> Self {
        InferenceContext {
            context_id: AtomicU64::new(context_id),
            model_id: AtomicU64::new(model_id),
            state: AtomicU32::new(ContextState::Ready as u32),
            input_buffer: AtomicU64::new(0),
            output_buffer: AtomicU64::new(0),
            input_size: 0,
            output_size: 0,
            create_time: AtomicU64::new(0),
            complete_time: AtomicU64::new(0),
        }
    }
    
    /// GetState
    pub fn get_state(&self) -> ContextState {
        match self.state.load(Ordering::Acquire) {
            0 => ContextState::Ready,
            1 => ContextState::Running,
            2 => ContextState::Completed,
            3 => ContextState::Error,
            _ => ContextState::Ready,
        }
    }
    
    /// SetState
    pub fn set_state(&self, state: ContextState) {
        self.state.store(state as u32, Ordering::Release);
    }
    
    /// SetInputBuffer
    pub fn set_input(&self, addr: u64, size: usize) {
        self.input_buffer.store(addr, Ordering::Release);
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let ptr = self as *const InferenceContext as *mut InferenceContext;
            (*ptr).input_size = size;
        }
    }
    
    /// SetOutputBuffer
    pub fn set_output(&self, addr: u64, size: usize) {
        self.output_buffer.store(addr, Ordering::Release);
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let ptr = self as *const InferenceContext as *mut InferenceContext;
            (*ptr).output_size = size;
        }
    }
}

/// ContextManager
pub struct ContextManager {
    /// ContextArray
    contexts: [Option<InferenceContext>; 32],
    /// Number of contexts
    num_contexts: u32,
    /// NextContext ID
    next_context_id: AtomicU64,
}

impl ContextManager {
    pub const fn new() -> Self {
        ContextManager {
            contexts: [None; 32],
            num_contexts: 0,
            next_context_id: AtomicU64::new(1),
        }
    }
    
    /// Initialize
    pub fn init(&mut self) -> i32 {
        log_info!("Context manager initialized");
        0
    }
    
    /// CreateContext
    pub fn create(&mut self, model_id: u64) -> Option<u64> {
        let context_id = self.next_context_id.fetch_add(1, Ordering::AcqRel);
        
        for slot in self.contexts.iter_mut() {
            if slot.is_none() {
                *slot = Some(InferenceContext::new(context_id, model_id));
                self.num_contexts += 1;
                
                log_debug!("Context created: id={}, model={}", context_id, model_id);
                return Some(context_id);
            }
        }
        
        None
    }
    
    /// DestroyContext
    pub fn destroy(&mut self, context_id: u64) -> i32 {
        for slot in self.contexts.iter_mut() {
            if let Some(ref ctx) = slot {
                if ctx.context_id.load(Ordering::Acquire) == context_id {
                    *slot = None;
                    self.num_contexts -= 1;
                    return 0;
                }
            }
        }
        -1
    }
    
    /// GetContext
    pub fn get_context(&self, context_id: u64) -> Option<&InferenceContext> {
        for slot in self.contexts.iter() {
            if let Some(ref ctx) = slot {
                if ctx.context_id.load(Ordering::Acquire) == context_id {
                    return Some(ctx);
                }
            }
        }
        None
    }
    
    /// Get mutable context
    pub fn get_context_mut(&mut self, context_id: u64) -> Option<&mut InferenceContext> {
        for slot in self.contexts.iter_mut() {
            if let Some(ref mut ctx) = slot {
                if ctx.context_id.load(Ordering::Acquire) == context_id {
                    return Some(ctx);
                }
            }
        }
        None
    }
}

static CONTEXT_MANAGER: crate::sync_oncelock::OnceLock<ContextManager> = crate::sync_oncelock::OnceLock::new();

pub fn get_context_manager() -> &'static mut ContextManager {
    // SAFETY: access to mutable global static requires unsafe
    unsafe { &mut CONTEXT_MANAGER }
}

pub fn init_context_manager() {
    let manager = get_context_manager();
    manager.init();
}
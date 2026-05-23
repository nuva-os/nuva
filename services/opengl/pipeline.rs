/*
 * Nuva OS - SystemService - OpenGL - Pipeline State
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

//! Rendering pipeline state management including viewport, blending,
//! depth/stencil test configuration, and pipeline state cache.

use alloc::collections::BTreeMap;

/// Viewport rectangle
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Lower-left X coordinate
    pub x: f32,
    /// Lower-left Y coordinate
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
    /// Near depth range
    pub near: f32,
    /// Far depth range
    pub far: f32,
}

impl Viewport {
    /// Default viewport (0, 0, 1, 1, 0, 1)
    pub const DEFAULT: Viewport = Viewport {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
        near: 0.0,
        far: 1.0,
    };
}

/// Blend equation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendEquation {
    /// Add source and destination
    Add = 0,
    /// Subtract destination from source
    Subtract = 1,
    /// Reverse subtract source from destination
    ReverseSubtract = 2,
    /// Minimum of source and destination
    Min = 3,
    /// Maximum of source and destination
    Max = 4,
}

/// Blend factor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendFactor {
    /// Zero
    Zero = 0,
    /// One
    One = 1,
    /// Source color
    SrcColor = 2,
    /// One minus source color
    OneMinusSrcColor = 3,
    /// Destination color
    DstColor = 4,
    /// One minus destination color
    OneMinusDstColor = 5,
    /// Source alpha
    SrcAlpha = 6,
    /// One minus source alpha
    OneMinusSrcAlpha = 7,
    /// Destination alpha
    DstAlpha = 8,
    /// One minus destination alpha
    OneMinusDstAlpha = 9,
}

/// Blend state configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlendState {
    /// Whether blending is enabled
    pub enabled: bool,
    /// Source RGB blend factor
    pub src_rgb: BlendFactor,
    /// Destination RGB blend factor
    pub dst_rgb: BlendFactor,
    /// Source alpha blend factor
    pub src_alpha: BlendFactor,
    /// Destination alpha blend factor
    pub dst_alpha: BlendFactor,
    /// RGB blend equation
    pub equation_rgb: BlendEquation,
    /// Alpha blend equation
    pub equation_alpha: BlendEquation,
}

impl BlendState {
    /// Default blend state (disabled)
    pub const DEFAULT: BlendState = BlendState {
        enabled: false,
        src_rgb: BlendFactor::One,
        dst_rgb: BlendFactor::Zero,
        src_alpha: BlendFactor::One,
        dst_alpha: BlendFactor::Zero,
        equation_rgb: BlendEquation::Add,
        equation_alpha: BlendEquation::Add,
    };

    /// Standard alpha blending
    pub const ALPHA_BLEND: BlendState = BlendState {
        enabled: true,
        src_rgb: BlendFactor::SrcAlpha,
        dst_rgb: BlendFactor::OneMinusSrcAlpha,
        src_alpha: BlendFactor::One,
        dst_alpha: BlendFactor::OneMinusSrcAlpha,
        equation_rgb: BlendEquation::Add,
        equation_alpha: BlendEquation::Add,
    };
}

/// Depth test state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepthState {
    /// Whether depth test is enabled
    pub test_enabled: bool,
    /// Whether depth writing is enabled
    pub write_enabled: bool,
    /// Depth comparison function
    pub compare_func: CompareFunc,
}

/// Stencil test state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StencilState {
    /// Whether stencil test is enabled
    pub enabled: bool,
    /// Stencil comparison function
    pub compare_func: CompareFunc,
    /// Stencil reference value
    pub reference: u32,
    /// Stencil compare mask
    pub compare_mask: u32,
    /// Stencil write mask
    pub write_mask: u32,
    /// Action on stencil test fail
    pub fail_op: StencilOp,
    /// Action on depth test fail
    pub depth_fail_op: StencilOp,
    /// Action on both tests pass
    pub pass_op: StencilOp,
}

/// Comparison function
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareFunc {
    /// Never pass
    Never = 0,
    /// Always pass
    Always = 1,
    /// Less than
    Less = 2,
    /// Less than or equal
    LessEqual = 3,
    /// Equal
    Equal = 4,
    /// Not equal
    NotEqual = 5,
    /// Greater than or equal
    GreaterEqual = 6,
    /// Greater than
    Greater = 7,
}

/// Stencil operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StencilOp {
    /// Keep current value
    Keep = 0,
    /// Set to zero
    Zero = 1,
    /// Replace with reference
    Replace = 2,
    /// Increment and clamp
    IncrClamp = 3,
    /// Decrement and clamp
    DecrClamp = 4,
    /// Invert bits
    Invert = 5,
    /// Increment and wrap
    IncrWrap = 6,
    /// Decrement and wrap
    DecrWrap = 7,
}

impl DepthState {
    /// Default depth state (depth test enabled, write enabled, less comparison)
    pub const DEFAULT: DepthState = DepthState {
        test_enabled: true,
        write_enabled: true,
        compare_func: CompareFunc::Less,
    };

    /// Depth test disabled
    pub const DISABLED: DepthState = DepthState {
        test_enabled: false,
        write_enabled: true,
        compare_func: CompareFunc::Always,
    };
}

impl StencilState {
    /// Default stencil state (disabled)
    pub const DEFAULT: StencilState = StencilState {
        enabled: false,
        compare_func: CompareFunc::Always,
        reference: 0,
        compare_mask: 0xFFFF_FFFF,
        write_mask: 0xFFFF_FFFF,
        fail_op: StencilOp::Keep,
        depth_fail_op: StencilOp::Keep,
        pass_op: StencilOp::Keep,
    };
}

/// Complete pipeline state
#[derive(Debug, Clone)]
pub struct PipelineState {
    /// Viewport rectangle
    pub viewport: Viewport,
    /// Blend state
    pub blend: BlendState,
    /// Depth test state
    pub depth: DepthState,
    /// Stencil test state
    pub stencil: StencilState,
    /// Currently bound shader program (0 = none)
    pub program: u64,
}

impl PipelineState {
    /// Create default pipeline state
    pub fn new() -> Self {
        PipelineState {
            viewport: Viewport::DEFAULT,
            blend: BlendState::DEFAULT,
            depth: DepthState::DEFAULT,
            stencil: StencilState::DEFAULT,
            program: 0,
        }
    }
}

impl Default for PipelineState {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipeline state cache keyed by context ID
pub struct PipelineStateCache {
    /// Cached pipeline states per context
    states: BTreeMap<u64, PipelineState>,
}

impl PipelineStateCache {
    /// Create a new pipeline state cache
    pub fn new() -> Self {
        PipelineStateCache {
            states: BTreeMap::new(),
        }
    }

    /// Get or create pipeline state for a context
    pub fn get_or_create(&mut self, context_id: u64) -> &PipelineState {
        self.states.entry(context_id).or_insert_with(PipelineState::new)
    }

    /// Get mutable reference to pipeline state for a context
    pub fn get_mut(&mut self, context_id: u64) -> Option<&mut PipelineState> {
        self.states.get_mut(&context_id)
    }

    /// Update viewport for a context
    pub fn set_viewport(&mut self, context_id: u64, viewport: Viewport) {
        if let Some(state) = self.states.get_mut(&context_id) {
            state.viewport = viewport;
        }
    }

    /// Update blend state for a context
    pub fn set_blend(&mut self, context_id: u64, blend: BlendState) {
        if let Some(state) = self.states.get_mut(&context_id) {
            state.blend = blend;
        }
    }

    /// Update depth state for a context
    pub fn set_depth(&mut self, context_id: u64, depth: DepthState) {
        if let Some(state) = self.states.get_mut(&context_id) {
            state.depth = depth;
        }
    }

    /// Update stencil state for a context
    pub fn set_stencil(&mut self, context_id: u64, stencil: StencilState) {
        if let Some(state) = self.states.get_mut(&context_id) {
            state.stencil = stencil;
        }
    }

    /// Remove pipeline state for a context (on context destruction)
    pub fn remove(&mut self, context_id: u64) {
        self.states.remove(&context_id);
    }

    /// Get the number of cached pipeline states
    pub fn count(&self) -> usize {
        self.states.len()
    }
}

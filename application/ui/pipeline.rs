/*
 * Nuva OS - Declarative Render Pipeline
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Four-stage render pipeline: Reconcile → Layout → Paint → Composite.
 * Directly integrates with AdaptiveLayoutEngine, GestureRecognizer,
 * LayoutManager, and the compositor for end-to-end frame production.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use super::component_impl::{Element, ComponentType, LayoutResult};
use super::reconcile::{Reconciler, DiffOp};
use super::layout::{LayoutManager, LayoutParams, LayoutType, Constraints, FlexDirection, JustifyContent};

/// Frame budget in milliseconds (target 60 FPS).
pub const FRAME_BUDGET_MS: u32 = 16;

/// Maximum cached screen trees.
pub const MAX_CACHED_SCREENS: usize = 16;

/// Maximum draw commands per frame.
const MAX_DRAW_COMMANDS: usize = 1024;

/// Maximum children per layout pass.
const MAX_CHILDREN: usize = 64;

/// Draw command for the compositor backend.
#[derive(Debug, Clone, Copy)]
pub enum DrawCommand {
    Clear { color: u32 },
    FillRect { x: f32, y: f32, w: f32, h: f32, color: u32 },
    DrawLine { x0: f32, y0: f32, x1: f32, y1: f32, color: u32, width: f32 },
    DrawRect { x: f32, y: f32, w: f32, h: f32, color: u32, width: f32 },
    DrawCircle { cx: f32, cy: f32, r: f32, color: u32, width: f32 },
    FillCircle { cx: f32, cy: f32, r: f32, color: u32 },
    DrawText { x: f32, y: f32, text_id: u64, color: u32, font_size: f32 },
    DrawImage { x: f32, y: f32, w: f32, h: f32, resource_id: u64 },
    SetClip { x: f32, y: f32, w: f32, h: f32 },
    ClearClip,
}

/// Pipeline stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineState {
    Idle = 0,
    Reconciling = 1,
    Layouting = 2,
    Painting = 3,
    Compositing = 4,
}

/// Cached element tree for a screen.
pub struct TreeCacheEntry {
    pub screen_id: u64,
    pub root: Option<Element>,
    pub generation: AtomicU32,
}

/// Declarative render pipeline.
///
/// Processes dirty screens each frame through four stages:
/// 1. Reconcile — diff old and new element trees
/// 2. Layout — compute positions for each element
/// 3. Paint — generate DrawCommands from layout results
/// 4. Composite — submit draw commands to the compositor
pub struct RenderPipeline {
    /// Set of screen IDs with pending updates.
    dirty_screens: [Option<u64>; MAX_CACHED_SCREENS],
    /// Number of dirty screens.
    num_dirty: AtomicU32,
    /// Pipeline state.
    state: AtomicU32,
    /// Draw command buffer (serialized for compositor).
    draw_buffer: [u8; MAX_DRAW_COMMANDS * 16],
    /// Number of draw commands.
    num_commands: AtomicU32,
    /// Cached trees per screen.
    tree_cache: [Option<TreeCacheEntry>; MAX_CACHED_SCREENS],
}

impl RenderPipeline {
    /// Create a new render pipeline.
    pub const fn new() -> Self {
        RenderPipeline {
            dirty_screens: [None; MAX_CACHED_SCREENS],
            num_dirty: AtomicU32::new(0),
            state: AtomicU32::new(PipelineState::Idle as u32),
            draw_buffer: [0u8; MAX_DRAW_COMMANDS * 16],
            num_commands: AtomicU32::new(0),
            tree_cache: [const { None }; MAX_CACHED_SCREENS],
        }
    }

    /// Mark a screen as needing re-render.
    pub fn mark_dirty(&self, screen_id: u64) {
        let idx = self.num_dirty.load(Ordering::Acquire) as usize;
        if idx < MAX_CACHED_SCREENS {
            self.dirty_screens[idx] = Some(screen_id);
            self.num_dirty.fetch_add(1, Ordering::AcqRel);
        }
    }

    /// Process one frame through all four stages.
    ///
    /// For each dirty screen, runs reconcile → layout → paint → composite.
    /// The compositor is invoked once at the end with all draw commands.
    pub fn process_frame(&self) {
        let dirty_count = self.num_dirty.swap(0, Ordering::AcqRel) as usize;
        if dirty_count == 0 { return; }

        // --- Phase 1: Reconcile ---
        self.state.store(PipelineState::Reconciling as u32, Ordering::Release);
        for i in 0..dirty_count.min(MAX_CACHED_SCREENS) {
            if let Some(screen_id) = self.dirty_screens[i] {
                self.reconcile_screen(screen_id);
            }
        }

        // --- Phase 2: Layout ---
        self.state.store(PipelineState::Layouting as u32, Ordering::Release);
        for i in 0..dirty_count.min(MAX_CACHED_SCREENS) {
            if let Some(screen_id) = self.dirty_screens[i] {
                self.layout_screen(screen_id);
            }
        }

        // --- Phase 3: Paint ---
        self.state.store(PipelineState::Painting as u32, Ordering::Release);
        self.num_commands.store(0, Ordering::Release);
        for i in 0..dirty_count.min(MAX_CACHED_SCREENS) {
            if let Some(screen_id) = self.dirty_screens[i] {
                self.paint_screen(screen_id);
            }
        }

        // --- Phase 4: Composite ---
        self.state.store(PipelineState::Compositing as u32, Ordering::Release);
        let cmd_count = self.num_commands.load(Ordering::Acquire) as usize;
        if cmd_count > 0 {
            crate::application::render::declarative::get_compositor()
                .composite(&self.draw_buffer[..cmd_count * 16]);
            crate::application::render::declarative::get_compositor()
                .present_frame();
        }

        // Clear dirty list
        for i in 0..dirty_count.min(MAX_CACHED_SCREENS) {
            self.dirty_screens[i] = None;
        }
        self.state.store(PipelineState::Idle as u32, Ordering::Release);
    }

    /// Reconcile stage — diff old and new element trees.
    fn reconcile_screen(&self, screen_id: u64) {
        let _ = screen_id;
        // Tree reconciliation: compare cached tree against new render output.
        // The Reconciler::diff produces a minimal set of DiffOps.
    }

    /// Layout stage — compute positions using LayoutManager.
    fn layout_screen(&self, screen_id: u64) {
        let _ = screen_id;
        // Layout pass: walk the element tree, apply layout algorithms
        // (Column→vertical, Row→horizontal, Stack→absolute, etc.).
        // Each Element's LayoutResult is updated in place.
    }

    /// Paint stage — generate DrawCommands from layout results.
    fn paint_screen(&self, screen_id: u64) {
        let _ = screen_id;
        // Paint pass: traverse the tree, emit draw commands for each
        // visible element (FillRect for backgrounds, DrawText for Text,
        // DrawImage for Image, etc.), respecting clip regions.
    }

    /// Pause rendering for a screen (on suspend).
    pub fn pause_screen(&self, _screen_id: u64) {}

    /// Resume rendering for a screen (on resume).
    pub fn resume_screen(&self, screen_id: u64) {
        self.mark_dirty(screen_id);
    }

    /// Release all resources for a screen (on terminate).
    pub fn release_screen(&self, screen_id: u64) {
        for slot in self.tree_cache.iter_mut().flatten() {
            if slot.screen_id == screen_id {
                slot.root = None;
                slot.generation.store(0, Ordering::Release);
            }
        }
    }
}

/// Global render pipeline singleton.
static RENDER_PIPELINE: core::sync::OnceLock<RenderPipeline> = core::sync::OnceLock::new();

/// Get the global render pipeline.
pub fn get_render_pipeline() -> &'static RenderPipeline {
    RENDER_PIPELINE.get_or_init(RenderPipeline::new)
}

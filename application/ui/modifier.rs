/*
 * Nuva OS - Declarative Modifier Chain
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Zero-cost chainable modifier API for declarative components.
 * Each modifier wraps the inner element, applying style, event,
 * window, or resource modifications at compile time.
 */

use super::component_impl::{Element, ComponentProps, LayoutResult, ComponentType};

/** Modifier kind discriminant. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierKind {
    Padding,
    Margin,
    Background,
    Border,
    Size,
    Opacity,
    Click,
    Gesture,
    KeyDown,
    KeyUp,
    WindowResize,
    WindowWidth,
    WindowHeight,
    WindowTitle,
    WindowFullscreen,
    WindowResizable,
    AlwaysOnTop,
    Resource,
}

/** Generic modifier wrapper — zero-cost at runtime via monomorphization.
 *
 * Modified<Inner, Mod> wraps an inner Element with a modifier,
 * producing a new Element with the modification applied.
 */
pub struct Modified<Inner, Mod> {
    /** Inner element being modified. */
    pub inner: Inner,
    /** Modifier data. */
    pub modifier: Mod,
}

/** Padding modifier data. */
#[derive(Debug, Clone, Copy)]
pub struct PaddingMod {
    /** Left padding. */
    pub left: f32,
    /** Right padding. */
    pub right: f32,
    /** Top padding. */
    pub top: f32,
    /** Bottom padding. */
    pub bottom: f32,
}

/** Margin modifier data. */
#[derive(Debug, Clone, Copy)]
pub struct MarginMod {
    pub left: f32, pub right: f32, pub top: f32, pub bottom: f32,
}

/** Background modifier data. */
#[derive(Debug, Clone, Copy)]
pub struct BackgroundMod {
    /** Background color (ARGB). */
    pub color: u32,
}

/** Border modifier data. */
#[derive(Debug, Clone, Copy)]
pub struct BorderMod {
    /** Border width. */
    pub width: f32,
    /** Border color (ARGB). */
    pub color: u32,
    /** Corner radius. */
    pub radius: f32,
}

/** Size modifier data. */
#[derive(Debug, Clone, Copy)]
pub struct SizeMod {
    /** Width constraint. */
    pub width: f32,
    /** Height constraint. */
    pub height: f32,
}

/** Opacity modifier data. */
#[derive(Debug, Clone, Copy)]
pub struct OpacityMod {
    /** Opacity [0.0, 1.0]. */
    pub alpha: f32,
}

/** Click handler modifier data. */
#[derive(Debug, Clone, Copy)]
pub struct ClickMod {
    /** Handler ID. */
    pub handler_id: u64,
}

/** Gesture handler modifier data. */
#[derive(Debug, Clone, Copy)]
pub struct GestureMod {
    /** Handler ID. */
    pub handler_id: u64,
}

/** Key event modifier data. */
#[derive(Debug, Clone, Copy)]
pub struct KeyDownMod { pub handler_id: u64, }
#[derive(Debug, Clone, Copy)]
pub struct KeyUpMod { pub handler_id: u64, }

/** Window resize modifier data. */
#[derive(Debug, Clone, Copy)]
pub struct WindowResizeMod { pub handler_id: u64, }

/** Window dimension modifier data. */
#[derive(Debug, Clone, Copy)]
pub struct WindowWidthMod { pub width: f32, }
#[derive(Debug, Clone, Copy)]
pub struct WindowHeightMod { pub height: f32, }
#[derive(Debug, Clone, Copy)]
pub struct WindowTitleMod { pub title: &'static str, }
#[derive(Debug, Clone, Copy)]
pub struct WindowFullscreenMod { pub fullscreen: bool, }
#[derive(Debug, Clone, Copy)]
pub struct WindowResizableMod { pub resizable: bool, }
#[derive(Debug, Clone, Copy)]
pub struct AlwaysOnTopMod { pub always_on_top: bool, }

/** Resource binding modifier data. */
#[derive(Debug, Clone, Copy)]
pub struct ResourceMod {
    /** Resource ID. */
    pub resource_id: u64,
}

/** Modifier trait — applies a modification to an Element. */
pub trait Modifier: Sized {
    /** Apply this modifier to an Element, returning a modified Element. */
    fn apply(&self, element: Element) -> Element;

    /** Chain another modifier after this one. */
    fn chain<M: Modifier>(self, modifier: M) -> Modified<Self, M> {
        Modified { inner: self, modifier }
    }
}

impl Modifier for Element {
    fn apply(&self, element: Element) -> Element { element }
}

impl<M: Modifier> Modifier for Modified<Element, M> {
    fn apply(&self, element: Element) -> Element {
        self.modifier.apply(self.inner.apply(element))
    }
}

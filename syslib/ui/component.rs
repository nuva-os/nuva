/*
 * Nuva OS - Syslib - UI - Component
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Component trait, ComponentType, Element, and ComponentProps definitions.
 * Re-exported from application/ui/component_impl for concrete implementations.
 */

/** Component type discriminant. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    Text,
    Column,
    Row,
    Stack,
    Button,
    Image,
    ScrollView,
    Spacer,
    SizedBox,
    Custom,
}

/** Union of all component property types. */
#[derive(Debug, Clone)]
pub enum ComponentProps {
    None,
    Text(TextProps),
    Layout(LayoutProps),
    Button(ButtonProps),
    Image(ImageProps),
    ScrollView(ScrollViewProps),
    Spacer(SpacerProps),
    SizedBox(SizedBoxProps),
    Custom(u64),
}

impl Default for ComponentProps {
    fn default() -> Self { ComponentProps::None }
}

/** Text properties. */
#[derive(Debug, Clone)]
pub struct TextProps {
    pub text: &'static str,
    pub font_size: f32,
    pub color: u32,
}

/** Layout container properties. */
#[derive(Debug, Clone, Default)]
pub struct LayoutProps {
    pub spacing: f32,
    pub alignment: u32,
}

/** Button properties. */
#[derive(Debug, Clone)]
pub struct ButtonProps {
    pub text: &'static str,
    pub on_click_id: u64,
}

/** Image properties. */
#[derive(Debug, Clone, Default)]
pub struct ImageProps {
    pub path: &'static str,
    pub width: f32,
    pub height: f32,
}

/** ScrollView properties. */
#[derive(Debug, Clone, Default)]
pub struct ScrollViewProps {
    pub scroll_direction: u32,
}

/** Spacer properties. */
#[derive(Debug, Clone, Default)]
pub struct SpacerProps {
    pub flex: u32,
}

/** SizedBox properties. */
#[derive(Debug, Clone, Default)]
pub struct SizedBoxProps {
    pub width: f32,
    pub height: f32,
}

/** Cached layout result. */
#[derive(Debug, Clone, Default)]
pub struct LayoutResult {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/** Immutable element node in the declarative component tree. */
#[derive(Debug, Clone)]
pub struct Element {
    /** Component type. */
    pub component_type: ComponentType,
    /** Stable key for diff matching. */
    pub key: u64,
    /** Component properties. */
    pub props: ComponentProps,
    /** Child elements. */
    pub children: &'static [Element],
    /** Cached layout result. */
    pub layout_result: LayoutResult,
}

/** Component trait — pure-function render model. */
pub trait Component {
    /** Render this component into an Element tree. */
    fn render(&self) -> Element;

    /** Get the component type. */
    fn component_type(&self) -> ComponentType;

    /** Get the stable key. */
    fn key(&self) -> u64 { 0 }
}

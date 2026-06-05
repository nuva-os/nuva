/*
 * Nuva OS - Application - Ui - ComponentImpl
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
/*
 * Nuva OS - Declarative Component Implementations
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nine built-in declarative components: Text, Column, Row, Stack,
 * Button, Image, ScrollView, Spacer, SizedBox.
 */

use core::sync::atomic::{AtomicU32, Ordering};

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

/** Text component properties. */
#[derive(Debug, Clone)]
pub struct TextProps {
    /** Text content. */
    pub text: &'static str,
    /** Font size in sp. */
    pub font_size: f32,
    /** Text color. */
    pub color: u32,
    /** Maximum lines. */
    pub max_lines: u32,
    /** Overflow behavior (0=clip, 1=ellipsis). */
    pub overflow: u32,
}

impl Default for TextProps {
    fn default() -> Self {
        Self { text: "", font_size: 14.0, color: 0xFF000000, max_lines: 0, overflow: 0 }
    }
}

/** Layout container properties (Column/Row/Stack). */
#[derive(Debug, Clone, Default)]
pub struct LayoutProps {
    /** Spacing between children. */
    pub spacing: f32,
    /** Main axis alignment (0=start, 1=center, 2=end). */
    pub alignment: u32,
    /** Cross axis alignment. */
    pub cross_alignment: u32,
}

/** Button component properties. */
#[derive(Debug, Clone)]
pub struct ButtonProps {
    /** Button label text. */
    pub text: &'static str,
    /** Text color. */
    pub text_color: u32,
    /** Normal background color. */
    pub normal_color: u32,
    /** Pressed background color. */
    pub pressed_color: u32,
    /** Disabled background color. */
    pub disabled_color: u32,
    /** Click handler ID (bound via Modifier). */
    pub on_click_id: u64,
}

impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            text: "", text_color: 0xFF000000,
            normal_color: 0xFFFFFFFF, pressed_color: 0xFF808080,
            disabled_color: 0xFF808080, on_click_id: 0,
        }
    }
}

/** Image component properties. */
#[derive(Debug, Clone, Default)]
pub struct ImageProps {
    /** Resource path. */
    pub path: &'static str,
    /** Width in dp. */
    pub width: f32,
    /** Height in dp. */
    pub height: f32,
    /** Fit mode (0=fill, 1=contain, 2=cover, 3=none). */
    pub fit: u32,
}

/** ScrollView component properties. */
#[derive(Debug, Clone, Default)]
pub struct ScrollViewProps {
    /** Scroll direction (0=vertical, 1=horizontal, 2=both). */
    pub scroll_direction: u32,
    /** Content offset X. */
    pub content_offset_x: f32,
    /** Content offset Y. */
    pub content_offset_y: f32,
}

/** Spacer component properties. */
#[derive(Debug, Clone, Default)]
pub struct SpacerProps {
    /** Flex factor (0=fixed, >0=flexible). */
    pub flex: u32,
}

/** SizedBox component properties. */
#[derive(Debug, Clone, Default)]
pub struct SizedBoxProps {
    /** Fixed width. */
    pub width: f32,
    /** Fixed height. */
    pub height: f32,
}

/** Union of all component property types. */
#[derive(Debug, Clone, Default)]
pub enum ComponentProps {
    #[default]
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

/** Cached layout result for a component. */
#[derive(Debug, Clone, Default)]
pub struct LayoutResult {
    /** Computed x position. */
    pub x: f32,
    /** Computed y position. */
    pub y: f32,
    /** Computed width. */
    pub width: f32,
    /** Computed height. */
    pub height: f32,
}

/** Immutable element node in the declarative component tree.
 *
 * An Element describes a component instance with its type, key,
 * properties, children, and cached layout result. The tree is
 * rebuilt each frame by the render() function; the Reconciler
 * diffs the old and new trees to produce minimal updates.
 */
#[derive(Debug, Clone)]
pub struct Element {
    /** Component type discriminant. */
    pub component_type: ComponentType,
    /** Stable key for diff matching. */
    pub key: u64,
    /** Component-specific properties. */
    pub props: ComponentProps,
    /** Child elements. */
    pub children: &'static [Element],
    /** Cached layout result. */
    pub layout_result: LayoutResult,
}

/** Component trait — pure-function render model.
 *
 * Each component implements this trait to declare its element tree.
 * The render() function is called each frame; it must be side-effect-free.
 */
pub trait Component {
    /** Render this component into an Element tree. */
    fn render(&self) -> Element;

    /** Get the component type discriminant. */
    fn component_type(&self) -> ComponentType;

    /** Get the stable key for diff matching. */
    fn key(&self) -> u64 { 0 }
}

/** Text declarative component. */
pub struct Text {
    /** Properties. */
    pub props: TextProps,
}

impl Component for Text {
    fn render(&self) -> Element {
        Element {
            component_type: ComponentType::Text,
            key: 0,
            props: ComponentProps::Text(self.props.clone()),
            children: &[],
            layout_result: LayoutResult::default(),
        }
    }
    fn component_type(&self) -> ComponentType { ComponentType::Text }
}

/** Column declarative component (vertical layout). */
pub struct Column {
    /** Properties. */
    pub props: LayoutProps,
    /** Children. */
    pub children: &'static [Element],
}

impl Component for Column {
    fn render(&self) -> Element {
        Element {
            component_type: ComponentType::Column,
            key: 0,
            props: ComponentProps::Layout(self.props.clone()),
            children: self.children,
            layout_result: LayoutResult::default(),
        }
    }
    fn component_type(&self) -> ComponentType { ComponentType::Column }
}

/** Row declarative component (horizontal layout). */
pub struct Row {
    /** Properties. */
    pub props: LayoutProps,
    /** Children. */
    pub children: &'static [Element],
}

impl Component for Row {
    fn render(&self) -> Element {
        Element {
            component_type: ComponentType::Row,
            key: 0,
            props: ComponentProps::Layout(self.props.clone()),
            children: self.children,
            layout_result: LayoutResult::default(),
        }
    }
    fn component_type(&self) -> ComponentType { ComponentType::Row }
}

/** Stack declarative component (layered layout). */
pub struct Stack {
    /** Properties. */
    pub props: LayoutProps,
    /** Children. */
    pub children: &'static [Element],
}

impl Component for Stack {
    fn render(&self) -> Element {
        Element {
            component_type: ComponentType::Stack,
            key: 0,
            props: ComponentProps::Layout(self.props.clone()),
            children: self.children,
            layout_result: LayoutResult::default(),
        }
    }
    fn component_type(&self) -> ComponentType { ComponentType::Stack }
}

/** Button declarative component. */
pub struct Button {
    /** Properties. */
    pub props: ButtonProps,
}

impl Component for Button {
    fn render(&self) -> Element {
        Element {
            component_type: ComponentType::Button,
            key: 0,
            props: ComponentProps::Button(self.props.clone()),
            children: &[],
            layout_result: LayoutResult::default(),
        }
    }
    fn component_type(&self) -> ComponentType { ComponentType::Button }
}

/** Image declarative component. */
pub struct Image {
    /** Properties. */
    pub props: ImageProps,
}

impl Component for Image {
    fn render(&self) -> Element {
        Element {
            component_type: ComponentType::Image,
            key: 0,
            props: ComponentProps::Image(self.props.clone()),
            children: &[],
            layout_result: LayoutResult::default(),
        }
    }
    fn component_type(&self) -> ComponentType { ComponentType::Image }
}

/** ScrollView declarative component. */
pub struct ScrollView {
    /** Properties. */
    pub props: ScrollViewProps,
    /** Child content. */
    pub child: &'static Element,
}

impl Component for ScrollView {
    fn render(&self) -> Element {
        Element {
            component_type: ComponentType::ScrollView,
            key: 0,
            props: ComponentProps::ScrollView(self.props.clone()),
            children: core::slice::from_ref(self.child),
            layout_result: LayoutResult::default(),
        }
    }
    fn component_type(&self) -> ComponentType { ComponentType::ScrollView }
}

/** Spacer declarative component. */
pub struct Spacer {
    /** Properties. */
    pub props: SpacerProps,
}

impl Component for Spacer {
    fn render(&self) -> Element {
        Element {
            component_type: ComponentType::Spacer,
            key: 0,
            props: ComponentProps::Spacer(self.props.clone()),
            children: &[],
            layout_result: LayoutResult::default(),
        }
    }
    fn component_type(&self) -> ComponentType { ComponentType::Spacer }
}

/** SizedBox declarative component. */
pub struct SizedBox {
    /** Properties. */
    pub props: SizedBoxProps,
    /** Child. */
    pub child: &'static Element,
}

impl Component for SizedBox {
    fn render(&self) -> Element {
        Element {
            component_type: ComponentType::SizedBox,
            key: 0,
            props: ComponentProps::SizedBox(self.props.clone()),
            children: core::slice::from_ref(self.child),
            layout_result: LayoutResult::default(),
        }
    }
    fn component_type(&self) -> ComponentType { ComponentType::SizedBox }
}

/** Reactive State<T> for declarative UI.
 *
 * Wraps a value with atomic version tracking and dirty marking.
 * When set() changes the value, the version increments and the
 * dirty flag is set, triggering a re-render on the next frame.
 *
 * Constraint: T: Copy + PartialEq (no heap types).
 */
pub struct State<T: Copy + PartialEq> {
    /** Current value. */
    value: T,
    /** Version counter (increments on each change). */
    version: AtomicU32,
    /** Dirty flag (set when value changes). */
    dirty: AtomicU32,
}

impl<T: Copy + PartialEq> State<T> {
    /** Create a new State with an initial value. */
    pub const fn new(value: T) -> Self {
        State {
            value,
            version: AtomicU32::new(0),
            dirty: AtomicU32::new(0),
        }
    }

    /** Get the current value. */
    pub fn get(&self) -> T { self.value }

    /** Set a new value. Only marks dirty if the value actually changes. */
    pub fn set(&mut self, value: T) {
        if self.value != value {
            self.value = value;
            self.version.fetch_add(1, Ordering::AcqRel);
            self.dirty.store(1, Ordering::Release);
        }
    }

    /** Get the current version number. */
    pub fn version(&self) -> u32 { self.version.load(Ordering::Acquire) }

    /** Check and consume the dirty flag. Returns true if dirty. */
    pub fn take_dirty(&self) -> bool {
        self.dirty.swap(0, Ordering::AcqRel) != 0
    }
}

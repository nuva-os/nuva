/*
 * Nuva OS - SystemService - Web - Layout Engine
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

//! Layout engine implementing CSS Box Model, Flexbox, and Grid layout.
//! Computes position and size for each DOM element to produce
//! a layout tree for the rendering backend.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::css_parser::{ComputedStyle, CssValue};
use super::dom::{DomTree, NodeId, NodeType};
use super::error::WebError;
use alloc::format;

/// 2D point/size
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point(pub f32, pub f32);

/// 2D rectangle (x, y, width, height)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Width
    pub w: f32,
    /// Height
    pub h: f32,
}

impl Rect {
    /// Create a zero rectangle
    pub const ZERO: Rect = Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };

    /// Check if this rect has positive area
    pub fn is_visible(&self) -> bool {
        self.w > 0.0 && self.h > 0.0
    }
}

/// CSS Box Model edges (margin, padding, border)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxEdges {
    /// Top edge
    pub top: f32,
    /// Right edge
    pub right: f32,
    /// Bottom edge
    pub bottom: f32,
    /// Left edge
    pub left: f32,
}

impl BoxEdges {
    /// All zeros
    pub const ZERO: BoxEdges = BoxEdges { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 };

    /// Create uniform edges
    pub const fn uniform(v: f32) -> BoxEdges {
        BoxEdges { top: v, right: v, bottom: v, left: v }
    }

    /// Horizontal sum (left + right)
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Vertical sum (top + bottom)
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// CSS display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// display: block
    Block,
    /// display: inline
    Inline,
    /// display: inline-block
    InlineBlock,
    /// display: flex
    Flex,
    /// display: inline-flex
    InlineFlex,
    /// display: grid
    Grid,
    /// display: inline-grid
    InlineGrid,
    /// display: none
    None,
}

/// CSS flex direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    /// Row (left to right)
    Row,
    /// Row reversed
    RowReverse,
    /// Column (top to bottom)
    Column,
    /// Column reversed
    ColumnReverse,
}

/// CSS flex wrap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    /// No wrapping
    NoWrap,
    /// Wrap
    Wrap,
    /// Wrap reversed
    WrapReverse,
}

/// CSS justify-content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    /// flex-start
    FlexStart,
    /// flex-end
    FlexEnd,
    /// center
    Center,
    /// space-between
    SpaceBetween,
    /// space-around
    SpaceAround,
    /// space-evenly
    SpaceEvenly,
}

/// CSS align-items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    /// flex-start
    FlexStart,
    /// flex-end
    FlexEnd,
    /// center
    Center,
    /// stretch
    Stretch,
    /// baseline
    Baseline,
}

/// Flexbox container properties
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlexProperties {
    /// Main axis direction
    pub direction: FlexDirection,
    /// Wrapping behavior
    pub wrap: FlexWrap,
    /// Main axis alignment
    pub justify_content: JustifyContent,
    /// Cross axis alignment
    pub align_items: AlignItems,
    /// Gap between items
    pub gap: f32,
}

impl FlexProperties {
    /// Default flex properties
    pub const DEFAULT: FlexProperties = FlexProperties {
        direction: FlexDirection::Row,
        wrap: FlexWrap::NoWrap,
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Stretch,
        gap: 0.0,
    };
}

/// CSS grid track sizing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridTrackSize {
    /// Fixed pixel size
    Fixed(f32),
    /// Flexible fraction (fr)
    Fr(f32),
    /// Auto-sized
    Auto,
    /// Min-content
    MinContent,
    /// Max-content
    MaxContent,
    /// Minmax(min, max)
    MinMax(f32, f32),
}

/// Grid container properties
#[derive(Debug, Clone)]
pub struct GridProperties {
    /// Column track definitions
    pub template_columns: Vec<GridTrackSize>,
    /// Row track definitions
    pub template_rows: Vec<GridTrackSize>,
    /// Column gap
    pub column_gap: f32,
    /// Row gap
    pub row_gap: f32,
}

impl GridProperties {
    /// Default grid properties
    pub fn default() -> Self {
        GridProperties {
            template_columns: Vec::new(),
            template_rows: Vec::new(),
            column_gap: 0.0,
            row_gap: 0.0,
        }
    }
}

/// Layout box for a single element
#[derive(Debug, Clone)]
pub struct LayoutBox {
    /// DOM node ID this box represents
    pub node_id: NodeId,
    /// Content area position and size
    pub content_rect: Rect,
    /// Margin edges
    pub margin: BoxEdges,
    /// Padding edges
    pub padding: BoxEdges,
    /// Border edges
    pub border: BoxEdges,
    /// Display mode
    pub display: DisplayMode,
    /// Flex properties (if display is Flex)
    pub flex_props: Option<FlexProperties>,
    /// Grid properties (if display is Grid)
    pub grid_props: Option<GridProperties>,
    /// Child layout boxes
    pub children: Vec<LayoutBox>,
    /// Computed style reference
    pub computed_style: ComputedStyle,
}

impl LayoutBox {
    /// Get the border box (content + padding + border)
    pub fn border_box(&self) -> Rect {
        Rect {
            x: self.content_rect.x - self.padding.left - self.border.left,
            y: self.content_rect.y - self.padding.top - self.border.top,
            w: self.content_rect.w + self.padding.horizontal() + self.border.horizontal(),
            h: self.content_rect.h + self.padding.vertical() + self.border.vertical(),
        }
    }

    /// Get the margin box (border box + margin)
    pub fn margin_box(&self) -> Rect {
        let bb = self.border_box();
        Rect {
            x: bb.x - self.margin.left,
            y: bb.y - self.margin.top,
            w: bb.w + self.margin.horizontal(),
            h: bb.h + self.margin.vertical(),
        }
    }
}

/// Layout engine
pub struct LayoutEngine {
    /// Viewport width in pixels
    viewport_width: f32,
    /// Viewport height in pixels
    viewport_height: f32,
    /// Default font size in pixels (for em/rem conversion)
    default_font_size: f32,
}

impl LayoutEngine {
    /// Create a new layout engine with viewport dimensions
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        LayoutEngine {
            viewport_width: viewport_width as f32,
            viewport_height: viewport_height as f32,
            default_font_size: 16.0,
        }
    }

    /// Compute the layout tree for a DOM tree
    pub fn compute_layout(
        &self,
        tree: &DomTree,
        styles: &BTreeMap<u64, ComputedStyle>,
    ) -> Result<LayoutBox, WebError> {
        let doc_id = tree.document();
        self.layout_node(tree, styles, doc_id, 0.0, 0.0, self.viewport_width)
    }

    /// Layout a single node and its children
    fn layout_node(
        &self,
        tree: &DomTree,
        styles: &BTreeMap<u64, ComputedStyle>,
        node_id: NodeId,
        x: f32,
        y: f32,
        available_width: f32,
    ) -> Result<LayoutBox, WebError> {
        let node = tree.get_node(node_id).ok_or(WebError::ResourceNotFound)?;

        let computed = styles.get(&node_id.0).cloned().unwrap_or_else(ComputedStyle::new);

        let display = self.resolve_display(&computed);
        let margin = self.resolve_box_edges(&computed, "margin");
        let padding = self.resolve_box_edges(&computed, "padding");
        let border = self.resolve_box_edges(&computed, "border-width");

        let content_x = x + margin.left + border.left + padding.left;
        let content_y = y + margin.top + border.top + padding.top;
        let content_width = available_width - margin.horizontal() - border.horizontal() - padding.horizontal();

        let mut children = Vec::new();
        let mut child_y = content_y;

        if display != DisplayMode::None {
            for &child_id in &node.children {
                let child_node = tree.get_node(child_id);
                if let Some(child) = child_node {
                    if child.node_type == NodeType::Text {
                        continue;
                    }
                    let child_box = self.layout_node(
                        tree, styles, child_id,
                        content_x, child_y, content_width,
                    )?;
                    if child_box.display != DisplayMode::None {
                        child_y = child_box.margin_box().y + child_box.margin_box().h;
                        children.push(child_box);
                    }
                }
            }
        }

        let content_height = if children.is_empty() {
            self.resolve_content_height(&computed)
        } else if child_y > content_y {
            child_y - content_y
        } else {
            0.0
        };

        let mut layout_box = LayoutBox {
            node_id,
            content_rect: Rect {
                x: content_x,
                y: content_y,
                w: if content_width > 0.0 { content_width } else { 0.0 },
                h: content_height,
            },
            margin,
            padding,
            border,
            display,
            flex_props: None,
            grid_props: None,
            children,
            computed_style: computed,
        };

        // Apply flexbox or grid layout if applicable
        match display {
            DisplayMode::Flex | DisplayMode::InlineFlex => {
                let flex_props = self.resolve_flex_properties(&layout_box.computed_style);
                layout_box.flex_props = Some(flex_props);
                self.apply_flex_layout(&mut layout_box)?;
            }
            DisplayMode::Grid | DisplayMode::InlineGrid => {
                let grid_props = self.resolve_grid_properties(&layout_box.computed_style);
                layout_box.grid_props = Some(grid_props);
                self.apply_grid_layout(&mut layout_box)?;
            }
            _ => {}
        }

        Ok(layout_box)
    }

    /// Resolve the display mode from computed style
    fn resolve_display(&self, style: &ComputedStyle) -> DisplayMode {
        match style.get_property("display") {
            Some(CssValue::Keyword(k)) => match k.as_str() {
                "block" => DisplayMode::Block,
                "inline" => DisplayMode::Inline,
                "inline-block" => DisplayMode::InlineBlock,
                "flex" => DisplayMode::Flex,
                "inline-flex" => DisplayMode::InlineFlex,
                "grid" => DisplayMode::Grid,
                "inline-grid" => DisplayMode::InlineGrid,
                "none" => DisplayMode::None,
                _ => DisplayMode::Block,
            },
            _ => DisplayMode::Block,
        }
    }

    /// Resolve box edges (margin/padding/border) from computed style
    fn resolve_box_edges(&self, style: &ComputedStyle, prefix: &str) -> BoxEdges {
        let top = self.resolve_length(style, &format!("{}-top", prefix));
        let right = self.resolve_length(style, &format!("{}-right", prefix));
        let bottom = self.resolve_length(style, &format!("{}-bottom", prefix));
        let left = self.resolve_length(style, &format!("{}-left", prefix));
        BoxEdges { top, right, bottom, left }
    }

    /// Resolve a length value from computed style
    fn resolve_length(&self, style: &ComputedStyle, prop: &str) -> f32 {
        match style.get_property(prop) {
            Some(CssValue::Px(v)) => *v,
            Some(CssValue::Em(v)) => *v * self.default_font_size,
            Some(CssValue::Rem(v)) => *v * self.default_font_size,
            Some(CssValue::Percent(v)) => *v / 100.0 * self.viewport_width,
            Some(CssValue::Vw(v)) => *v / 100.0 * self.viewport_width,
            Some(CssValue::Vh(v)) => *v / 100.0 * self.viewport_height,
            _ => 0.0,
        }
    }

    /// Resolve content height
    fn resolve_content_height(&self, style: &ComputedStyle) -> f32 {
        self.resolve_length(style, "height")
    }

    /// Resolve flex properties from computed style
    fn resolve_flex_properties(&self, style: &ComputedStyle) -> FlexProperties {
        let direction = match style.get_property("flex-direction") {
            Some(CssValue::Keyword(k)) => match k.as_str() {
                "row" => FlexDirection::Row,
                "row-reverse" => FlexDirection::RowReverse,
                "column" => FlexDirection::Column,
                "column-reverse" => FlexDirection::ColumnReverse,
                _ => FlexDirection::Row,
            },
            _ => FlexDirection::Row,
        };

        let wrap = match style.get_property("flex-wrap") {
            Some(CssValue::Keyword(k)) => match k.as_str() {
                "nowrap" => FlexWrap::NoWrap,
                "wrap" => FlexWrap::Wrap,
                "wrap-reverse" => FlexWrap::WrapReverse,
                _ => FlexWrap::NoWrap,
            },
            _ => FlexWrap::NoWrap,
        };

        let justify = match style.get_property("justify-content") {
            Some(CssValue::Keyword(k)) => match k.as_str() {
                "flex-start" => JustifyContent::FlexStart,
                "flex-end" => JustifyContent::FlexEnd,
                "center" => JustifyContent::Center,
                "space-between" => JustifyContent::SpaceBetween,
                "space-around" => JustifyContent::SpaceAround,
                "space-evenly" => JustifyContent::SpaceEvenly,
                _ => JustifyContent::FlexStart,
            },
            _ => JustifyContent::FlexStart,
        };

        let align = match style.get_property("align-items") {
            Some(CssValue::Keyword(k)) => match k.as_str() {
                "flex-start" => AlignItems::FlexStart,
                "flex-end" => AlignItems::FlexEnd,
                "center" => AlignItems::Center,
                "stretch" => AlignItems::Stretch,
                "baseline" => AlignItems::Baseline,
                _ => AlignItems::Stretch,
            },
            _ => AlignItems::Stretch,
        };

        let gap = self.resolve_length(style, "gap");

        FlexProperties {
            direction,
            wrap,
            justify_content: justify,
            align_items: align,
            gap,
        }
    }

    /// Resolve grid properties from computed style
    fn resolve_grid_properties(&self, style: &ComputedStyle) -> GridProperties {
        let _ = style;
        GridProperties::default()
    }

    /// Apply flexbox layout algorithm to a container
    fn apply_flex_layout(&self, container: &mut LayoutBox) -> Result<(), WebError> {
        let flex_props = container.flex_props.unwrap_or(FlexProperties::DEFAULT);
        let is_row = flex_props.direction == FlexDirection::Row || flex_props.direction == FlexDirection::RowReverse;

        let main_size = if is_row { container.content_rect.w } else { container.content_rect.h };
        let cross_size = if is_row { container.content_rect.h } else { container.content_rect.w };

        // Collect main-axis sizes of children
        let child_count = container.children.len();
        if child_count == 0 {
            return Ok(());
        }

        let mut total_main: f32 = 0.0;
        for child in &container.children {
            let size = if is_row { child.margin_box().w } else { child.margin_box().h };
            total_main += size;
        }
        total_main += flex_props.gap * (child_count.saturating_sub(1) as f32);

        let free_space = main_size - total_main;
        let mut current_main = 0.0f32;

        // Distribute children along main axis
        let gap = flex_props.gap;
        for (i, child) in container.children.iter_mut().enumerate() {
            let child_main = if is_row { child.margin_box().w } else { child.margin_box().h };
            let child_cross = if is_row { child.margin_box().h } else { child.margin_box().w };

            // Position on main axis
            if is_row {
                child.content_rect.x = container.content_rect.x + current_main + child.margin.left + child.border.left + child.padding.left;
            } else {
                child.content_rect.y = container.content_rect.y + current_main + child.margin.top + child.border.top + child.padding.top;
            }

            // Align on cross axis
            let cross_offset = match flex_props.align_items {
                AlignItems::FlexStart => 0.0,
                AlignItems::FlexEnd => cross_size - child_cross,
                AlignItems::Center => (cross_size - child_cross) / 2.0,
                AlignItems::Stretch => {
                    if is_row {
                        child.content_rect.h = cross_size - child.padding.vertical() - child.border.vertical();
                    } else {
                        child.content_rect.w = cross_size - child.padding.horizontal() - child.border.horizontal();
                    }
                    0.0
                }
                AlignItems::Baseline => 0.0,
            };

            if is_row {
                child.content_rect.y = container.content_rect.y + cross_offset + child.margin.top + child.border.top + child.padding.top;
            } else {
                child.content_rect.x = container.content_rect.x + cross_offset + child.margin.left + child.border.left + child.padding.left;
            }

            current_main += child_main + if i < child_count - 1 { gap } else { 0.0 };
        }

        // Apply justify-content for remaining free space
        if free_space > 0.0 && child_count > 0 {
            let offset = match flex_props.justify_content {
                JustifyContent::FlexStart => 0.0,
                JustifyContent::FlexEnd => free_space,
                JustifyContent::Center => free_space / 2.0,
                JustifyContent::SpaceBetween => {
                    if child_count > 1 { 0.0 } else { free_space / 2.0 }
                }
                JustifyContent::SpaceAround => free_space / (child_count as f32 * 2.0),
                JustifyContent::SpaceEvenly => free_space / (child_count as f32 + 1.0),
            };

            for child in &mut container.children {
                if is_row {
                    child.content_rect.x += offset;
                } else {
                    child.content_rect.y += offset;
                }
            }
        }

        Ok(())
    }

    /// Apply grid layout algorithm to a container
    fn apply_grid_layout(&self, container: &mut LayoutBox) -> Result<(), WebError> {
        let grid_props = container.grid_props.as_ref().unwrap_or(&GridProperties::default());

        let col_count = if grid_props.template_columns.is_empty() {
            1
        } else {
            grid_props.template_columns.len()
        };

        let col_width = if col_count > 0 {
            container.content_rect.w / col_count as f32
        } else {
            container.content_rect.w
        };

        let mut row = 0u32;
        let mut col = 0u32;
        let mut row_height = 0.0f32;
        let mut y_offset = 0.0f32;

        for child in container.children.iter_mut() {
            child.content_rect.x = container.content_rect.x + col as f32 * col_width;
            child.content_rect.y = container.content_rect.y + y_offset;
            child.content_rect.w = col_width;

            row_height = if child.content_rect.h > row_height { child.content_rect.h } else { row_height };

            col += 1;
            if col as usize >= col_count {
                col = 0;
                y_offset += row_height + grid_props.row_gap;
                row_height = 0.0;
                row += 1;
            }
        }

        Ok(())
    }
}

/*
 * Nuva OS - Application - Ui - Layout
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
 * Nuva OS - Declarative Layout Engine
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Layout algorithms for the declarative render pipeline:
 * Horizontal, Vertical, Grid (CSS-compatible), and Flex layouts.
 * Operates on Element trees via LayoutResult.
 */

use super::component_impl::{Element, LayoutResult, ComponentType};

/// Layout direction for flex containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

/// Layout type discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutType {
    None,
    Horizontal,
    Vertical,
    Grid,
    Flex,
    Absolute,
}

/// Flex direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

/// Justify content (main axis alignment).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Alignment on the cross axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Start,
    Center,
    End,
    Fill,
}

/// Layout constraints for a component.
#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
    pub gap: u32,
    pub direction: FlexDirection,
    pub justify_content: JustifyContent,
}

impl Constraints {
    pub const fn none() -> Self {
        Constraints {
            min_width: 0, max_width: u32::MAX,
            min_height: 0, max_height: u32::MAX,
            gap: 0, direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
        }
    }

    pub const fn tight(width: u32, height: u32) -> Self {
        Constraints {
            min_width: width, max_width: width,
            min_height: height, max_height: height,
            gap: 0, direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
        }
    }

    pub fn constrain(&self, width: u32, height: u32) -> (u32, u32) {
        (width.clamp(self.min_width, self.max_width),
         height.clamp(self.min_height, self.max_height))
    }
}

impl Default for Constraints {
    fn default() -> Self { Self::none() }
}

/// Layout parameters for container components.
#[derive(Debug, Clone)]
pub struct LayoutParams {
    pub layout_type: LayoutType,
    pub spacing: u32,
    pub alignment: Alignment,
    pub grid_columns: u32,
    pub grid_rows: u32,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: f32,
    pub constraints: Constraints,
}

impl Default for LayoutParams {
    fn default() -> Self {
        LayoutParams {
            layout_type: LayoutType::None,
            spacing: 0,
            alignment: Alignment::Start,
            grid_columns: 1,
            grid_rows: 1,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: 0.0,
            constraints: Constraints::none(),
        }
    }
}

/// Layout manager — computes LayoutResult for each Element in a tree.
///
/// Called during the Layout phase of the render pipeline. Applies the
/// appropriate layout algorithm based on the parent container's type.
pub struct LayoutManager;

impl LayoutManager {
    /// Layout children within a parent container.
    /// Returns the updated children with computed LayoutResults.
    pub fn layout(parent: &LayoutResult, children: &mut [Element], params: &LayoutParams) {
        match params.layout_type {
            LayoutType::None => {}
            LayoutType::Horizontal => Self::layout_horizontal(parent, children, params),
            LayoutType::Vertical => Self::layout_vertical(parent, children, params),
            LayoutType::Grid => Self::layout_grid(parent, children, params),
            LayoutType::Flex => Self::layout_flex(parent, children, params),
            LayoutType::Absolute => {},
        }
    }

    fn layout_horizontal(parent: &LayoutResult, children: &mut [Element], params: &LayoutParams) {
        let spacing = params.spacing as f32;
        let mut x = parent.x;
        let y = parent.y;

        for child in children.iter_mut() {
            child.layout_result.x = x;
            child.layout_result.y = y;

            match params.alignment {
                Alignment::Center => {
                    child.layout_result.y = y + (parent.height - child.layout_result.height) / 2.0;
                }
                Alignment::End => {
                    child.layout_result.y = y + parent.height - child.layout_result.height;
                }
                Alignment::Fill => {
                    child.layout_result.height = parent.height;
                }
                _ => {}
            }

            x += child.layout_result.width + spacing;
        }
    }

    fn layout_vertical(parent: &LayoutResult, children: &mut [Element], params: &LayoutParams) {
        let spacing = params.spacing as f32;
        let x = parent.x;
        let mut y = parent.y;

        for child in children.iter_mut() {
            child.layout_result.x = x;
            child.layout_result.y = y;

            match params.alignment {
                Alignment::Center => {
                    child.layout_result.x = x + (parent.width - child.layout_result.width) / 2.0;
                }
                Alignment::End => {
                    child.layout_result.x = x + parent.width - child.layout_result.width;
                }
                Alignment::Fill => {
                    child.layout_result.width = parent.width;
                }
                _ => {}
            }

            y += child.layout_result.height + spacing;
        }
    }

    fn layout_grid(parent: &LayoutResult, children: &mut [Element], params: &LayoutParams) {
        let cols = params.grid_columns.max(1);
        let rows = params.grid_rows.max(1);

        let cell_width = parent.width / cols as f32;
        let cell_height = parent.height / rows as f32;

        for (i, child) in children.iter_mut().enumerate() {
            let col = i % cols as usize;
            let row = i / cols as usize;

            child.layout_result.x = parent.x + col as f32 * cell_width;
            child.layout_result.y = parent.y + row as f32 * cell_height;
            child.layout_result.width = cell_width;
            child.layout_result.height = cell_height;
        }
    }

    fn layout_flex(parent: &LayoutResult, children: &mut [Element], params: &LayoutParams) {
        let constraints = &params.constraints;
        let is_row = matches!(constraints.direction,
            FlexDirection::Row | FlexDirection::RowReverse);

        let available_main = if is_row { parent.width } else { parent.height };
        let available_cross = if is_row { parent.height } else { parent.width };

        // Constrain children on the cross axis
        for child in children.iter_mut() {
            if is_row {
                child.layout_result.height = available_cross
                    .min(child.layout_result.height);
            } else {
                child.layout_result.width = available_cross
                    .min(child.layout_result.width);
            }
        }

        // Compute main axis positions based on justify_content
        let total_main: f32 = children.iter().map(|c| {
            if is_row { c.layout_result.width } else { c.layout_result.height }
        }).sum();
        let gap_total = (children.len().saturating_sub(1)) as f32 * constraints.gap as f32;
        let remaining = available_main - total_main - gap_total;

        let (start_pos, gap) = match constraints.justify_content {
            JustifyContent::FlexStart => (0.0, constraints.gap as f32),
            JustifyContent::FlexEnd => (remaining.max(0.0), constraints.gap as f32),
            JustifyContent::Center => ((remaining / 2.0).max(0.0), constraints.gap as f32),
            JustifyContent::SpaceBetween => {
                let g = if children.len() > 1 {
                    remaining / (children.len() - 1) as f32
                } else { 0.0 };
                (0.0, g.max(constraints.gap as f32))
            }
            JustifyContent::SpaceAround => {
                let g = if !children.is_empty() {
                    remaining / children.len() as f32
                } else { 0.0 };
                (g / 2.0, g.max(constraints.gap as f32))
            }
            JustifyContent::SpaceEvenly => {
                let g = if !children.is_empty() {
                    remaining / (children.len() + 1) as f32
                } else { remaining / 2.0 };
                (g, g.max(constraints.gap as f32))
            }
        };

        let mut pos = start_pos;
        let reverse = matches!(constraints.direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse);

        if reverse {
            pos = available_main - start_pos;
            for child in children.iter_mut() {
                let size = if is_row { child.layout_result.width } else { child.layout_result.height };
                pos -= size;
                if is_row { child.layout_result.x = parent.x + pos; }
                else { child.layout_result.y = parent.y + pos; }
                pos -= gap;
            }
        } else {
            for child in children.iter_mut() {
                if is_row {
                    child.layout_result.x = parent.x + pos;
                    child.layout_result.y = parent.y;
                } else {
                    child.layout_result.x = parent.x;
                    child.layout_result.y = parent.y + pos;
                }
                let size = if is_row { child.layout_result.width } else { child.layout_result.height };
                pos += size + gap;
            }
        }
    }
}

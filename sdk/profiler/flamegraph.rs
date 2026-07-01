/*
 * Nuva OS
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

// ! Flamegraph generation with SVG/PNG support

use std::collections::HashMap;
use crate::error::SdkError;
use super::cpu::{CpuProfile, CallNode};
use alloc::vec;
use alloc::format;
use alloc::vec::Vec;

/// Generate flamegraph with default width
pub fn generate(profile: &CpuProfile) -> Result<String, SdkError> {
    generate_with_width(profile, 1200)
}

/// Generate flamegraph with custom width
pub fn generate_with_width(profile: &CpuProfile, width: usize) -> Result<String, SdkError> {
    let mut generator = FlameGraphGenerator::new(width);
    generator.generate(profile)
}

/// Flamegraph generator
struct FlameGraphGenerator {
    /// SVG width
    width: usize,
    /// Color theme
    theme: ColorTheme,
}

impl FlameGraphGenerator {
    fn new(width: usize) -> Self {
        Self {
            width,
            theme: ColorTheme::default(),
        }
    }

    /// Generate SVG flamegraph
    fn generate(&mut self, profile: &CpuProfile) -> Result<String, SdkError> {
        let root = self.build_flame_tree(profile);

        let height = self.compute_height(&root) * 16 + 50;

        let mut svg = String::new();

        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
<style>
 .func {{ font-family: monospace; font-size: 11px; fill: #000000; }}
 .bg {{ fill: #eeeeee; }}
 .title {{ font-family: sans-serif; font-size: 16px; font-weight: bold; fill: #333333; }}
 .subtitle {{ font-family: sans-serif; font-size: 12px; fill: #666666; }}
</style>
<rect class="bg" x="0" y="0" width="100%" height="100%"/>
<text class="title" x="{}" y="25" text-anchor="middle">Flame Graph</text>
<text class="subtitle" x="{}" y="40" text-anchor="middle">{} samples</text>
"#,
            self.width, height, self.width, height,
            self.width / 2, self.width / 2, profile.samples.len()
        ));

        self.render_node(&mut svg, &root, 0.0, self.width as f64, 2, self.width);

        svg.push_str("</svg>");

        Ok(svg)
    }

    /// Build flame tree from profile
    fn build_flame_tree(&self, profile: &CpuProfile) -> FlameNode {
        let mut root = FlameNode {
            name: "root".to_string(),
            value: if profile.samples.is_empty() { 1 } else { profile.samples.len() },
            children: HashMap::new(),
        };

        for sample in &profile.samples {
            let mut current = &mut root;

            for frame in sample.stack.iter().rev() {
                current = current.children
                    .entry(frame.function.clone())
                    .or_insert_with(|| FlameNode {
                        name: frame.function.clone(),
                        value: 0,
                        children: HashMap::new(),
                    });
                current.value += 1;
            }
        }

        root
    }

    /// Compute tree height
    fn compute_height(&self, node: &FlameNode) -> usize {
        if node.children.is_empty() {
            1
        } else {
            1 + node.children.values().map(|c| self.compute_height(c)).max().unwrap_or(0)
        }
    }

    /// Render a node and its children
    fn render_node(
        &self,
        svg: &mut String,
        node: &FlameNode,
        x: f64,
        width: f64,
        depth: usize,
        total_width: usize,
    ) {
        let y = depth * 16;
        let height = 15;

        let color = self.theme.color_for_name(&node.name);

        svg.push_str(&format!(
            r#"<rect x="{:.2}" y="{}" width="{:.2}" height="{}" fill="{}" rx="1" ry="1"/>"#,
            x, y, width - 0.5, height, color
        ));

        if width > 40.0 {
            let text_x = x + 3.0;
            let text_y = y + 12;
            let text = self.truncate_text(&node.name, width as usize);

            let pct = if node.value > 0 && total_width > 0 {
                format!(" ({:.1}%)", (node.value as f64 / total_width as f64) * 100.0)
            } else {
                String::new()
            };

            svg.push_str(&format!(
                r#"<title>{}{}</title>"#,
                node.name, pct
            ));

            svg.push_str(&format!(
                r#"<text class="func" x="{:.2}" y="{}">{}</text>"#,
                text_x, text_y, text
            ));
        }

        let mut child_x = x;
        let mut sorted_children: Vec<_> = node.children.values().collect();
        sorted_children.sort_by(|a, b| b.value.cmp(&a.value));

        for child in sorted_children {
            if node.value == 0 {
                continue;
            }
            let child_width = (child.value as f64 / node.value as f64) * width;
            if child_width >= 1.0 {
                self.render_node(svg, child, child_x, child_width, depth + 1, total_width);
            }
            child_x += child_width;
        }
    }

    /// Truncate text to fit in given width
    fn truncate_text(&self, text: &str, max_width: usize) -> String {
        let char_width = 7;
        let max_chars = max_width / char_width;

        if text.len() <= max_chars {
            text.to_string()
        } else if max_chars > 3 {
            format!("{}...", &text[..max_chars.saturating_sub(3)])
        } else {
            "...".to_string()
        }
    }
}

/// Flame node
struct FlameNode {
    name: String,
    value: usize,
    children: HashMap<String, FlameNode>,
}

/// Color theme
struct ColorTheme {
    colors: Vec<String>,
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            colors: vec![
                "#ff6b6b".to_string(),
                "#feca57".to_string(),
                "#48dbfb".to_string(),
                "#ff9ff3".to_string(),
                "#54a0ff".to_string(),
                "#5f27cd".to_string(),
                "#00d2d3".to_string(),
                "#ff9f43".to_string(),
                "#ee5a24".to_string(),
                "#009432".to_string(),
            ],
        }
    }
}

impl ColorTheme {
    fn color_for_name(&self, name: &str) -> String {
        let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        let index = (hash as usize) % self.colors.len();
        self.colors[index].clone()
    }
}

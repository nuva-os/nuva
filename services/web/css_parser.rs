/*
 * Nuva OS - SystemService - Web - CSS Parser
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

//! CSS3 parser and style computation engine.
//! Supports selector matching, style cascade resolution,
//! and computed style generation for the layout engine.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::dom::{DomNode, DomTree, NodeId, NodeType};
use super::error::WebError;

/// CSS specificity (a, b, c) per W3C spec
/// a = inline styles, b = ID selectors, c = class/attr/pseudo-class selectors
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity(pub u32, pub u32, pub u32);

impl Specificity {
    /// Zero specificity (universal selector)
    pub const ZERO: Specificity = Specificity(0, 0, 0);

    /// Inline style specificity
    pub const INLINE: Specificity = Specificity(1, 0, 0);

    /// Add two specificities
    pub fn add(self, other: Specificity) -> Specificity {
        Specificity(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

/// CSS selector type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// Universal selector (*)
    Universal,
    /// Type selector (tag name)
    Type(String),
    /// Class selector (.className)
    Class(String),
    /// ID selector (#id)
    Id(String),
    /// Attribute selector [attr], [attr=value], [attr~=value]
    Attribute {
        /// Attribute name
        name: String,
        /// Attribute value (None for existence check)
        value: Option<String>,
        /// Match operator
        op: AttrMatchOp,
    },
    /// Compound selector (tag.class#id)
    Compound(Vec<Selector>),
    /// Descendant combinator (ancestor descendant)
    Descendant(Box<Selector>, Box<Selector>),
    /// Child combinator (parent > child)
    Child(Box<Selector>, Box<Selector>),
}

/// Attribute match operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrMatchOp {
    /// [attr] - existence
    Exists,
    /// [attr=value] - exact match
    Exact,
    /// [attr~=value] - space-separated list contains
    ContainsWord,
    /// [attr^=value] - starts with
    StartsWith,
    /// [attr$=value] - ends with
    EndsWith,
    /// [attr*=value] - contains substring
    Contains,
}

impl Selector {
    /// Compute the specificity of this selector
    pub fn specificity(&self) -> Specificity {
        match self {
            Selector::Universal => Specificity::ZERO,
            Selector::Type(_) => Specificity(0, 0, 1),
            Selector::Class(_) => Specificity(0, 0, 1),
            Selector::Id(_) => Specificity(0, 1, 0),
            Selector::Attribute { .. } => Specificity(0, 0, 1),
            Selector::Compound(selectors) => {
                selectors.iter().map(|s| s.specificity()).fold(Specificity::ZERO, |a, b| a.add(b))
            }
            Selector::Descendant(a, b) | Selector::Child(a, b) => {
                a.specificity().add(b.specificity())
            }
        }
    }

    /// Check if this selector matches a DOM node
    pub fn matches(&self, node: &DomNode, tree: &DomTree) -> bool {
        match self {
            Selector::Universal => true,
            Selector::Type(tag) => node.node_type == NodeType::Element && node.tag_name == *tag,
            Selector::Class(class) => {
                if let Some(class_attr) = node.get_attribute("class") {
                    class_attr.split_whitespace().any(|c| c == class.as_str())
                } else {
                    false
                }
            }
            Selector::Id(id) => node.get_attribute("id").map_or(false, |v| v == id),
            Selector::Attribute { name, value, op } => {
                match node.get_attribute(name) {
                    None => false,
                    Some(attr_val) => match value {
                        None => true,
                        Some(v) => match op {
                            AttrMatchOp::Exists => true,
                            AttrMatchOp::Exact => attr_val == v,
                            AttrMatchOp::ContainsWord => {
                                attr_val.split_whitespace().any(|w| w == v.as_str())
                            }
                            AttrMatchOp::StartsWith => attr_val.starts_with(v.as_str()),
                            AttrMatchOp::EndsWith => attr_val.ends_with(v.as_str()),
                            AttrMatchOp::Contains => attr_val.contains(v.as_str()),
                        },
                    },
                }
            }
            Selector::Compound(selectors) => {
                selectors.iter().all(|s| s.matches(node, tree))
            }
            Selector::Descendant(ancestor, descendant) => {
                if !descendant.matches(node, tree) {
                    return false;
                }
                let mut current = node.parent;
                while let Some(parent_id) = current {
                    if let Some(parent) = tree.get_node(parent_id) {
                        if ancestor.matches(parent, tree) {
                            return true;
                        }
                        current = parent.parent;
                    } else {
                        break;
                    }
                }
                false
            }
            Selector::Child(parent_sel, child_sel) => {
                if !child_sel.matches(node, tree) {
                    return false;
                }
                if let Some(parent_id) = node.parent {
                    if let Some(parent) = tree.get_node(parent_id) {
                        return parent_sel.matches(parent, tree);
                    }
                }
                false
            }
        }
    }
}

/// CSS property value
#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    /// Length in pixels
    Px(f32),
    /// Percentage value
    Percent(f32),
    /// Em units
    Em(f32),
    /// Rem units
    Rem(f32),
    /// Viewport width percentage
    Vw(f32),
    /// Viewport height percentage
    Vh(f32),
    /// Auto value
    Auto,
    /// None value
    None,
    /// Inherit from parent
    Inherit,
    /// Color value (RGBA as u32: 0xRRGGBBAA)
    Color(u32),
    /// String value (font-family, etc.)
    String(String),
    /// Integer value
    Integer(i32),
    /// Keyword value
    Keyword(String),
}

/// Style declaration (property + value + importance)
#[derive(Debug, Clone)]
pub struct StyleDeclaration {
    /// CSS property name
    pub property: String,
    /// CSS property value
    pub value: CssValue,
    /// Whether this declaration is !important
    pub important: bool,
}

/// A single CSS rule (selector + declarations)
#[derive(Debug, Clone)]
pub struct CssRule {
    /// Selector for this rule
    pub selector: Selector,
    /// Style declarations
    pub declarations: Vec<StyleDeclaration>,
    /// Source order (for cascade resolution)
    pub source_order: u32,
}

impl CssRule {
    /// Compute the cascade priority: (importance, specificity, source_order)
    pub fn cascade_priority(&self) -> (bool, Specificity, u32) {
        let has_important = self.declarations.iter().any(|d| d.important);
        (has_important, self.selector.specificity(), self.source_order)
    }
}

/// A parsed CSS stylesheet
#[derive(Debug, Clone)]
pub struct Stylesheet {
    /// CSS rules in source order
    pub rules: Vec<CssRule>,
}

impl Stylesheet {
    /// Create an empty stylesheet
    pub fn new() -> Self {
        Stylesheet { rules: Vec::new() }
    }

    /// Compute the fully resolved style for a DOM node
    pub fn compute_style(
        &self,
        node: &DomNode,
        tree: &DomTree,
        inline_styles: &BTreeMap<String, CssValue>,
    ) -> ComputedStyle {
        let mut computed = ComputedStyle::new();

        // Collect matching rules sorted by cascade priority
        let mut matching: Vec<&CssRule> = self.rules.iter().filter(|r| r.selector.matches(node, tree)).collect();
        matching.sort_by_key(|r| r.cascade_priority());

        // Apply declarations in cascade order (lower priority first)
        for rule in &matching {
            for decl in &rule.declarations {
                computed.set_property(decl.property.clone(), decl.value.clone());
            }
        }

        // Inline styles have highest specificity (except !important)
        for (prop, val) in inline_styles {
            computed.set_property(prop.clone(), val.clone());
        }

        computed
    }
}

/// Computed style for a single element
#[derive(Debug, Clone)]
pub struct ComputedStyle {
    /// Property name to computed value mapping
    pub properties: BTreeMap<String, CssValue>,
}

impl ComputedStyle {
    /// Create an empty computed style
    pub fn new() -> Self {
        ComputedStyle {
            properties: BTreeMap::new(),
        }
    }

    /// Set a property value (overwrites if already present)
    pub fn set_property(&mut self, name: String, value: CssValue) {
        self.properties.insert(name, value);
    }

    /// Get a property value
    pub fn get_property(&self, name: &str) -> Option<&CssValue> {
        self.properties.get(name)
    }

    /// Remove a property
    pub fn remove_property(&mut self, name: &str) -> Option<CssValue> {
        self.properties.remove(name)
    }
}

/// CSS3 parser
pub struct CssParser {
    /// Current source order counter
    source_order: u32,
}

impl CssParser {
    /// Create a new CSS parser
    pub fn new() -> Self {
        CssParser { source_order: 0 }
    }

    /// Parse a CSS string into a stylesheet
    pub fn parse(&mut self, css: &str) -> Result<Stylesheet, WebError> {
        let mut rules = Vec::new();
        let mut pos = 0;
        let chars: Vec<char> = css.chars().collect();
        let len = chars.len();

        while pos < len {
            // Skip whitespace and comments
            pos = self.skip_ws_and_comments(&chars, pos);

            if pos >= len {
                break;
            }

            // Parse selector
            let (selector_str, new_pos) = self.read_until(&chars, pos, '{');
            if new_pos >= len {
                break;
            }
            pos = new_pos + 1;

            // Parse declaration block
            let (block_str, new_pos) = self.read_until(&chars, pos, '}');
            pos = if new_pos < len { new_pos + 1 } else { new_pos };

            let selector = self.parse_selector(selector_str.trim())?;
            let declarations = self.parse_declarations(block_str.trim())?;

            rules.push(CssRule {
                selector,
                declarations,
                source_order: self.source_order,
            });
            self.source_order += 1;
        }

        Ok(Stylesheet { rules })
    }

    /// Skip whitespace and /* comments */
    fn skip_ws_and_comments(&self, chars: &[char], mut pos: usize) -> usize {
        let len = chars.len();
        while pos < len {
            if chars[pos].is_whitespace() {
                pos += 1;
            } else if pos + 1 < len && chars[pos] == '/' && chars[pos + 1] == '*' {
                pos += 2;
                while pos + 1 < len && !(chars[pos] == '*' && chars[pos + 1] == '/') {
                    pos += 1;
                }
                if pos + 1 < len {
                    pos += 2;
                }
            } else {
                break;
            }
        }
        pos
    }

    /// Read characters until a delimiter is found (respecting nesting)
    fn read_until(&self, chars: &[char], start: usize, delimiter: char) -> (&str, usize) {
        let len = chars.len();
        let mut pos = start;
        while pos < len && chars[pos] != delimiter {
            pos += 1;
        }
        let s: String = chars[start..pos].iter().collect();
        // SAFETY: we constructed s from chars which are valid Unicode
        (Box::leak(s.into_boxed_str()), pos)
    }

    /// Parse a selector string into a Selector
    fn parse_selector(&self, s: &str) -> Result<Selector, WebError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Ok(Selector::Universal);
        }

        let mut parts = Vec::new();
        let mut current = String::new();

        for ch in trimmed.chars() {
            if ch == ' ' || ch == '\t' || ch == '\n' {
                if !current.is_empty() {
                    parts.push(self.parse_simple_selector(&current)?);
                    current.clear();
                }
            } else {
                current.push(ch);
            }
        }
        if !current.is_empty() {
            parts.push(self.parse_simple_selector(&current)?);
        }

        if parts.is_empty() {
            Ok(Selector::Universal)
        } else if parts.len() == 1 {
            // SAFETY: parts.len() == 1 guarantees next() returns Some
            Ok(parts.into_iter().next().unwrap_or(Selector::Universal))
        } else {
            Ok(Selector::Compound(parts))
        }
    }

    /// Parse a single simple selector (type, class, id, or compound like div.class#id)
    fn parse_simple_selector(&self, s: &str) -> Result<Selector, WebError> {
        let mut compound = Vec::new();
        let mut i = 0;
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();

        while i < len {
            if chars[i] == '#' {
                i += 1;
                let mut id = String::new();
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_') {
                    id.push(chars[i]);
                    i += 1;
                }
                compound.push(Selector::Id(id));
            } else if chars[i] == '.' {
                i += 1;
                let mut class = String::new();
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_') {
                    class.push(chars[i]);
                    i += 1;
                }
                compound.push(Selector::Class(class));
            } else if chars[i] == '[' {
                let mut attr_name = String::new();
                i += 1;
                while i < len && chars[i] != ']' && chars[i] != '=' && chars[i] != '~' {
                    attr_name.push(chars[i]);
                    i += 1;
                }
                let op = if i < len && chars[i] == '~' {
                    i += 1;
                    if i < len && chars[i] == '=' { i += 1; }
                    AttrMatchOp::ContainsWord
                } else if i < len && chars[i] == '=' {
                    i += 1;
                    AttrMatchOp::Exact
                } else {
                    AttrMatchOp::Exists
                };
                let mut attr_val = String::new();
                if i < len && chars[i] != ']' {
                    if chars[i] == '"' || chars[i] == '\'' { i += 1; }
                    while i < len && chars[i] != ']' && chars[i] != '"' && chars[i] != '\'' {
                        attr_val.push(chars[i]);
                        i += 1;
                    }
                    if i < len && (chars[i] == '"' || chars[i] == '\'') { i += 1; }
                }
                if i < len && chars[i] == ']' { i += 1; }
                compound.push(Selector::Attribute {
                    name: attr_name,
                    value: if attr_val.is_empty() { None } else { Some(attr_val) },
                    op,
                });
            } else if chars[i] == '*' {
                compound.push(Selector::Universal);
                i += 1;
            } else if chars[i].is_alphanumeric() || chars[i] == '-' {
                let mut tag = String::new();
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '-') {
                    tag.push(chars[i].to_ascii_lowercase());
                    i += 1;
                }
                compound.push(Selector::Type(tag));
            } else {
                i += 1;
            }
        }

        if compound.is_empty() {
            Ok(Selector::Universal)
        } else if compound.len() == 1 {
            // SAFETY: compound.len() == 1 guarantees next() returns Some
            Ok(compound.into_iter().next().unwrap_or(Selector::Universal))
        } else {
            Ok(Selector::Compound(compound))
        }
    }

    /// Parse a declaration block string into declarations
    fn parse_declarations(&self, block: &str) -> Result<Vec<StyleDeclaration>, WebError> {
        let mut declarations = Vec::new();

        for decl_str in block.split(';') {
            let trimmed = decl_str.trim();
            if trimmed.is_empty() {
                continue;
            }

            let mut parts = trimmed.splitn(2, ':');
            let property = parts.next().unwrap_or("").trim().to_ascii_lowercase();
            let value_str = parts.next().unwrap_or("").trim();

            if property.is_empty() || value_str.is_empty() {
                continue;
            }

            let (value_str, important) = if value_str.ends_with("!important") {
                (&value_str[..value_str.len() - 10], true)
            } else {
                (value_str, false)
            };

            let value = self.parse_css_value(value_str.trim());
            declarations.push(StyleDeclaration {
                property,
                value,
                important,
            });
        }

        Ok(declarations)
    }

    /// Parse a CSS value string into a CssValue
    fn parse_css_value(&self, s: &str) -> CssValue {
        if s == "auto" {
            return CssValue::Auto;
        }
        if s == "none" {
            return CssValue::None;
        }
        if s == "inherit" {
            return CssValue::Inherit;
        }

        // Try numeric value with unit
        let mut num_end = 0;
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();

        if len > 0 && chars[0] == '-' {
            num_end = 1;
        }
        while num_end < len && (chars[num_end].is_ascii_digit() || chars[num_end] == '.') {
            num_end += 1;
        }

        if num_end > 0 {
            let num_str: String = chars[..num_end].iter().collect();
            if let Ok(n) = num_str.parse::<f32>() {
                let unit: String = chars[num_end..].iter().collect();
                match unit.as_str() {
                    "px" => return CssValue::Px(n),
                    "%" => return CssValue::Percent(n),
                    "em" => return CssValue::Em(n),
                    "rem" => return CssValue::Rem(n),
                    "vw" => return CssValue::Vw(n),
                    "vh" => return CssValue::Vh(n),
                    "" => return CssValue::Px(n),
                    _ => {}
                }
                if let Ok(i) = num_str.parse::<i32>() {
                    return CssValue::Integer(i);
                }
            }
        }

        // Try color (#hex)
        if s.starts_with('#') {
            let hex = &s[1..];
            if hex.len() == 6 {
                if let Ok(c) = u32::from_str_radix(hex, 16) {
                    return CssValue::Color(c << 8 | 0xFF);
                }
            } else if hex.len() == 8 {
                if let Ok(c) = u32::from_str_radix(hex, 16) {
                    return CssValue::Color(c);
                }
            }
        }

        CssValue::Keyword(String::from(s))
    }
}

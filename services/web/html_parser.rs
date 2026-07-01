/*
 * Nuva OS - SystemService - Web - HTML Parser
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

//! HTML5 parser that constructs a DOM tree.
//! Supports tag, attribute, and text node parsing with basic
//! error recovery following the HTML5 specification's forgiving parsing model.

use alloc::string::String;
use alloc::vec::Vec;

use super::dom::{DomNode, DomTree, NodeId, NodeType};
use super::error::WebError;
use alloc::vec;

/// HTML parser state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserState {
    /// Reading initial content
    Initial,
    /// Inside an opening tag
    OpenTag,
    /// Inside a closing tag
    CloseTag,
    /// Reading attribute name
    AttrName,
    /// Reading attribute value
    AttrValue,
    /// Reading text content
    Text,
    /// Inside a comment
    Comment,
    /// Parser reached end of input
    Eof,
}

/// Self-closing HTML elements (void elements per spec)
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input",
    "link", "meta", "param", "source", "track", "wbr",
];

/// HTML5 parser
pub struct HtmlParser {
    /// Current parser state
    state: ParserState,
    /// Stack of open element tag names (for matching close tags)
    open_stack: Vec<String>,
    /// Current tag name being parsed
    current_tag: String,
    /// Current attribute name being parsed
    current_attr_name: String,
    /// Current attribute value being parsed
    current_attr_value: String,
    /// Whether the attribute value is quoted
    attr_quoted: bool,
    /// Current text content accumulator
    text_buffer: String,
    /// Collected attributes for current element
    pending_attrs: Vec<(String, String)>,
    /// Whether this tag is a closing tag
    is_closing: bool,
}

impl HtmlParser {
    /// Create a new HTML parser
    pub fn new() -> Self {
        HtmlParser {
            state: ParserState::Initial,
            open_stack: Vec::new(),
            current_tag: String::new(),
            current_attr_name: String::new(),
            current_attr_value: String::new(),
            attr_quoted: false,
            text_buffer: String::new(),
            pending_attrs: Vec::new(),
            is_closing: false,
        }
    }

    /// Parse an HTML string and build a DOM tree
    pub fn parse(&mut self, html: &str) -> Result<DomTree, WebError> {
        let mut tree = DomTree::new();
        let doc_id = tree.document();

        // Create <html> root element
        let html_node = DomNode::new_element(String::from("html"));
        let html_id = tree.append_child(doc_id, html_node)?;

        // Create <head> element
        let head_node = DomNode::new_element(String::from("head"));
        let head_id = tree.append_child(html_id, head_node)?;

        // Create <body> element
        let body_node = DomNode::new_element(String::from("body"));
        let body_id = tree.append_child(html_id, body_node)?;

        // Stack tracks current parent for insertion
        let mut parent_stack: Vec<NodeId> = vec![body_id];
        let mut current_parent = body_id;

        let chars: Vec<char> = html.chars().collect();
        let len = chars.len();
        let mut pos = 0;

        while pos < len {
            let ch = chars[pos];

            match self.state {
                ParserState::Initial | ParserState::Text => {
                    if ch == '<' {
                        // Flush accumulated text
                        if !self.text_buffer.is_empty() {
                            let text = core::mem::take(&mut self.text_buffer);
                            let text_node = DomNode::new_text(text);
                            if let Ok(_) = tree.append_child(current_parent, text_node) {}
                        }

                        // Check for comment
                        if pos + 3 < len && chars[pos + 1] == '!' && chars[pos + 2] == '-' && chars[pos + 3] == '-' {
                            self.state = ParserState::Comment;
                            pos += 3;
                        } else if pos + 1 < len && chars[pos + 1] == '/' {
                            self.state = ParserState::CloseTag;
                            self.is_closing = true;
                            pos += 1;
                        } else {
                            self.state = ParserState::OpenTag;
                            self.is_closing = false;
                        }
                        self.current_tag.clear();
                    } else {
                        self.text_buffer.push(ch);
                    }
                }

                ParserState::OpenTag | ParserState::CloseTag => {
                    if ch == '>' {
                        self.finish_tag(
                            &mut tree,
                            &mut parent_stack,
                            &mut current_parent,
                            head_id,
                            body_id,
                        )?;
                        self.state = ParserState::Text;
                    } else if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                        if !self.current_tag.is_empty() && !self.is_closing {
                            self.state = ParserState::AttrName;
                            self.current_attr_name.clear();
                            self.current_attr_value.clear();
                        }
                    } else if ch == '/' && !self.is_closing {
                        // Self-closing tag marker (e.g. <br/>)
                        self.is_closing = true;
                    } else {
                        self.current_tag.push(ch.to_ascii_lowercase());
                    }
                }

                ParserState::AttrName => {
                    if ch == '=' {
                        self.state = ParserState::AttrValue;
                        self.current_attr_value.clear();
                        self.attr_quoted = false;
                    } else if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                        // Attribute without value (boolean attribute)
                        if !self.current_attr_name.is_empty() {
                            let name = core::mem::take(&mut self.current_attr_name);
                            self.pending_attrs.push((name, String::from("")));
                        }
                    } else if ch == '>' {
                        // Flush pending boolean attribute
                        if !self.current_attr_name.is_empty() {
                            let name = core::mem::take(&mut self.current_attr_name);
                            self.pending_attrs.push((name, String::from("")));
                        }
                        self.finish_tag(
                            &mut tree,
                            &mut parent_stack,
                            &mut current_parent,
                            head_id,
                            body_id,
                        )?;
                        self.state = ParserState::Text;
                    } else {
                        self.current_attr_name.push(ch.to_ascii_lowercase());
                    }
                }

                ParserState::AttrValue => {
                    if ch == '"' || ch == '\'' {
                        if self.attr_quoted {
                            // End of quoted value
                            let name = core::mem::take(&mut self.current_attr_name);
                            let value = core::mem::take(&mut self.current_attr_value);
                            self.pending_attrs.push((name, value));
                            self.state = ParserState::AttrName;
                            self.current_attr_name.clear();
                        } else {
                            self.attr_quoted = true;
                        }
                    } else if !self.attr_quoted && (ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r') {
                        // End of unquoted value
                        let name = core::mem::take(&mut self.current_attr_name);
                        let value = core::mem::take(&mut self.current_attr_value);
                        self.pending_attrs.push((name, value));
                        self.state = ParserState::AttrName;
                        self.current_attr_name.clear();
                    } else if !self.attr_quoted && ch == '>' {
                        let name = core::mem::take(&mut self.current_attr_name);
                        let value = core::mem::take(&mut self.current_attr_value);
                        self.pending_attrs.push((name, value));
                        self.finish_tag(
                            &mut tree,
                            &mut parent_stack,
                            &mut current_parent,
                            head_id,
                            body_id,
                        )?;
                        self.state = ParserState::Text;
                    } else {
                        self.current_attr_value.push(ch);
                    }
                }

                ParserState::Comment => {
                    if ch == '-' && pos + 1 < len && chars[pos + 1] == '-' && pos + 2 < len && chars[pos + 2] == '>' {
                        let comment_text = core::mem::take(&mut self.text_buffer);
                        let comment_node = DomNode::new_comment(comment_text);
                        if let Ok(_) = tree.append_child(current_parent, comment_node) {}
                        pos += 2;
                        self.state = ParserState::Text;
                    } else {
                        self.text_buffer.push(ch);
                    }
                }

                ParserState::Eof => break,
            }

            pos += 1;
        }

        // Flush any remaining text
        if !self.text_buffer.is_empty() {
            let text = core::mem::take(&mut self.text_buffer);
            let text_node = DomNode::new_text(text);
            if let Ok(_) = tree.append_child(current_parent, text_node) {}
        }

        Ok(tree)
    }

    /// Finish parsing the current tag and update the DOM tree
    fn finish_tag(
        &mut self,
        tree: &mut DomTree,
        parent_stack: &mut Vec<NodeId>,
        current_parent: &mut NodeId,
        _head_id: NodeId,
        _body_id: NodeId,
    ) -> Result<(), WebError> {
        let tag = core::mem::take(&mut self.current_tag);
        let attrs = core::mem::take(&mut self.pending_attrs);

        if self.is_closing {
            // Close tag: pop matching element from stack
            let mut found = false;
            for i in (0..self.open_stack.len()).rev() {
                if self.open_stack[i] == tag {
                    self.open_stack.truncate(i);
                    if parent_stack.len() > i + 1 {
                        parent_stack.truncate(i + 1);
                    }
                    if let Some(&p) = parent_stack.last() {
                        *current_parent = p;
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                // No matching open tag; ignore this close tag (forgiving parsing)
            }
        } else {
            // Open tag: create element and push onto stack
            let mut node = DomNode::new_element(tag.clone());
            for (name, value) in attrs {
                node.set_attribute(name, value);
            }

            let node_id = tree.append_child(*current_parent, node)?;

            let is_void = VOID_ELEMENTS.contains(&tag.as_str());
            if !is_void {
                self.open_stack.push(tag);
                parent_stack.push(node_id);
                *current_parent = node_id;
            }
        }

        self.is_closing = false;
        self.current_attr_name.clear();
        self.current_attr_value.clear();
        Ok(())
    }
}

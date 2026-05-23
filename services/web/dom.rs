/*
 * Nuva OS - SystemService - Web - DOM
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

//! DOM tree construction and manipulation interface.
//! Supports node creation, insertion, removal, attribute operations,
//! and event binding for the web rendering pipeline.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use super::error::WebError;

/// Global node ID counter
static NEXT_NODE_ID: AtomicU64 = AtomicU64::new(1);

/// Unique DOM node identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(pub u64);

/// DOM node type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Element node (e.g. <div>, <p>)
    Element = 1,
    /// Text node
    Text = 3,
    /// Comment node
    Comment = 8,
    /// Document node (root)
    Document = 9,
    /// Document fragment
    DocumentFragment = 11,
}

/// Event listener callback identifier (opaque handle)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventListenerId(pub u64);

/// DOM event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomEventType {
    /// Mouse click
    Click,
    /// Mouse double click
    DblClick,
    /// Mouse down
    MouseDown,
    /// Mouse up
    MouseUp,
    /// Mouse move
    MouseMove,
    /// Key down
    KeyDown,
    /// Key up
    KeyUp,
    /// Focus gained
    Focus,
    /// Focus lost
    Blur,
    /// Input value changed
    Input,
    /// Form submitted
    Submit,
    /// Value changed
    Change,
    /// Element scrolled
    Scroll,
    /// Custom event
    Custom,
}

/// Event listener registration
#[derive(Debug, Clone)]
pub struct EventListener {
    /// Listener ID
    pub id: EventListenerId,
    /// Event type
    pub event_type: DomEventType,
    /// Owner JS context ID
    pub js_context_id: u64,
}

/// DOM node data
#[derive(Debug, Clone)]
pub struct DomNode {
    /// Unique node ID
    pub id: NodeId,
    /// Node type
    pub node_type: NodeType,
    /// Tag name for element nodes (e.g. "div", "span")
    pub tag_name: String,
    /// Text content for text/comment nodes
    pub text_content: String,
    /// Attributes for element nodes
    pub attributes: BTreeMap<String, String>,
    /// Child node IDs (ordered)
    pub children: Vec<NodeId>,
    /// Parent node ID (None for root)
    pub parent: Option<NodeId>,
    /// Registered event listeners
    pub event_listeners: Vec<EventListener>,
    /// Whether node is connected to the document
    pub connected: bool,
}

impl DomNode {
    /// Create a new element node
    pub fn new_element(tag_name: String) -> Self {
        let id = NodeId(NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed));
        DomNode {
            id,
            node_type: NodeType::Element,
            tag_name,
            text_content: String::new(),
            attributes: BTreeMap::new(),
            children: Vec::new(),
            parent: None,
            event_listeners: Vec::new(),
            connected: false,
        }
    }

    /// Create a new text node
    pub fn new_text(content: String) -> Self {
        let id = NodeId(NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed));
        DomNode {
            id,
            node_type: NodeType::Text,
            tag_name: String::new(),
            text_content: content,
            attributes: BTreeMap::new(),
            children: Vec::new(),
            parent: None,
            event_listeners: Vec::new(),
            connected: false,
        }
    }

    /// Create a new comment node
    pub fn new_comment(content: String) -> Self {
        let id = NodeId(NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed));
        DomNode {
            id,
            node_type: NodeType::Comment,
            tag_name: String::new(),
            text_content: content,
            attributes: BTreeMap::new(),
            children: Vec::new(),
            parent: None,
            event_listeners: Vec::new(),
            connected: false,
        }
    }

    /// Create the document root node
    pub fn new_document() -> Self {
        DomNode {
            id: NodeId(0),
            node_type: NodeType::Document,
            tag_name: String::new(),
            text_content: String::new(),
            attributes: BTreeMap::new(),
            children: Vec::new(),
            parent: None,
            event_listeners: Vec::new(),
            connected: true,
        }
    }

    /// Create a document fragment
    pub fn new_document_fragment() -> Self {
        let id = NodeId(NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed));
        DomNode {
            id,
            node_type: NodeType::DocumentFragment,
            tag_name: String::new(),
            text_content: String::new(),
            attributes: BTreeMap::new(),
            children: Vec::new(),
            parent: None,
            event_listeners: Vec::new(),
            connected: false,
        }
    }

    /// Get an attribute value by name
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    /// Set an attribute value
    pub fn set_attribute(&mut self, name: String, value: String) {
        self.attributes.insert(name, value);
    }

    /// Remove an attribute by name
    pub fn remove_attribute(&mut self, name: &str) -> Option<String> {
        self.attributes.remove(name)
    }

    /// Check if this node has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get the first child node ID
    pub fn first_child(&self) -> Option<NodeId> {
        self.children.first().copied()
    }

    /// Get the last child node ID
    pub fn last_child(&self) -> Option<NodeId> {
        self.children.last().copied()
    }
}

/// DOM tree manager
#[derive(Debug)]
pub struct DomTree {
    /// All nodes indexed by NodeId
    nodes: BTreeMap<u64, DomNode>,
    /// Document root node ID
    document_id: NodeId,
    /// Next event listener ID
    next_listener_id: u64,
}

impl DomTree {
    /// Create a new DOM tree with a document root
    pub fn new() -> Self {
        let doc = DomNode::new_document();
        let doc_id = doc.id;
        let mut nodes = BTreeMap::new();
        nodes.insert(doc_id.0, doc);
        DomTree {
            nodes,
            document_id: doc_id,
            next_listener_id: 1,
        }
    }

    /// Get the document root node ID
    pub fn document(&self) -> NodeId {
        self.document_id
    }

    /// Get a node by ID
    pub fn get_node(&self, id: NodeId) -> Option<&DomNode> {
        self.nodes.get(&id.0)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut DomNode> {
        self.nodes.get_mut(&id.0)
    }

    /// Insert a node as the last child of a parent
    pub fn append_child(&mut self, parent_id: NodeId, child: DomNode) -> Result<NodeId, WebError> {
        if !self.nodes.contains_key(&parent_id.0) {
            return Err(WebError::InvalidArgument);
        }

        let child_id = child.id;
        let mut child = child;
        child.parent = Some(parent_id);
        child.connected = true;
        self.nodes.insert(child_id.0, child);

        if let Some(parent) = self.nodes.get_mut(&parent_id.0) {
            parent.children.push(child_id);
        }

        Ok(child_id)
    }

    /// Insert a node before a reference sibling
    pub fn insert_before(
        &mut self,
        parent_id: NodeId,
        new_child: DomNode,
        ref_child_id: NodeId,
    ) -> Result<NodeId, WebError> {
        if !self.nodes.contains_key(&parent_id.0) {
            return Err(WebError::InvalidArgument);
        }

        let child_id = new_child.id;
        let mut new_child = new_child;
        new_child.parent = Some(parent_id);
        new_child.connected = true;
        self.nodes.insert(child_id.0, new_child);

        if let Some(parent) = self.nodes.get_mut(&parent_id.0) {
            if let Some(pos) = parent.children.iter().position(|&c| c == ref_child_id) {
                parent.children.insert(pos, child_id);
            } else {
                parent.children.push(child_id);
            }
        }

        Ok(child_id)
    }

    /// Remove a child node from its parent
    pub fn remove_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<DomNode, WebError> {
        if let Some(parent) = self.nodes.get_mut(&parent_id.0) {
            parent.children.retain(|&c| c != child_id);
        } else {
            return Err(WebError::InvalidArgument);
        }

        if let Some(mut child) = self.nodes.remove(&child_id.0) {
            child.parent = None;
            child.connected = false;
            Ok(child)
        } else {
            Err(WebError::ResourceNotFound)
        }
    }

    /// Set the text content of a node
    pub fn set_text_content(&mut self, id: NodeId, text: String) -> Result<(), WebError> {
        if let Some(node) = self.nodes.get_mut(&id.0) {
            node.text_content = text;
            Ok(())
        } else {
            Err(WebError::ResourceNotFound)
        }
    }

    /// Set an attribute on an element node
    pub fn set_attribute(
        &mut self,
        id: NodeId,
        name: String,
        value: String,
    ) -> Result<(), WebError> {
        if let Some(node) = self.nodes.get_mut(&id.0) {
            if node.node_type != NodeType::Element {
                return Err(WebError::InvalidArgument);
            }
            node.set_attribute(name, value);
            Ok(())
        } else {
            Err(WebError::ResourceNotFound)
        }
    }

    /// Remove an attribute from an element node
    pub fn remove_attribute(&mut self, id: NodeId, name: &str) -> Result<Option<String>, WebError> {
        if let Some(node) = self.nodes.get_mut(&id.0) {
            if node.node_type != NodeType::Element {
                return Err(WebError::InvalidArgument);
            }
            Ok(node.remove_attribute(name))
        } else {
            Err(WebError::ResourceNotFound)
        }
    }

    /// Add an event listener to a node
    pub fn add_event_listener(
        &mut self,
        id: NodeId,
        event_type: DomEventType,
        js_context_id: u64,
    ) -> Result<EventListenerId, WebError> {
        let listener_id = EventListenerId(self.next_listener_id);
        self.next_listener_id += 1;

        if let Some(node) = self.nodes.get_mut(&id.0) {
            node.event_listeners.push(EventListener {
                id: listener_id,
                event_type,
                js_context_id,
            });
            Ok(listener_id)
        } else {
            Err(WebError::ResourceNotFound)
        }
    }

    /// Remove an event listener from a node
    pub fn remove_event_listener(&mut self, id: NodeId, listener_id: EventListenerId) -> Result<(), WebError> {
        if let Some(node) = self.nodes.get_mut(&id.0) {
            node.event_listeners.retain(|l| l.id != listener_id);
            Ok(())
        } else {
            Err(WebError::ResourceNotFound)
        }
    }

    /// Query the first element matching a tag name under a root
    pub fn query_by_tag(&self, root: NodeId, tag: &str) -> Option<NodeId> {
        self.query_by_tag_recursive(root, tag)
    }

    fn query_by_tag_recursive(&self, current: NodeId, tag: &str) -> Option<NodeId> {
        if let Some(node) = self.nodes.get(&current.0) {
            if node.node_type == NodeType::Element && node.tag_name == tag {
                return Some(current);
            }
            for &child_id in &node.children {
                if let Some(found) = self.query_by_tag_recursive(child_id, tag) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Get total node count in the tree
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

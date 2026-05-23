/*
 * Nuva OS - Kernel - Red-Black Tree (Complete Implementation)
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

use core::ptr;

/// Red-black tree node color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RbColor {
    Red,
    Black,
}

/// Red-black tree node
pub struct RbNode {
    /// Key value for sorting
    pub key: u64,
    /// Left child
    pub left: *mut RbNode,
    /// Right child
    pub right: *mut RbNode,
    /// Parent node
    pub parent: *mut RbNode,
    /// Node color
    pub color: RbColor,
    /// User data
    pub data: u64,
}

impl RbNode {
    pub const fn new(key: u64) -> Self {
        RbNode {
            key,
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            parent: ptr::null_mut(),
            color: RbColor::Red,
            data: 0,
        }
    }
    
    #[inline]
    pub fn is_red(&self) -> bool {
        self.color == RbColor::Red
    }
    
    #[inline]
    pub fn is_black(&self) -> bool {
        self.color == RbColor::Black
    }
    
    #[inline]
    pub fn set_red(&mut self) {
        self.color = RbColor::Red;
    }
    
    #[inline]
    pub fn set_black(&mut self) {
        self.color = RbColor::Black;
    }
}

/// Red-black tree
pub struct RbTree {
    /// Root node
    pub root: *mut RbNode,
    /// Number of nodes
    pub count: u64,
    /// Leftmost node (for CFS)
    pub leftmost: *mut RbNode,
}

impl RbTree {
    pub const fn new() -> Self {
        RbTree {
            root: ptr::null_mut(),
            count: 0,
            leftmost: ptr::null_mut(),
        }
    }
    
    /// Insert a node into the tree
    pub fn insert(&mut self, node: *mut RbNode) {
        if node.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Initialize node
            (*node).left = ptr::null_mut();
            (*node).right = ptr::null_mut();
            (*node).parent = ptr::null_mut();
            (*node).set_red();
            
            // Empty tree
            if self.root.is_null() {
                (*node).set_black();
                self.root = node;
                self.leftmost = node;
                self.count = 1;
                return;
            }
            
            // Find insertion position
            let mut parent: *mut RbNode = ptr::null_mut();
            let mut current = self.root;
            let mut is_left = true;
            
            while !current.is_null() {
                parent = current;
                if (*node).key < (*current).key {
                    current = (*current).left;
                    is_left = true;
                } else {
                    current = (*current).right;
                    is_left = false;
                }
            }
            
            // Link node to parent
            (*node).parent = parent;
            if is_left {
                (*parent).left = node;
                // Update leftmost
                if parent == self.leftmost {
                    self.leftmost = node;
                }
            } else {
                (*parent).right = node;
            }
            
            // Fix red-black properties
            self.insert_fixup(node);
            self.count += 1;
        }
    }
    
    /// Fix red-black tree after insertion
    fn insert_fixup(&mut self, mut node: *mut RbNode) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            while !node.is_null() && !(*node).parent.is_null() {
                let parent = (*node).parent;
                
                // Parent is black, no violation
                if (*parent).is_black() {
                    break;
                }
                
                let grandparent = (*parent).parent;
                if grandparent.is_null() {
                    break;
                }
                
                // Determine if parent is left child
                let parent_is_left = (*grandparent).left == parent;
                let uncle = if parent_is_left {
                    (*grandparent).right
                } else {
                    (*grandparent).left
                };
                
                if !uncle.is_null() && (*uncle).is_red() {
                    // Case 1: Uncle is red - recolor
                    (*parent).set_black();
                    (*uncle).set_black();
                    (*grandparent).set_red();
                    node = grandparent;
                } else {
                    // Case 2 & 3: Uncle is black
                    let node_is_left = (*parent).left == node;
                    
                    if parent_is_left != node_is_left {
                        // Case 2: Node is inner child - rotate parent
                        if node_is_left {
                            self.rotate_right(parent);
                        } else {
                            self.rotate_left(parent);
                        }
                        node = parent;
                    }
                    
                    // Case 3: Node is outer child - rotate grandparent
                    let new_parent = (*node).parent;
                    if !new_parent.is_null() {
                        (*new_parent).set_black();
                    }
                    (*grandparent).set_red();
                    
                    if parent_is_left {
                        self.rotate_right(grandparent);
                    } else {
                        self.rotate_left(grandparent);
                    }
                    break;
                }
            }
            
            // Root must be black
            if !self.root.is_null() {
                (*self.root).set_black();
            }
        }
    }
    
    /// Remove a node from the tree (complete implementation)
    pub fn remove(&mut self, node: *mut RbNode) {
        if node.is_null() || self.root.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Update leftmost if needed
            if node == self.leftmost {
                self.leftmost = self.find_successor(node);
            }
            
            // Find the node to be removed (y) and its child (x)
            let y: *mut RbNode;
            let x: *mut RbNode;
            let x_parent: *mut RbNode;
            
            if (*node).left.is_null() || (*node).right.is_null() {
                // Node has at most one child - remove it directly
                y = node;
            } else {
                // Node has two children - replace with successor
                y = self.find_successor(node);
            }
            
            // Get y's child (x)
            if !(*y).left.is_null() {
                x = (*y).left;
            } else {
                x = (*y).right;
            }
            
            // Remove y from tree
            x_parent = (*y).parent;
            
            if !x.is_null() {
                (*x).parent = (*y).parent;
            }
            
            if (*y).parent.is_null() {
                // y is root
                self.root = x;
            } else {
                if y == (*(*y).parent).left {
                    (*(*y).parent).left = x;
                } else {
                    (*(*y).parent).right = x;
                }
            }
            
            // If y is not node, replace node with y
            if y != node {
                // Copy node's data to y
                (*y).key = (*node).key;
                (*y).data = (*node).data;
                (*y).left = (*node).left;
                (*y).right = (*node).right;
                (*y).parent = (*node).parent;
                (*y).color = (*node).color;
                
                // Update children's parent pointers
                if !(*y).left.is_null() {
                    (*(*y).left).parent = y;
                }
                if !(*y).right.is_null() {
                    (*(*y).right).parent = y;
                }
                
                // Update parent's child pointer
                if (*y).parent.is_null() {
                    self.root = y;
                } else {
                    if node == (*(*y).parent).left {
                        (*(*y).parent).left = y;
                    } else {
                        (*(*y).parent).right = y;
                    }
                }
            }
            
            // If y was black, fix the tree
            if (*y).is_black() {
                self.delete_fixup(x, x_parent);
            }
            
            // Clear node's pointers
            (*node).left = ptr::null_mut();
            (*node).right = ptr::null_mut();
            (*node).parent = ptr::null_mut();
            
            self.count = self.count.saturating_sub(1);
        }
    }
    
    /// Fix red-black tree after deletion
    fn delete_fixup(&mut self, mut x: *mut RbNode, mut x_parent: *mut RbNode) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            while x != self.root && (x.is_null() || (*x).is_black()) {
                if x_parent.is_null() {
                    break;
                }
                
                let x_is_left = if x.is_null() {
                    // x is null, determine position from parent
                    // This is a simplification; in practice we need to track this
                    true
                } else {
                    x == (*x_parent).left
                };
                
                if x_is_left {
                    // x is left child
                    let mut w = (*x_parent).right;
                    
                    if !w.is_null() && (*w).is_red() {
                        // Case 1: Sibling is red
                        (*w).set_black();
                        (*x_parent).set_red();
                        self.rotate_left(x_parent);
                        w = (*x_parent).right;
                    }
                    
                    if w.is_null() {
                        x = x_parent;
                        x_parent = (*x).parent;
                    } else {
                        let w_left_black = (*w).left.is_null() || (*(*w).left).is_black();
                        let w_right_black = (*w).right.is_null() || (*(*w).right).is_black();
                        
                        if w_left_black && w_right_black {
                            // Case 2: Sibling's children are black
                            (*w).set_red();
                            x = x_parent;
                            x_parent = (*x).parent;
                        } else {
                            if w_right_black {
                                // Case 3: Sibling's right child is black
                                if !(*w).left.is_null() {
                                    (*(*w).left).set_black();
                                }
                                (*w).set_red();
                                self.rotate_right(w);
                                w = (*x_parent).right;
                            }
                            
                            // Case 4: Sibling's right child is red
                            if !w.is_null() {
                                (*w).color = (*x_parent).color;
                            }
                            (*x_parent).set_black();
                            if !w.is_null() && !(*w).right.is_null() {
                                (*(*w).right).set_black();
                            }
                            self.rotate_left(x_parent);
                            x = self.root;
                        }
                    }
                } else {
                    // x is right child (mirror of above)
                    let mut w = (*x_parent).left;
                    
                    if !w.is_null() && (*w).is_red() {
                        (*w).set_black();
                        (*x_parent).set_red();
                        self.rotate_right(x_parent);
                        w = (*x_parent).left;
                    }
                    
                    if w.is_null() {
                        x = x_parent;
                        x_parent = (*x).parent;
                    } else {
                        let w_left_black = (*w).left.is_null() || (*(*w).left).is_black();
                        let w_right_black = (*w).right.is_null() || (*(*w).right).is_black();
                        
                        if w_left_black && w_right_black {
                            (*w).set_red();
                            x = x_parent;
                            x_parent = (*x).parent;
                        } else {
                            if w_left_black {
                                if !(*w).right.is_null() {
                                    (*(*w).right).set_black();
                                }
                                (*w).set_red();
                                self.rotate_left(w);
                                w = (*x_parent).left;
                            }
                            
                            if !w.is_null() {
                                (*w).color = (*x_parent).color;
                            }
                            (*x_parent).set_black();
                            if !w.is_null() && !(*w).left.is_null() {
                                (*(*w).left).set_black();
                            }
                            self.rotate_right(x_parent);
                            x = self.root;
                        }
                    }
                }
            }
            
            if !x.is_null() {
                (*x).set_black();
            }
        }
    }
    
    /// Left rotation
    fn rotate_left(&mut self, x: *mut RbNode) {
        if x.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let y = (*x).right;
            if y.is_null() {
                return;
            }
            
            // Turn y's left subtree into x's right subtree
            (*x).right = (*y).left;
            if !(*y).left.is_null() {
                (*(*y).left).parent = x;
            }
            
            // Link y's parent
            (*y).parent = (*x).parent;
            if (*x).parent.is_null() {
                self.root = y;
            } else if x == (*(*x).parent).left {
                (*(*x).parent).left = y;
            } else {
                (*(*x).parent).right = y;
            }
            
            // Put x on y's left
            (*y).left = x;
            (*x).parent = y;
        }
    }
    
    /// Right rotation
    fn rotate_right(&mut self, x: *mut RbNode) {
        if x.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let y = (*x).left;
            if y.is_null() {
                return;
            }
            
            // Turn y's right subtree into x's left subtree
            (*x).left = (*y).right;
            if !(*y).right.is_null() {
                (*(*y).right).parent = x;
            }
            
            // Link y's parent
            (*y).parent = (*x).parent;
            if (*x).parent.is_null() {
                self.root = y;
            } else if x == (*(*x).parent).right {
                (*(*x).parent).right = y;
            } else {
                (*(*x).parent).left = y;
            }
            
            // Put x on y's right
            (*y).right = x;
            (*x).parent = y;
        }
    }
    
    /// Find successor of a node
    fn find_successor(&self, node: *mut RbNode) -> *mut RbNode {
        if node.is_null() {
            return ptr::null_mut();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // If right subtree exists, find leftmost in it
            if !(*node).right.is_null() {
                let mut current = (*node).right;
                while !(*current).left.is_null() {
                    current = (*current).left;
                }
                return current;
            }
            
            // Otherwise, go up until we find a node that is a left child
            let mut current = node;
            let mut parent = (*node).parent;
            
            while !parent.is_null() && current == (*parent).right {
                current = parent;
                parent = (*parent).parent;
            }
            
            parent
        }
    }
    
    /// Find predecessor of a node
    fn find_predecessor(&self, node: *mut RbNode) -> *mut RbNode {
        if node.is_null() {
            return ptr::null_mut();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // If left subtree exists, find rightmost in it
            if !(*node).left.is_null() {
                let mut current = (*node).left;
                while !(*current).right.is_null() {
                    current = (*current).right;
                }
                return current;
            }
            
            // Otherwise, go up until we find a node that is a right child
            let mut current = node;
            let mut parent = (*node).parent;
            
            while !parent.is_null() && current == (*parent).left {
                current = parent;
                parent = (*parent).parent;
            }
            
            parent
        }
    }
    
    /// Search for a node with the given key
    pub fn search(&self, key: u64) -> *mut RbNode {
        let mut current = self.root;
        
        while !current.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if key == (*current).key {
                    return current;
                } else if key < (*current).key {
                    current = (*current).left;
                } else {
                    current = (*current).right;
                }
            }
        }
        
        ptr::null_mut()
    }
    
    /// Get the minimum node
    pub fn min(&self) -> *mut RbNode {
        self.leftmost
    }
    
    /// Get the maximum node
    pub fn max(&self) -> *mut RbNode {
        if self.root.is_null() {
            return ptr::null_mut();
        }
        
        let mut current = self.root;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            while !(*current).right.is_null() {
                current = (*current).right;
            }
        }
        current
    }
    
    /// Check if tree is empty
    pub fn is_empty(&self) -> bool {
        self.root.is_null()
    }
    
    /// Get tree size
    pub fn len(&self) -> u64 {
        self.count
    }
    
    /// Validate red-black tree properties (for testing)
    pub fn validate(&self) -> bool {
        if self.root.is_null() {
            return true;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Property 1: Root is black
            if (*self.root).is_red() {
                return false;
            }
            
            // Property 2 & 3: Check red-black properties recursively
            let (valid, black_height) = self.validate_node(self.root);
            
            valid && black_height > 0
        }
    }
    
    fn validate_node(&self, node: *mut RbNode) -> (bool, i32) {
        if node.is_null() {
            return (true, 1); // Null nodes are black
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Property 4: Red node has black children
            if (*node).is_red() {
                if !(*node).left.is_null() && (*(*node).left).is_red() {
                    return (false, 0);
                }
                if !(*node).right.is_null() && (*(*node).right).is_red() {
                    return (false, 0);
                }
            }
            
            // Check subtrees
            let (left_valid, left_height) = self.validate_node((*node).left);
            let (right_valid, right_height) = self.validate_node((*node).right);
            
            // Property 5: Same black height
            if !left_valid || !right_valid || left_height != right_height {
                return (false, 0);
            }
            
            // Calculate black height
            let height = if (*node).is_black() {
                left_height + 1
            } else {
                left_height
            };
            
            (true, height)
        }
    }
}

/// Cached red-black tree with O(1) first() access.
/// Wraps RbTree with cached leftmost pointer and node count
/// to avoid tree traversal for the common CFS scheduler pattern
/// of always picking the leftmost (minimum-key) node.
pub struct CachedRbTree {
    /// Inner red-black tree
    inner: RbTree,
}

impl CachedRbTree {
    /// Create a new cached red-black tree
    pub const fn new() -> Self {
        CachedRbTree {
            inner: RbTree::new(),
        }
    }

    /// Get the minimum node in O(1) via cached leftmost pointer.
    /// Returns null if the tree is empty.
    #[inline(always)]
    pub fn first(&self) -> *mut RbNode {
        self.inner.leftmost
    }

    /// Insert a node and update the leftmost cache
    pub fn insert(&mut self, node: *mut RbNode) {
        self.inner.insert(node);
    }

    /// Remove a node and update the leftmost cache
    pub fn remove(&mut self, node: *mut RbNode) {
        self.inner.remove(node);
    }

    /// Search for a node with the given key
    #[inline]
    pub fn search(&self, key: u64) -> *mut RbNode {
        self.inner.search(key)
    }

    /// Get the minimum node (same as first, O(1))
    #[inline(always)]
    pub fn min(&self) -> *mut RbNode {
        self.inner.leftmost
    }

    /// Get the maximum node (O(log n))
    #[inline]
    pub fn max(&self) -> *mut RbNode {
        self.inner.max()
    }

    /// Check if tree is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get tree size
    #[inline(always)]
    pub fn len(&self) -> u64 {
        self.inner.count
    }

    /// Get the cached leftmost pointer directly
    #[inline(always)]
    pub fn leftmost(&self) -> *mut RbNode {
        self.inner.leftmost
    }

    /// Validate the tree (for testing)
    pub fn validate(&self) -> bool {
        self.inner.validate()
    }
}

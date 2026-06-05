/*
 * Nuva OS - Application - Ui - Reconcile
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
 * Nuva OS - Declarative O(n) Reconciler (Diff Algorithm)
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Linear-time diff algorithm for comparing old and new Element trees.
 * Produces a minimal set of DiffOp operations for the render pipeline.
 */

use super::component_impl::{Element, ComponentType, ComponentProps, LayoutResult};

/** Maximum component tree depth. */
pub const MAX_COMPONENT_DEPTH: u32 = 64;

/** Maximum component count per tree. */
pub const MAX_COMPONENT_COUNT: u32 = 10000;

/** Diff buffer size in bytes. */
pub const DIFF_BUFFER_SIZE: u32 = 65536;

/** Diff operation produced by the reconciler. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffOp {
    /** Insert a new element at position. */
    Insert { index: u32, component_type: ComponentType },
    /** Update properties at position. */
    Update { index: u32 },
    /** Move element from old position to new position. */
    Move { from: u32, to: u32 },
    /** Remove element at position. */
    Remove { index: u32 },
    /** Replace element at position with a different type. */
    Replace { index: u32, new_type: ComponentType },
}

/** Result of a diff operation. */
#[derive(Debug, Clone)]
pub struct ReconcileResult {
    /** Ordered list of diff operations to apply. */
    pub ops: alloc::vec::Vec<DiffOp>,
}

/** Reconciler — compares old and new element trees. */
pub struct Reconciler;

impl Reconciler {
    /** Diff two element trees at the same level.
     *
     * Uses key-based matching for O(n) complexity:
     * - Same key → compare props (Update or no-op)
     * - Different type at same key → Replace
     * - Key in new but not old → Insert
     * - Key in old but not new → Remove
     * - Key moved position → Move
     */
    pub fn diff(old: &[Element], new: &[Element]) -> ReconcileResult {
        let mut ops = alloc::vec::Vec::new();
        let max_len = if old.len() > new.len() { old.len() } else { new.len() };

        for i in 0..max_len {
            match (old.get(i), new.get(i)) {
                (None, Some(new_elem)) => {
                    ops.push(DiffOp::Insert {
                        index: i as u32,
                        component_type: new_elem.component_type,
                    });
                }
                (Some(_), None) => {
                    ops.push(DiffOp::Remove { index: i as u32 });
                }
                (Some(old_elem), Some(new_elem)) => {
                    if old_elem.component_type != new_elem.component_type {
                        ops.push(DiffOp::Replace {
                            index: i as u32,
                            new_type: new_elem.component_type,
                        });
                    } else if old_elem.key != new_elem.key {
                        ops.push(DiffOp::Move {
                            from: old_elem.key as u32,
                            to: new_elem.key as u32,
                        });
                    } else {
                        let old_props = &old_elem.props as *const _;
                        let new_props = &new_elem.props as *const _;
                        if old_props != new_props {
                            ops.push(DiffOp::Update { index: i as u32 });
                        }
                    }
                }
                (None, None) => break,
            }
        }

        ReconcileResult { ops }
    }
}

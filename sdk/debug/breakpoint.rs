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

// ! Breakpoint management with conditional, data, and function breakpoints

use std::collections::HashMap;
use crate::error::SdkError;
use super::target::DebugTarget;

/// Breakpoint manager
pub struct BreakpointManager {
    /// Breakpoint list
    breakpoints: HashMap<u32, Breakpoint>,
    /// Next breakpoint ID
    next_id: u32,
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            next_id: 1,
        }
    }

    /// Set a breakpoint in the target
    pub fn set(&mut self, target: &mut DebugTarget, location: BreakpointLocation) -> Result<Breakpoint, SdkError> {
        let id = self.next_id;
        self.next_id += 1;

        match &location {
            BreakpointLocation::Line { file, line } => {
                if !std::path::Path::new(file).exists() {
                    return Err(SdkError::FileNotFound(format!("{}:{}", file, line)));
                }
            }
            BreakpointLocation::Address { address } => {
                if *address == 0 {
                    return Err(SdkError::InvalidArgument("Address cannot be zero".to_string()));
                }
            }
            BreakpointLocation::Function { name } => {
                if name.is_empty() {
                    return Err(SdkError::InvalidArgument("Function name cannot be empty".to_string()));
                }
            }
            BreakpointLocation::Watch { expression, .. } => {
                if expression.is_empty() {
                    return Err(SdkError::InvalidArgument("Watch expression cannot be empty".to_string()));
                }
            }
        }

        let bp = Breakpoint {
            id,
            location,
            enabled: true,
            condition: None,
            hit_count: 0,
            hit_condition: None,
            log_message: None,
        };

        self.breakpoints.insert(id, bp.clone());
        Ok(bp)
    }

    /// Remove a breakpoint
    pub fn remove(&mut self, id: u32) -> Result<(), SdkError> {
        self.breakpoints.remove(&id);
        Ok(())
    }

    /// Enable a breakpoint
    pub fn enable(&mut self, id: u32) -> Result<(), SdkError> {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.enabled = true;
        }
        Ok(())
    }

    /// Disable a breakpoint
    pub fn disable(&mut self, id: u32) -> Result<(), SdkError> {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.enabled = false;
        }
        Ok(())
    }

    /// Set condition for conditional breakpoint
    pub fn set_condition(&mut self, id: u32, condition: String) -> Result<(), SdkError> {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.condition = Some(condition);
        }
        Ok(())
    }

    /// Set hit condition (e.g., ">5" means break after 5 hits)
    pub fn set_hit_condition(&mut self, id: u32, hit_condition: String) -> Result<(), SdkError> {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.hit_condition = Some(hit_condition);
        }
        Ok(())
    }

    /// Set log message for logpoint
    pub fn set_log_message(&mut self, id: u32, log_message: String) -> Result<(), SdkError> {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.log_message = Some(log_message);
        }
        Ok(())
    }

    /// Get breakpoint by ID
    pub fn get(&self, id: u32) -> Option<&Breakpoint> {
        self.breakpoints.get(&id)
    }

    /// Get all breakpoints
    pub fn all(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// Get enabled breakpoints
    pub fn enabled(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().filter(|bp| bp.enabled).collect()
    }

    /// Get breakpoints by file
    pub fn by_file(&self, file: &str) -> Vec<&Breakpoint> {
        self.breakpoints.values()
            .filter(|bp| match &bp.location {
                BreakpointLocation::Line { file: f, .. } => f == file,
                _ => false,
            })
            .collect()
    }

    /// Increment hit count
    pub fn hit(&mut self, id: u32) {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.hit_count += 1;
        }
    }

    /// Check if breakpoint should trigger based on condition and hit count
    pub fn should_trigger(&self, id: u32) -> bool {
        if let Some(bp) = self.breakpoints.get(id) {
            if !bp.enabled {
                return false;
            }

            if let Some(ref hit_cond) = bp.hit_condition {
                return evaluate_hit_condition(hit_cond, bp.hit_count);
            }

            return true;
        }
        false
    }

    /// Get breakpoint count
    pub fn count(&self) -> usize {
        self.breakpoints.len()
    }

    /// Remove all breakpoints
    pub fn clear(&mut self) {
        self.breakpoints.clear();
    }
}

/// Evaluate hit condition expression
fn evaluate_hit_condition(condition: &str, hit_count: u32) -> bool {
    if condition.is_empty() {
        return true;
    }

    if let Some(rest) = condition.strip_prefix('>') {
        if let Ok(threshold) = rest.parse::<u32>() {
            return hit_count > threshold;
        }
    }

    if let Some(rest) = condition.strip_prefix('=') {
        if let Ok(threshold) = rest.parse::<u32>() {
            return hit_count == threshold;
        }
    }

    if let Some(rest) = condition.strip_prefix('%') {
        if let Ok(modulus) = rest.parse::<u32>() {
            if modulus > 0 {
                return hit_count % modulus == 0;
            }
        }
    }

    if let Ok(threshold) = condition.parse::<u32>() {
        return hit_count == threshold;
    }

    true
}

impl Default for BreakpointManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Breakpoint
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Breakpoint ID
    pub id: u32,
    /// Location
    pub location: BreakpointLocation,
    /// Whether enabled
    pub enabled: bool,
    /// Condition for conditional breakpoint
    pub condition: Option<String>,
    /// Hit count
    pub hit_count: u32,
    /// Hit condition (e.g., ">5")
    pub hit_condition: Option<String>,
    /// Log message for logpoint
    pub log_message: Option<String>,
}

/// Breakpoint location
#[derive(Debug, Clone)]
pub enum BreakpointLocation {
    /// Line breakpoint
    Line {
        file: String,
        line: u32,
    },
    /// Function breakpoint
    Function {
        name: String,
    },
    /// Address breakpoint
    Address {
        address: u64,
    },
    /// Data watchpoint
    Watch {
        expression: String,
        watch_type: WatchType,
    },
}

/// Watch type for data breakpoints
#[derive(Debug, Clone, Copy)]
pub enum WatchType {
    Read,
    Write,
    ReadWrite,
}

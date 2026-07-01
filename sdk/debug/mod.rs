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

// ! Debug module

pub mod target;
pub mod breakpoint;
pub mod execution;
pub mod variable;
pub mod memory;
pub mod stack;
pub mod dap;

use crate::error::SdkError;
use alloc::format;
use alloc::vec::Vec;

/// Debugger
pub struct Debugger {
    /// Debug target
    target: Option<target::DebugTarget>,
    /// Breakpoint manager
    breakpoints: breakpoint::BreakpointManager,
    /// Execution controller
    execution: execution::ExecutionController,
}

impl Debugger {
    /// Create new debugger
    pub fn new() -> Self {
        Self {
            target: None,
            breakpoints: breakpoint::BreakpointManager::new(),
            execution: execution::ExecutionController::new(),
        }
    }

    /// Launch program for debugging
    pub fn launch(&mut self, program: &str, args: &[String]) -> Result<(), SdkError> {
        let target = target::DebugTarget::launch(program, args)?;
        self.target = Some(target);
        Ok(())
    }

    /// Attach to running process
    pub fn attach(&mut self, pid: u32) -> Result<(), SdkError> {
        let target = target::DebugTarget::attach(pid)?;
        self.target = Some(target);
        Ok(())
    }

    /// Set breakpoint by location
    pub fn set_breakpoint(&mut self, location: breakpoint::BreakpointLocation) -> Result<breakpoint::Breakpoint, SdkError> {
        let target = self.target.as_mut()
            .ok_or_else(|| SdkError::DebugError("No debug target".to_string()))?;

        self.breakpoints.set(target, location)
    }

    /// Set breakpoint by string spec (file:line or address)
    pub fn set_breakpoint_str(&mut self, spec: &str) -> Result<breakpoint::Breakpoint, SdkError> {
        let location = if spec.contains(':') {
            let parts: Vec<&str> = spec.split(':').collect();
            if parts.len() >= 2 {
                if let Ok(line) = parts[1].parse::<u32>() {
                    breakpoint::BreakpointLocation::Line {
                        file: parts[0].to_string(),
                        line,
                    }
                } else {
                    return Err(SdkError::InvalidArgument(format!("Invalid breakpoint spec: {}", spec)));
                }
            } else {
                return Err(SdkError::InvalidArgument(format!("Invalid breakpoint spec: {}", spec)));
            }
        } else if spec.starts_with("0x") || spec.starts_with("0X") {
            if let Ok(addr) = u64::from_str_radix(&spec[2..], 16) {
                breakpoint::BreakpointLocation::Address { address: addr }
            } else {
                return Err(SdkError::InvalidArgument(format!("Invalid address: {}", spec)));
            }
        } else {
            breakpoint::BreakpointLocation::Function { name: spec.to_string() }
        };

        self.set_breakpoint(location)
    }

    /// Continue execution
    pub fn continue_execution(&mut self) -> Result<execution::StopReason, SdkError> {
        let target = self.target.as_mut()
            .ok_or_else(|| SdkError::DebugError("No debug target".to_string()))?;

        self.execution.cont(target, &self.breakpoints)
    }

    /// Step into
    pub fn step_into(&mut self) -> Result<execution::StopReason, SdkError> {
        let target = self.target.as_mut()
            .ok_or_else(|| SdkError::DebugError("No debug target".to_string()))?;

        self.execution.step_in(target)
    }

    /// Step over
    pub fn step_over(&mut self) -> Result<execution::StopReason, SdkError> {
        let target = self.target.as_mut()
            .ok_or_else(|| SdkError::DebugError("No debug target".to_string()))?;

        self.execution.step_over(target)
    }

    /// Step out
    pub fn step_out(&mut self) -> Result<execution::StopReason, SdkError> {
        let target = self.target.as_mut()
            .ok_or_else(|| SdkError::DebugError("No debug target".to_string()))?;

        self.execution.step_out(target)
    }

    /// Pause execution
    pub fn pause(&mut self) -> Result<(), SdkError> {
        let target = self.target.as_mut()
            .ok_or_else(|| SdkError::DebugError("No debug target".to_string()))?;

        target.pause()
    }

    /// Terminate debug session
    pub fn terminate(&mut self) -> Result<(), SdkError> {
        if let Some(target) = self.target.take() {
            target.terminate()?;
        }
        Ok(())
    }

    /// Get call stack
    pub fn stack_trace(&self) -> Result<Vec<stack::StackFrame>, SdkError> {
        let target = self.target.as_ref()
            .ok_or_else(|| SdkError::DebugError("No debug target".to_string()))?;

        target.stack_trace()
    }

    /// Read variable
    pub fn read_variable(&self, name: &str) -> Result<variable::Variable, SdkError> {
        let target = self.target.as_ref()
            .ok_or_else(|| SdkError::DebugError("No debug target".to_string()))?;

        target.read_variable(name)
    }

    /// Evaluate expression
    pub fn evaluate_expression(&self, expr: &str) -> Result<String, SdkError> {
        let var = self.read_variable(expr)?;
        Ok(var.value.to_string_repr())
    }

    /// Get stack trace as strings
    pub fn get_stack_trace(&self) -> Result<Vec<String>, SdkError> {
        let frames = self.stack_trace()?;
        Ok(frames.iter().map(|f| {
            if let (Some(ref file), Some(line)) = (&f.file, f.line) {
                format!("{} @ {}:{} (0x{:016x})", f.function, file, line, f.address)
            } else {
                format!("{} @ 0x{:016x}", f.function, f.address)
            }
        }).collect())
    }

    /// Read memory
    pub fn read_memory(&self, address: u64, size: usize) -> Result<Vec<u8>, SdkError> {
        let target = self.target.as_ref()
            .ok_or_else(|| SdkError::DebugError("No debug target".to_string()))?;

        target.read_memory(address, size)
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

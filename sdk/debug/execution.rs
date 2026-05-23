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

// ! executecontrolcontrol

use crate::error::SdkError;
use super::target::DebugTarget;
use super::breakpoint::BreakpointManager;

/// executecontroller
pub struct ExecutionController {
    /// currentstate
    state: ExecutionState,
}

impl ExecutionController {
    pub fn new() -> Self {
        Self {
            state: ExecutionState::Stopped(StopReason::NotStarted),
        }
    }

    /// Continue execution
    pub fn cont(&mut self, target: &mut DebugTarget, breakpoints: &BreakpointManager) -> Result<StopReason, SdkError> {
        log_debug!("Continuing execution");
        
        self.state = ExecutionState::Running;
        
        // Continue target execution
        target.cont()?;
        
        // Wait for stop event
        let reason = self.wait_for_stop(target, breakpoints)?;
        
        self.state = ExecutionState::Stopped(reason.clone());
        
        log_info!("Stopped at: {:?}", reason);
        
        Ok(reason)
    }

    /// Step into
    pub fn step_in(&mut self, target: &mut DebugTarget) -> Result<StopReason, SdkError> {
        log_debug!("Stepping into");
        
        self.state = ExecutionState::Running;
        
        // Single step into function calls
        target.step_in()?;
        
        // Wait for stop
        let reason = self.wait_for_stop(target, &BreakpointManager::new())?;
        
        self.state = ExecutionState::Stopped(reason.clone());
        
        log_info!("Stepped into at: {:?}", reason);
        
        Ok(reason)
    }

    /// Step over
    pub fn step_over(&mut self, target: &mut DebugTarget) -> Result<StopReason, SdkError> {
        log_debug!("Stepping over");
        
        self.state = ExecutionState::Running;
        
        // Single step over function calls
        target.step_over()?;
        
        // Wait for stop
        let reason = self.wait_for_stop(target, &BreakpointManager::new())?;
        
        self.state = ExecutionState::Stopped(reason.clone());
        
        log_info!("Stepped over at: {:?}", reason);
        
        Ok(reason)
    }

    /// Step out
    pub fn step_out(&mut self, target: &mut DebugTarget) -> Result<StopReason, SdkError> {
        log_debug!("Stepping out");
        
        self.state = ExecutionState::Running;
        
        // Step out of current function
        target.step_out()?;
        
        // Wait for stop
        let reason = self.wait_for_stop(target, &BreakpointManager::new())?;
        
        self.state = ExecutionState::Stopped(reason.clone());
        
        log_info!("Stepped out at: {:?}", reason);

        Ok(reason)
    }

    /// Wait for target to stop
    fn wait_for_stop(&self, target: &DebugTarget, breakpoints: &BreakpointManager) -> Result<StopReason, SdkError> {
        // Wait for target to stop
        loop {
            match target.wait_for_event()? {
                DebugEvent::BreakpointHit => {
                    return Ok(StopReason::Breakpoint);
                }
                DebugEvent::StepComplete => {
                    return Ok(StopReason::Step);
                }
                DebugEvent::Exception(msg) => {
                    return Ok(StopReason::Exception(msg));
                }
                DebugEvent::Signal(sig) => {
                    return Ok(StopReason::Signal(sig));
                }
                DebugEvent::ProcessExit(code) => {
                    return Ok(StopReason::ProcessExit { code });
                }
                DebugEvent::ThreadExit => {
                    return Ok(StopReason::ThreadExit);
                }
                DebugEvent::None => {
                    // Continue waiting
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
            }
        }
    }

    /// getcurrentstate
    pub fn state(&self) -> &ExecutionState {
        &self.state
    }
}

impl Default for ExecutionController {
    fn default() -> Self {
        Self::new()
    }
}

/// executestate
#[derive(Debug, Clone)]
pub enum ExecutionState {
    /// runinfix
    Running,
    /// alreadystop
    Stopped(StopReason),
    /// alreadyTerminate
    Terminated,
}

/// Stop reason
#[derive(Debug, Clone)]
pub enum StopReason {
    /// Not started
    NotStarted,
    /// Breakpoint hit
    Breakpoint,
    /// Step completed
    Step,
    /// Exception occurred
    Exception(String),
    /// Signal received
    Signal(String),
    /// User paused
    Pause,
    /// Thread exited
    ThreadExit,
    /// Process exited
    ProcessExit { code: i32 },
}

/// Debug event
#[derive(Debug, Clone)]
pub enum DebugEvent {
    /// Breakpoint hit
    BreakpointHit,
    /// Step completed
    StepComplete,
    /// Exception occurred
    Exception(String),
    /// Signal received
    Signal(String),
    /// Process exited
    ProcessExit(i32),
    /// Thread exited
    ThreadExit,
    /// No event
    None,
}
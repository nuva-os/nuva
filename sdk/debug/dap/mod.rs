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

//! Debug Adapter Protocol (DAP) server implementation

pub mod server;
pub mod protocol;

use std::collections::HashMap;
use crate::error::SdkError;
use super::breakpoint::{BreakpointManager, BreakpointLocation, Breakpoint, WatchType};
use super::execution::{ExecutionController, ExecutionState, StopReason};
use super::target::DebugTarget;
use super::stack::{StackFrame, StackVariable};
use super::variable::{Variable, VariableType, VariableValue};
use super::memory::MemoryViewer;

/// DAP server
pub struct DapServer {
    /// Whether initialized
    initialized: bool,
    /// Client capabilities
    client_capabilities: Option<ClientCapabilities>,
    /// Debug target
    target: Option<DebugTarget>,
    /// Breakpoint manager
    breakpoints: BreakpointManager,
    /// Execution controller
    execution: ExecutionController,
    /// Variable references for DAP protocol
    variable_refs: HashMap<u32, VariableRefEntry>,
    /// Next variable reference ID
    next_var_ref: u32,
    /// Thread list
    threads: Vec<ThreadInfo>,
}

/// Variable reference entry
#[derive(Debug, Clone)]
struct VariableRefEntry {
    name: String,
    frame_id: Option<u32>,
    scope: Option<String>,
}

/// Thread information
#[derive(Debug, Clone)]
struct ThreadInfo {
    id: u64,
    name: String,
    stopped: bool,
}

impl DapServer {
    pub fn new() -> Self {
        Self {
            initialized: false,
            client_capabilities: None,
            target: None,
            breakpoints: BreakpointManager::new(),
            execution: ExecutionController::new(),
            variable_refs: HashMap::new(),
            next_var_ref: 1,
            threads: vec![ThreadInfo {
                id: 1,
                name: "main".to_string(),
                stopped: true,
            }],
        }
    }

    /// Process DAP request
    pub fn handle_request(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        match request.command.as_str() {
            "initialize" => self.handle_initialize(request),
            "launch" => self.handle_launch(request),
            "attach" => self.handle_attach(request),
            "setBreakpoints" => self.handle_set_breakpoints(request),
            "setFunctionBreakpoints" => self.handle_set_function_breakpoints(request),
            "setDataBreakpoints" => self.handle_set_data_breakpoints(request),
            "configurationDone" => self.handle_configuration_done(request),
            "continue" => self.handle_continue(request),
            "next" => self.handle_next(request),
            "stepIn" => self.handle_step_in(request),
            "stepOut" => self.handle_step_out(request),
            "pause" => self.handle_pause(request),
            "stackTrace" => self.handle_stack_trace(request),
            "scopes" => self.handle_scopes(request),
            "variables" => self.handle_variables(request),
            "setVariable" => self.handle_set_variable(request),
            "evaluate" => self.handle_evaluate(request),
            "threads" => self.handle_threads(request),
            "disconnect" => self.handle_disconnect(request),
            "readMemory" => self.handle_read_memory(request),
            "writeMemory" => self.handle_write_memory(request),
            "disassemble" => self.handle_disassemble(request),
            _ => Err(SdkError::Unsupported(format!("Unknown DAP command: {}", request.command))),
        }
    }

    fn handle_initialize(&mut self, _request: protocol::Request) -> Result<protocol::Response, SdkError> {
        self.initialized = true;

        let body = serde_json::to_string(&InitializeResponse {
            supports_configuration_done_request: true,
            supports_set_variable: true,
            supports_conditional_breakpoints: true,
            supports_hit_conditional_breakpoints: true,
            supports_evaluate_for_hovers: true,
            supports_step_back: false,
            supports_restart_frame: false,
            supports_goto_targets: false,
            supports_step_in_targets: false,
            supports_completions: false,
            supports_modules: false,
            supports_exception_options: false,
            supports_value_formatting_options: false,
            supports_exception_info: false,
            support_suspend_debuggee: true,
            supports_terminate_debuggee: true,
            supports_delayed_stack_trace_loading: false,
            supports_loaded_sources: false,
            supports_log_points: true,
            supports_terminate_threads: true,
            supports_set_expression: true,
            supports_disassemble: true,
            supports_data_breakpoints: true,
            supports_function_breakpoints: true,
            supports_read_memory_request: true,
            supports_write_memory_request: true,
        }).unwrap_or_default();

        Ok(protocol::Response::success(0, body))
    }

    fn handle_launch(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let args: LaunchArguments = serde_json::from_str(&request.arguments)
            .map_err(|e| SdkError::ParseError(format!("Failed to parse launch arguments: {}", e)))?;

        let target = DebugTarget::launch(&args.program, &args.args)?;
        self.target = Some(target);

        for thread in &mut self.threads {
            thread.stopped = args.stop_on_entry.unwrap_or(true);
        }

        let body = serde_json::json!({
            "allThreadsStopped": args.stop_on_entry.unwrap_or(true)
        }).to_string();

        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_attach(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let args: AttachArguments = serde_json::from_str(&request.arguments)
            .map_err(|e| SdkError::ParseError(format!("Failed to parse attach arguments: {}", e)))?;

        let pid = args.process_id.unwrap_or(0);
        if pid == 0 {
            return Err(SdkError::InvalidArgument("Invalid PID for attach".to_string()));
        }

        let target = DebugTarget::attach(pid)?;
        self.target = Some(target);

        let body = serde_json::json!({
            "allThreadsStopped": true
        }).to_string();

        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_set_breakpoints(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let args: SetBreakpointsArguments = serde_json::from_str(&request.arguments)
            .map_err(|e| SdkError::ParseError(format!("Failed to parse breakpoint arguments: {}", e)))?;

        let mut breakpoints = vec![];
        for bp in &args.breakpoints {
            let location = BreakpointLocation::Line {
                file: args.source.path.clone(),
                line: bp.line,
            };

            let result = if let Some(ref mut target) = self.target {
                self.breakpoints.set(target, location).ok()
            } else {
                None
            };

            let verified = result.is_some();
            breakpoints.push(BreakpointResult {
                id: result.map(|r| r.id).unwrap_or(0),
                verified,
                line: bp.line,
                column: bp.column,
                source: args.source.clone(),
            });
        }

        let body = serde_json::to_string(&SetBreakpointsResponse {
            breakpoints,
        }).unwrap_or_default();

        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_set_function_breakpoints(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let body = r#"{"breakpoints":[]}"#.to_string();
        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_set_data_breakpoints(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let body = r#"{"breakpoints":[]}"#.to_string();
        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_configuration_done(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        Ok(protocol::Response::success(request.seq, "{}".to_string()))
    }

    fn handle_continue(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        if let Some(ref mut target) = self.target {
            let _reason = self.execution.cont(target, &self.breakpoints)?;
            for thread in &mut self.threads {
                thread.stopped = false;
            }
        }

        let body = r#"{"allThreadsContinued":true}"#.to_string();
        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_next(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        if let Some(ref mut target) = self.target {
            let _reason = self.execution.step_over(target)?;
        }

        Ok(protocol::Response::success(request.seq, "{}".to_string()))
    }

    fn handle_step_in(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        if let Some(ref mut target) = self.target {
            let _reason = self.execution.step_in(target)?;
        }

        Ok(protocol::Response::success(request.seq, "{}".to_string()))
    }

    fn handle_step_out(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        if let Some(ref mut target) = self.target {
            let _reason = self.execution.step_out(target)?;
        }

        Ok(protocol::Response::success(request.seq, "{}".to_string()))
    }

    fn handle_pause(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        if let Some(ref mut target) = self.target {
            target.pause()?;
            for thread in &mut self.threads {
                thread.stopped = true;
            }
        }

        Ok(protocol::Response::success(request.seq, "{}".to_string()))
    }

    fn handle_stack_trace(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let frames = if let Some(ref target) = self.target {
            target.stack_trace().unwrap_or_default()
        } else {
            vec![]
        };

        let stack_frames: Vec<StackFrameResult> = frames.iter().map(|f| StackFrameResult {
            id: f.id,
            name: f.function.clone(),
            source: f.file.as_ref().map(|p| SourceResult {
                path: p.clone(),
                name: None,
            }),
            line: f.line.unwrap_or(0),
            column: f.column.unwrap_or(0),
        }).collect();

        let body = serde_json::json!({
            "stackFrames": stack_frames,
            "totalFrames": stack_frames.len()
        }).to_string();

        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_scopes(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let args: ScopesArguments = serde_json::from_str(&request.arguments)
            .map_err(|e| SdkError::ParseError(format!("Failed to parse scopes arguments: {}", e)))?;

        let local_ref = self.next_var_ref;
        self.next_var_ref += 1;
        self.variable_refs.insert(local_ref, VariableRefEntry {
            name: "Locals".to_string(),
            frame_id: Some(args.frame_id),
            scope: Some("local".to_string()),
        });

        let global_ref = self.next_var_ref;
        self.next_var_ref += 1;
        self.variable_refs.insert(global_ref, VariableRefEntry {
            name: "Globals".to_string(),
            frame_id: Some(args.frame_id),
            scope: Some("global".to_string()),
        });

        let scopes = vec![
            ScopeResult { name: "Locals".to_string(), variables_reference: local_ref, expensive: false },
            ScopeResult { name: "Globals".to_string(), variables_reference: global_ref, expensive: true },
        ];

        let body = serde_json::json!({
            "scopes": scopes
        }).to_string();

        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_variables(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let args: VariablesArguments = serde_json::from_str(&request.arguments)
            .map_err(|e| SdkError::ParseError(format!("Failed to parse variables arguments: {}", e)))?;

        let mut variables = Vec::new();
        if let Some(ref target) = self.target {
            if let Some(ref entry) = self.variable_refs.get(&args.variables_reference) {
                if let Some(frame_id) = entry.frame_id {
                    let frames = target.stack_trace().unwrap_or_default();
                    if let Some(frame) = frames.iter().find(|f| f.id == frame_id) {
                        if let Ok(var) = target.read_variable(&frame.function) {
                            variables.push(VariableResult {
                                name: var.name.clone(),
                                value: var.value.to_string_repr(),
                                r#type: Some(var.var_type.to_string()),
                                variables_reference: 0,
                            });
                        }
                    }
                }
            }
        }

        let body = serde_json::json!({
            "variables": variables
        }).to_string();

        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_set_variable(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let body = r#"{"value":""}"#.to_string();
        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_evaluate(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let args: EvaluateArguments = serde_json::from_str(&request.arguments)
            .map_err(|e| SdkError::ParseError(format!("Failed to parse evaluate arguments: {}", e)))?;

        let result = if let Some(ref target) = self.target {
            target.read_variable(&args.expression)
                .map(|v| v.value.to_string_repr())
                .unwrap_or_else(|_| "<error>".to_string())
        } else {
            "<no target>".to_string()
        };

        let body = serde_json::json!({
            "result": result,
            "variablesReference": 0
        }).to_string();

        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_threads(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let threads: Vec<ThreadResult> = self.threads.iter().map(|t| ThreadResult {
            id: t.id,
            name: t.name.clone(),
        }).collect();

        let body = serde_json::json!({
            "threads": threads
        }).to_string();

        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_disconnect(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        self.initialized = false;
        if let Some(target) = self.target.take() {
            target.terminate()?;
        }

        Ok(protocol::Response::success(request.seq, "{}".to_string()))
    }

    fn handle_read_memory(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let args: ReadMemoryArguments = serde_json::from_str(&request.arguments)
            .map_err(|e| SdkError::ParseError(format!("Failed to parse readMemory arguments: {}", e)))?;

        let data = if let Some(ref target) = self.target {
            target.read_memory(args.memory_reference, args.count).unwrap_or_default()
        } else {
            vec![]
        };

        let viewer = MemoryViewer::new(data.clone(), args.memory_reference);
        let body = serde_json::json!({
            "address": format!("0x{:016x}", args.memory_reference),
            "data": viewer.to_hex(),
            "unreadableBytes": 0
        }).to_string();

        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_write_memory(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let body = r#"{"bytesWritten":0}"#.to_string();
        Ok(protocol::Response::success(request.seq, body))
    }

    fn handle_disassemble(&mut self, request: protocol::Request) -> Result<protocol::Response, SdkError> {
        let args: DisassembleArguments = serde_json::from_str(&request.arguments)
            .map_err(|e| SdkError::ParseError(format!("Failed to parse disassemble arguments: {}", e)))?;

        let mut instructions = Vec::new();
        if let Some(ref target) = self.target {
            if let Ok(data) = target.read_memory(args.memory_reference, args.instruction_count as usize * 16) {
                let mut offset = 0u64;
                for chunk in data.chunks(16).take(args.instruction_count as usize) {
                    let instr_addr = args.memory_reference + offset;
                    let mut hex_bytes = String::new();
                    for b in chunk {
                        hex_bytes.push_str(&format!("{:02x} ", b));
                    }
                    instructions.push(DisassembledInstruction {
                        address: format!("0x{:016x}", instr_addr),
                        instruction: hex_bytes.trim_end().to_string(),
                    });
                    offset += chunk.len() as u64;
                }
            }
        }

        let body = serde_json::json!({
            "instructions": instructions
        }).to_string();

        Ok(protocol::Response::success(request.seq, body))
    }
}

impl Default for DapServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Client capabilities
#[derive(Debug, Default, serde::Deserialize)]
struct ClientCapabilities {
    supports_variable_type: bool,
    supports_variable_paging: bool,
    supports_run_in_terminal_request: bool,
}

#[derive(Debug, serde::Serialize)]
struct InitializeResponse {
    supports_configuration_done_request: bool,
    supports_set_variable: bool,
    supports_conditional_breakpoints: bool,
    supports_hit_conditional_breakpoints: bool,
    supports_evaluate_for_hovers: bool,
    supports_step_back: bool,
    supports_restart_frame: bool,
    supports_goto_targets: bool,
    supports_step_in_targets: bool,
    supports_completions: bool,
    supports_modules: bool,
    supports_exception_options: bool,
    supports_value_formatting_options: bool,
    supports_exception_info: bool,
    support_suspend_debuggee: bool,
    supports_terminate_debuggee: bool,
    supports_delayed_stack_trace_loading: bool,
    supports_loaded_sources: bool,
    supports_log_points: bool,
    supports_terminate_threads: bool,
    supports_set_expression: bool,
    supports_disassemble: bool,
    supports_data_breakpoints: bool,
    supports_function_breakpoints: bool,
    supports_read_memory_request: bool,
    supports_write_memory_request: bool,
}

#[derive(Debug, serde::Deserialize)]
struct LaunchArguments {
    program: String,
    #[serde(default)]
    args: Vec<String>,
    cwd: Option<String>,
    env: Option<std::collections::HashMap<String, String>>,
    stop_on_entry: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
struct AttachArguments {
    process_id: Option<u32>,
    host: Option<String>,
    port: Option<u16>,
}

#[derive(Debug, serde::Deserialize)]
struct SetBreakpointsArguments {
    source: Source,
    breakpoints: Vec<BreakpointArg>,
}

#[derive(Debug, serde::Deserialize)]
struct BreakpointArg {
    line: u32,
    column: Option<u32>,
    condition: Option<String>,
    hit_condition: Option<String>,
    log_message: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ScopesArguments {
    frame_id: u32,
}

#[derive(Debug, serde::Deserialize)]
struct EvaluateArguments {
    expression: String,
    frame_id: Option<u32>,
    context: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ReadMemoryArguments {
    memory_reference: u64,
    offset: Option<u64>,
    count: usize,
}

#[derive(Debug, serde::Deserialize)]
struct VariablesArguments {
    variables_reference: u64,
}

#[derive(Debug, serde::Deserialize)]
struct DisassembleArguments {
    memory_reference: u64,
    instruction_offset: Option<u64>,
    instruction_count: u32,
    resolve_symbols: Option<bool>,
}

#[derive(Debug, serde::Serialize)]
struct DisassembledInstruction {
    address: String,
    instruction: String,
}

/// Source
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
struct Source {
    path: String,
    name: Option<String>,
}

/// Breakpoint result
#[derive(Debug, serde::Serialize)]
struct BreakpointResult {
    id: u32,
    verified: bool,
    line: u32,
    column: Option<u32>,
    source: Source,
}

/// Set breakpoints response
#[derive(Debug, serde::Serialize)]
struct SetBreakpointsResponse {
    breakpoints: Vec<BreakpointResult>,
}

/// Stack frame result
#[derive(Debug, serde::Serialize)]
struct StackFrameResult {
    id: u32,
    name: String,
    source: Option<SourceResult>,
    line: u32,
    column: u32,
}

/// Source result
#[derive(Debug, serde::Serialize)]
struct SourceResult {
    path: String,
    name: Option<String>,
}

/// Scope result
#[derive(Debug, serde::Serialize)]
struct ScopeResult {
    name: String,
    variables_reference: u32,
    expensive: bool,
}

/// Thread result
#[derive(Debug, serde::Serialize)]
struct ThreadResult {
    id: u64,
    name: String,
}

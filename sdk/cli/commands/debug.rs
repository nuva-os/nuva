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

//! Debug command

use crate::NuvaSdk;
use crate::error::SdkError;
use crate::cli::args::DebugCommand;
use crate::cli::output;

/// Execute debug command
pub fn execute(sdk: &mut NuvaSdk, cmd: DebugCommand) -> Result<(), SdkError> {
    // 1. Determine debug mode
    if let Some(pid) = cmd.attach {
        return execute_attach(sdk, pid, &cmd);
    } else if let Some(program) = &cmd.program {
        return execute_launch(sdk, program, &cmd);
    } else {
        // Debug current project
        return execute_project_debug(sdk, &cmd);
    }
}

/// Attach to running process
fn execute_attach(sdk: &mut NuvaSdk, pid: u32, cmd: &DebugCommand) -> Result<(), SdkError> {
    output::info(&format!("Attaching to process {}...", pid));
    
    // 1. Create debug target for attaching
    let target = sdk.create_debug_target_attach(pid)?;
    output::debug(&format!("Debug target created for PID {}", pid));
    
    // 2. Initialize debugger
    let mut debugger = sdk.create_debugger(target)?;
    output::info("Debugger initialized");
    
    // 3. Set breakpoints if specified
    if !cmd.breakpoints.is_empty() {
        output::info(&format!("Setting {} breakpoints...", cmd.breakpoints.len()));
        for bp in &cmd.breakpoints {
            debugger.set_breakpoint(bp)?;
        }
    }
    
    // 4. Start DAP server if requested
    if cmd.dap {
        output::info("Starting DAP server...");
        let dap_server = sdk.start_dap_server(debugger)?;
        output::info(&format!("DAP server listening on {}", dap_server.address()));
        
        // Keep DAP server running
        output::info("DAP server running. Press Ctrl+C to stop.");
        dap_server.run()?;
    } else {
        // Run interactive debug session
        output::info("Starting interactive debug session...");
        run_interactive_session(debugger)?;
    }
    
    Ok(())
}

/// Launch program for debugging
fn execute_launch(sdk: &mut NuvaSdk, program: &str, cmd: &DebugCommand) -> Result<(), SdkError> {
    output::info(&format!("Starting debug session for {}...", program));
    
    // 1. Check if program exists
    if !std::path::Path::new(program).exists() {
        return Err(SdkError::FileNotFound(program.to_string()));
    }
    
    // 2. Create debug target for launching
    let target = sdk.create_debug_target_launch(program, &cmd.args)?;
    output::debug(&format!("Debug target created for {}", program));
    
    // 3. Initialize debugger
    let mut debugger = sdk.create_debugger(target)?;
    output::info("Debugger initialized");
    
    // 4. Set breakpoints if specified
    if !cmd.breakpoints.is_empty() {
        output::info(&format!("Setting {} breakpoints...", cmd.breakpoints.len()));
        for bp in &cmd.breakpoints {
            debugger.set_breakpoint(bp)?;
        }
    }
    
    // 5. Start DAP server if requested
    if cmd.dap {
        output::info("Starting DAP server...");
        let dap_server = sdk.start_dap_server(debugger)?;
        output::info(&format!("DAP server listening on {}", dap_server.address()));
        
        // Keep DAP server running
        output::info("DAP server running. Press Ctrl+C to stop.");
        dap_server.run()?;
    } else {
        // Run interactive debug session
        output::info("Starting interactive debug session...");
        run_interactive_session(debugger)?;
    }
    
    Ok(())
}

/// Debug current project
fn execute_project_debug(sdk: &mut NuvaSdk, cmd: &DebugCommand) -> Result<(), SdkError> {
    output::info("Starting debug session for current project...");
    
    // 1. Build project in debug mode
    output::info("Building project in debug mode...");
    let build_cmd = crate::cli::args::BuildCommand {
        release: false,
        target: cmd.target.clone(),
        features: vec![],
        jobs: None,
        opt_level: Some(0),
        debug_info: true,
    };
    crate::cli::commands::build::execute(sdk, build_cmd)?;
    
    // 2. Get project binary path
    let manifest = sdk.load_manifest()?;
    let binary_path = std::path::PathBuf::from("target/debug")
        .join(format!("{}{}", manifest.name, std::env::consts::EXE_SUFFIX));
    
    if !binary_path.exists() {
        return Err(SdkError::FileNotFound(binary_path.display().to_string()));
    }
    
    // 3. Create debug target
    let target = sdk.create_debug_target_launch(
        binary_path.to_str().unwrap(),
        &cmd.args
    )?;
    output::debug(&format!("Debug target created for {}", binary_path.display()));
    
    // 4. Initialize debugger
    let mut debugger = sdk.create_debugger(target)?;
    output::info("Debugger initialized");
    
    // 5. Set breakpoints if specified
    if !cmd.breakpoints.is_empty() {
        output::info(&format!("Setting {} breakpoints...", cmd.breakpoints.len()));
        for bp in &cmd.breakpoints {
            debugger.set_breakpoint(bp)?;
        }
    }
    
    // 6. Start DAP server if requested
    if cmd.dap {
        output::info("Starting DAP server...");
        let dap_server = sdk.start_dap_server(debugger)?;
        output::info(&format!("DAP server listening on {}", dap_server.address()));
        
        // Keep DAP server running
        output::info("DAP server running. Press Ctrl+C to stop.");
        dap_server.run()?;
    } else {
        // Run interactive debug session
        output::info("Starting interactive debug session...");
        run_interactive_session(debugger)?;
    }
    
    Ok(())
}

/// Run interactive debug session
fn run_interactive_session(debugger: &mut crate::debug::Debugger) -> Result<(), SdkError> {
    use std::io::{self, Write, BufRead};
    
    output::info("Interactive debug session started");
    output::info("Available commands: continue, next, step, step-out, break, print, stack, quit");
    
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    loop {
        print!("(debug) ");
        stdout.flush()?;
        
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];
        
        match command {
            "continue" | "c" => {
                output::info("Continuing execution...");
                debugger.continue_execution()?;
            }
            "next" | "n" => {
                output::info("Stepping over...");
                debugger.step_over()?;
            }
            "step" | "s" => {
                output::info("Stepping into...");
                debugger.step_into()?;
            }
            "step-out" | "o" => {
                output::info("Stepping out...");
                debugger.step_out()?;
            }
            "break" | "b" => {
                if parts.len() < 2 {
                    output::error("Usage: break <file:line> or break <address>");
                    continue;
                }
                debugger.set_breakpoint(parts[1])?;
                output::success(&format!("Breakpoint set at {}", parts[1]));
            }
            "print" | "p" => {
                if parts.len() < 2 {
                    output::error("Usage: print <expression>");
                    continue;
                }
                let value = debugger.evaluate_expression(parts[1])?;
                println!("{} = {}", parts[1], value);
            }
            "stack" | "bt" => {
                let stack_trace = debugger.get_stack_trace()?;
                for (i, frame) in stack_trace.iter().enumerate() {
                    println!("{}: {}", i, frame);
                }
            }
            "quit" | "q" | "exit" => {
                output::info("Exiting debug session...");
                break;
            }
            "help" | "h" => {
                println!("Available commands:");
                println!("  continue, c     - Continue execution");
                println!("  next, n         - Step over");
                println!("  step, s         - Step into");
                println!("  step-out, o     - Step out");
                println!("  break, b        - Set breakpoint");
                println!("  print, p        - Evaluate expression");
                println!("  stack, bt       - Show stack trace");
                println!("  quit, q         - Exit debugger");
                println!("  help, h         - Show this help");
            }
            _ => {
                output::error(&format!("Unknown command: {}", command));
                output::info("Type 'help' for available commands");
            }
        }
    }
    
    Ok(())
}

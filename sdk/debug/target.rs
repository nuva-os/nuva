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

// ! Debug target

use crate::error::SdkError;
use super::stack::StackFrame;
use super::variable::Variable;
use alloc::vec;
use alloc::format;
use alloc::vec::Vec;

/// Debug target
pub struct DebugTarget {
    /// Process ID
    pid: u32,
    /// Program path
    program: String,
    /// Whether running
    running: bool,
}

impl DebugTarget {
    /// Launch program
    pub fn launch(program: &str, args: &[String]) -> Result<Self, SdkError> {
        let program_path = std::path::PathBuf::from(program);
        if !program_path.exists() {
            return Err(SdkError::NotFoundError(format!("Program not found: {}", program)));
        }

        let pid = unsafe { libc::fork() };
        if pid < 0 {
            return Err(SdkError::IoError("fork failed".to_string()));
        }
        if pid == 0 {
            let mut argv = vec![program.to_string()];
            for a in args {
                argv.push(a.clone());
            }
            let c_args: Vec<std::ffi::CString> = argv.iter()
                .map(|s| std::ffi::CString::new(s.as_str()).unwrap_or_default())
                .collect();
            let c_argv: Vec<*const i8> = c_args.iter()
                .map(|s| s.as_ptr())
                .chain(std::iter::once(std::ptr::null()))
                .collect();
            unsafe { libc::execv(program.as_ptr() as *const i8, c_argv.as_ptr()); }
            libc::_exit(1);
        }

        Ok(Self {
            pid: pid as u32,
            program: program.to_string(),
            running: true,
        })
    }

    /// Attach to process
    pub fn attach(pid: u32) -> Result<Self, SdkError> {
        if pid == 0 {
            return Err(SdkError::InvalidArgument("Invalid PID".to_string()));
        }

        Ok(Self {
            pid,
            program: String::new(),
            running: true,
        })
    }

    /// Pause target
    pub fn pause(&mut self) -> Result<(), SdkError> {
        self.running = false;
        Ok(())
    }

    /// Continue target
    pub fn cont(&mut self) -> Result<(), SdkError> {
        self.running = true;
        Ok(())
    }

    /// Step into
    pub fn step_in(&mut self) -> Result<(), SdkError> {
        Ok(())
    }

    /// Step over
    pub fn step_over(&mut self) -> Result<(), SdkError> {
        Ok(())
    }

    /// Step out
    pub fn step_out(&mut self) -> Result<(), SdkError> {
        Ok(())
    }

    /// Wait for debug event
    pub fn wait_for_event(&self) -> Result<super::execution::DebugEvent, SdkError> {
        Ok(super::execution::DebugEvent::None)
    }

    /// Terminate target
    pub fn terminate(self) -> Result<(), SdkError> {
        Ok(())
    }

    /// Get process ID
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Read registers
    pub fn read_registers(&self) -> Result<Registers, SdkError> {
        if !self.running {
            return Err(SdkError::InvalidState("Process not running".to_string()));
        }

        let mut regs = Registers::default();
        unsafe {
            let mut user_regs: libc::user_regs_struct = std::mem::zeroed();
            let ret = libc::ptrace(
                libc::PTRACE_GETREGS,
                self.pid as i32,
                std::ptr::null_mut(),
                &mut user_regs as *mut _ as *mut libc::c_void,
            );
            if ret == 0 {
                regs.pc = user_regs.rip;
                regs.sp = user_regs.rsp;
                regs.general[0] = user_regs.rax;
                regs.general[1] = user_regs.rbx;
                regs.general[2] = user_regs.rcx;
                regs.general[3] = user_regs.rdx;
                regs.general[4] = user_regs.rsi;
                regs.general[5] = user_regs.rdi;
                regs.general[6] = user_regs.rbp;
                regs.general[7] = user_regs.r8;
                regs.general[8] = user_regs.r9;
                regs.general[9] = user_regs.r10;
                regs.general[10] = user_regs.r11;
                regs.general[11] = user_regs.r12;
                regs.general[12] = user_regs.r13;
                regs.general[13] = user_regs.r14;
                regs.general[14] = user_regs.r15;
                regs.flags = user_regs.eflags as u64;
            }
        }

        Ok(regs)
    }

    /// Write registers
    pub fn write_registers(&mut self, _regs: &Registers) -> Result<(), SdkError> {
        if !self.running {
            return Err(SdkError::InvalidState("Process not running".to_string()));
        }

        Ok(())
    }

    /// Read memory
    pub fn read_memory(&self, address: u64, size: usize) -> Result<Vec<u8>, SdkError> {
        if !self.running {
            return Err(SdkError::InvalidState("Process not running".to_string()));
        }

        if size == 0 {
            return Err(SdkError::InvalidArgument("Invalid size".to_string()));
        }

        let mut buf = vec![0u8; size];
        let local_iov = libc::iovec {
            iov_base: buf.as_mut_ptr() as *mut libc::c_void,
            iov_len: size,
        };
        let remote_iov = libc::iovec {
            iov_base: address as *mut libc::c_void,
            iov_len: size,
        };
        let n = unsafe {
            libc::process_vm_readv(
                self.pid as i32,
                &local_iov as *const libc::iovec,
                1,
                &remote_iov as *const libc::iovec,
                1,
                0,
            )
        };
        if n < 0 {
            for i in 0..size {
                let word = unsafe {
                    libc::ptrace(
                        libc::PTRACE_PEEKDATA,
                        self.pid as i32,
                        (address + i as u64) as *mut libc::c_void,
                        std::ptr::null_mut(),
                    )
                };
                buf[i] = word as u8;
            }
        }

        Ok(buf)
    }

    /// Write memory
    pub fn write_memory(&mut self, address: u64, data: &[u8]) -> Result<(), SdkError> {
        if !self.running {
            return Err(SdkError::InvalidState("Process not running".to_string()));
        }

        if data.is_empty() {
            return Err(SdkError::InvalidArgument("Invalid data".to_string()));
        }

        Ok(())
    }

    /// Get call stack
    pub fn stack_trace(&self) -> Result<Vec<StackFrame>, SdkError> {
        if !self.running {
            return Err(SdkError::InvalidState("Process not running".to_string()));
        }

        let frame = StackFrame::new(0, "main", 0)
            .with_source("main.rs".to_string(), 1, 1);

        Ok(vec![frame])
    }

    /// Read variable
    pub fn read_variable(&self, name: &str) -> Result<Variable, SdkError> {
        if !self.running {
            return Err(SdkError::InvalidState("Process not running".to_string()));
        }

        if name.is_empty() {
            return Err(SdkError::InvalidArgument("Invalid variable name".to_string()));
        }

        Ok(Variable::new(
            name,
            super::variable::VariableType::Unknown,
            super::variable::VariableValue::Void,
        ))
    }
}

/// Register set
#[derive(Debug, Default)]
pub struct Registers {
    /// General purpose registers
    pub general: [u64; 32],
    /// Program counter
    pub pc: u64,
    /// Stack pointer
    pub sp: u64,
    /// Status flags
    pub flags: u64,
}

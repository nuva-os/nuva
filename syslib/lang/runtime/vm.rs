/*
 * Nuva OS - System Library - Lang Runtime VM
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


use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Virtual machine state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    /// Stopped
    Stopped = 0,
    /// Running
    Running = 1,
    /// Paused
    Paused = 2,
    /// Error
    Error = 3,
}

/// Virtual machine value
#[derive(Debug, Clone, Copy)]
pub struct VmValue {
    /// Value type
    pub value_type: VmValueType,
    /// Value data
    pub data: u64,
}

/// Virtual machine value type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmValueType {
    /// Integer
    Integer,
    /// Float
    Float,
    /// Boolean
    Bool,
    /// Pointer
    Pointer,
    /// None
    None,
}

/// Virtual machine
pub struct VirtualMachine {
    /// State
    state: AtomicU32,
    /// Registers
    registers: [VmValue; 256],
    /// Stack pointer
    stack_ptr: AtomicU64,
    /// Stack base
    stack_base: AtomicU64,
    /// Program counter
    pc: AtomicU64,
    /// Executed instruction count
    instruction_count: AtomicU64,
}

impl VirtualMachine {
    /// Create a new virtual machine
    pub const fn new() -> Self {
        VirtualMachine {
            state: AtomicU32::new(VmState::Stopped as u32),
            registers: [VmValue { value_type: VmValueType::None, data: 0 }; 256],
            stack_ptr: AtomicU64::new(0),
            stack_base: AtomicU64::new(0),
            pc: AtomicU64::new(0),
            instruction_count: AtomicU64::new(0),
        }
    }

    /// Initialize the virtual machine
    pub fn init(&mut self, stack_base: u64, stack_size: usize) {
        self.stack_base.store(stack_base, Ordering::Release);
        self.stack_ptr.store(stack_base + stack_size as u64, Ordering::Release);

        log_info!("Virtual machine initialized");
        log_info!("  Stack base: {:#x}", stack_base);
        log_info!("  Stack size: {} KB", stack_size / 1024);
    }

    /// Start the virtual machine
    pub fn start(&mut self, entry_point: u64) -> i32 {
        if self.state.load(Ordering::Acquire) != VmState::Stopped as u32 {
            return -1;
        }

        self.pc.store(entry_point, Ordering::Release);
        self.state.store(VmState::Running as u32, Ordering::Release);

        log_info!("Virtual machine started at {:#x}", entry_point);
        0
    }

    /// Stop the virtual machine
    pub fn stop(&mut self) -> i32 {
        self.state.store(VmState::Stopped as u32, Ordering::Release);

        log_info!("Virtual machine stopped");
        0
    }

    /// Pause the virtual machine
    pub fn pause(&mut self) -> i32 {
        if self.state.load(Ordering::Acquire) != VmState::Running as u32 {
            return -1;
        }

        self.state.store(VmState::Paused as u32, Ordering::Release);
        0
    }

    /// Resume the virtual machine
    pub fn resume(&mut self) -> i32 {
        if self.state.load(Ordering::Acquire) != VmState::Paused as u32 {
            return -1;
        }

        self.state.store(VmState::Running as u32, Ordering::Release);
        0
    }

    /// Execute a single instruction read from the current PC
    pub fn step(&mut self) -> i32 {
        if self.state.load(Ordering::Acquire) != VmState::Running as u32 {
            return -1;
        }

        let pc = self.pc.load(Ordering::Acquire);
        let sp = self.stack_ptr.load(Ordering::Acquire);
        let base = self.stack_base.load(Ordering::Acquire);

        // Read 32-bit instruction opcode from the program counter address
        if pc == 0 {
            self.state.store(VmState::Error as u32, Ordering::Release);
            return -2;
        }

        // SAFETY: PC points to valid executable memory set by the loader
        let opcode = unsafe { *(pc as *const u32) };

        // Dispatch based on opcode (simplified instruction set)
        match opcode {
            // NOP (0x00): advance PC and continue
            0x00 => {
                self.pc.store(pc + 4, Ordering::Release);
            }
            // HALT (0x01): stop the VM
            0x01 => {
                self.state.store(VmState::Stopped as u32, Ordering::Release);
                return 1;
            }
            // PUSH_IMM (0x02): push immediate value (next 8 bytes)
            0x02 => {
                if pc + 12 > base + 1024 * 1024 {
                    self.state.store(VmState::Error as u32, Ordering::Release);
                    return -3;
                }
                // SAFETY: immediate value follows the opcode at PC+4
                let imm = unsafe { *((pc + 4) as *const u64) };
                let val = VmValue { value_type: VmValueType::Integer, data: imm };
                if self.push(val) != 0 {
                    self.state.store(VmState::Error as u32, Ordering::Release);
                    return -3;
                }
                self.pc.store(pc + 12, Ordering::Release);
            }
            // POP (0x03): discard top of stack
            0x03 => {
                if self.pop().is_none() && sp >= base + 1024 * 1024 {
                    self.state.store(VmState::Error as u32, Ordering::Release);
                    return -3;
                }
                self.pc.store(pc + 4, Ordering::Release);
            }
            // ADD (0x04): pop two values, push their sum
            0x04 => {
                match (self.pop(), self.pop()) {
                    (Some(b), Some(a)) => {
                        let result = VmValue {
                            value_type: VmValueType::Integer,
                            data: a.data.wrapping_add(b.data),
                        };
                        if self.push(result) != 0 {
                            self.state.store(VmState::Error as u32, Ordering::Release);
                            return -3;
                        }
                    }
                    _ => {
                        self.state.store(VmState::Error as u32, Ordering::Release);
                        return -3;
                    }
                }
                self.pc.store(pc + 4, Ordering::Release);
            }
            // SUB (0x05): pop two values, push (a - b)
            0x05 => {
                match (self.pop(), self.pop()) {
                    (Some(b), Some(a)) => {
                        let result = VmValue {
                            value_type: VmValueType::Integer,
                            data: a.data.wrapping_sub(b.data),
                        };
                        if self.push(result) != 0 {
                            self.state.store(VmState::Error as u32, Ordering::Release);
                            return -3;
                        }
                    }
                    _ => {
                        self.state.store(VmState::Error as u32, Ordering::Release);
                        return -3;
                    }
                }
                self.pc.store(pc + 4, Ordering::Release);
            }
            // JUMP (0x06): absolute jump to address in next 8 bytes
            0x06 => {
                // SAFETY: target address follows the opcode at PC+4
                let target = unsafe { *((pc + 4) as *const u64) };
                self.pc.store(target, Ordering::Release);
            }
            // LOAD_REG (0x07): load register index from next byte, push value
            0x07 => {
                // SAFETY: register index follows at PC+4
                let reg_idx = unsafe { *((pc + 4) as *const u8) };
                if reg_idx as usize >= 256 {
                    self.state.store(VmState::Error as u32, Ordering::Release);
                    return -3;
                }
                let val = self.registers[reg_idx as usize];
                if self.push(val) != 0 {
                    self.state.store(VmState::Error as u32, Ordering::Release);
                    return -3;
                }
                self.pc.store(pc + 8, Ordering::Release);
            }
            // Default: treat as NOP and advance PC
            _ => {
                self.pc.store(pc + 4, Ordering::Release);
            }
        }

        self.instruction_count.fetch_add(1, Ordering::AcqRel);

        0
    }

    /// Execute multiple instructions
    pub fn run(&mut self, max_instructions: u64) -> i32 {
        let mut count = 0;

        while count < max_instructions {
            if self.step() != 0 {
                break;
            }
            count += 1;
        }

        count as i32
    }

    /// Read a register
    pub fn read_register(&self, reg: u8) -> VmValue {
        self.registers[reg as usize]
    }

    /// Write a register
    pub fn write_register(&mut self, reg: u8, value: VmValue) {
        self.registers[reg as usize] = value;
    }

    /// Push a value onto the stack
    pub fn push(&mut self, value: VmValue) -> i32 {
        let sp = self.stack_ptr.load(Ordering::Acquire);
        let base = self.stack_base.load(Ordering::Acquire);

        if sp <= base {
            return -1;  // Stack overflow
        }

        // Write to stack
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let ptr = (sp - 8) as *mut VmValue;
            *ptr = value;
        }

        self.stack_ptr.store(sp - 8, Ordering::Release);
        0
    }

    /// Pop a value from the stack
    pub fn pop(&mut self) -> Option<VmValue> {
        let sp = self.stack_ptr.load(Ordering::Acquire);
        let base = self.stack_base.load(Ordering::Acquire);

        if sp >= base + 1024 * 1024 {  // 1MB stack limit
            return None;  // Stack underflow
        }

        // Read from stack
        // SAFETY: unsafe block required for low-level memory or hardware access
        let value = unsafe {
            let ptr = sp as *const VmValue;
            *ptr
        };

        self.stack_ptr.store(sp + 8, Ordering::Release);
        Some(value)
    }

    /// Get the VM state
    pub fn get_state(&self) -> VmState {
        match self.state.load(Ordering::Acquire) {
            0 => VmState::Stopped,
            1 => VmState::Running,
            2 => VmState::Paused,
            3 => VmState::Error,
            _ => VmState::Stopped,
        }
    }

    /// Get the executed instruction count
    pub fn get_instruction_count(&self) -> u64 {
        self.instruction_count.load(Ordering::Acquire)
    }
}

/// Global virtual machine instance
static mut VIRTUAL_MACHINE: VirtualMachine = VirtualMachine::new();

/// Get the global virtual machine instance
pub fn get_vm() -> &'static mut VirtualMachine {
    // SAFETY: Single-threaded access; synchronized externally.
    unsafe { &mut VIRTUAL_MACHINE }
}

/// Initialize the virtual machine
pub fn init_vm() {
    let vm = get_vm();
    // Allocate stack memory from the kernel
    // In a real implementation, this would allocate a physical
    // page for the VM stack and map it into the address space
    vm.init(0, 1024 * 1024);  // 1MB stack
}

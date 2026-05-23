/*
* Nuva OS - Kernel - Arch
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

/// Kirin9020 UART base address (assuming PL011)
const UART0_BASE: u64 = 0x09000000;

/// PL011 register offsets
const UART_DR: u64 = 0x00; // Data Register
const UART_FR: u64 = 0x18; // Flag Register

/// PL011 flag bits
const UART_FR_TXFF: u32 = 1 << 5; // Transmit FIFO full

/// Early console structure
pub struct EarlyConsole {
    base: u64,
}

impl EarlyConsole {
    /// Create new early console instance
    pub const fn new() -> Self {
        EarlyConsole { base: UART0_BASE }
    }

    /// Create with specified base address
    pub const fn with_base(base: u64) -> Self {
        EarlyConsole { base }
    }

    /// Initialize UART
    pub fn init(&self) {
        // Simple implementation: assume bootloader has already initialized UART
        // Actual implementation needs to configure baud rate, data bits, stop bits, etc.
    }

    /// Send single byte
    pub fn putc(&self, c: u8) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Wait until transmit FIFO is not full
            while self.read_fr() & UART_FR_TXFF != 0 {
                // Busy wait
            }

            // Write data register
            self.write_dr(c as u32);
        }
    }

    /// Send string
    pub fn puts(&self, s: &str) {
        for c in s.bytes() {
            if c == b'\n' {
                self.putc(b'\r');
            }
            self.putc(c);
        }
    }

    /// Send byte array
    pub fn write(&self, data: &[u8]) {
        for &c in data {
            self.putc(c);
        }
    }

    /// Read data register
    // SAFETY: The caller must ensure the UART base address is valid and
    // the data register offset is within the MMIO region.
    unsafe fn read_dr(&self) -> u32 {
        ptr::read_volatile((self.base + UART_DR) as *const u32)
    }

    /// Write data register
    // SAFETY: The caller must ensure the UART base address is valid and
    // the data register offset is within the MMIO region.
    unsafe fn write_dr(&self, value: u32) {
        ptr::write_volatile((self.base + UART_DR) as *mut u32, value);
    }

    /// Read flag register
    // SAFETY: The caller must ensure the UART base address is valid and
    // the flag register offset is within the MMIO region.
    unsafe fn read_fr(&self) -> u32 {
        ptr::read_volatile((self.base + UART_FR) as *const u32)
    }
}

/// Global early console instance
static EARLY_CONSOLE: core::sync::OnceLock<EarlyConsole> = core::sync::OnceLock::new();

/// Get early console instance
pub fn get_early_console() -> &'static EarlyConsole {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &EARLY_CONSOLE }
}

/// Initialize early console
pub fn init_early_console() {
    let console = get_early_console();
    console.init();
}

/// Early print macro
#[macro_export]
macro_rules! early_print {
    ($($arg:tt)*) => {
        $crate::debug::early_console::get_early_console().puts(
            format_args!($($arg)*).as_str()
        );
    };
}

/// Early print line macro
#[macro_export]
macro_rules! early_println {
    () => ($crate::early_print!("
"));
    ($($arg:tt)*) => {
        $crate::early_print!("{}
", format_args!($($arg)*));
    };
}

/// Output hexadecimal number
pub fn print_hex(value: u64) {
    let console = get_early_console();
    console.puts("0x");

    for i in (0..16).rev() {
        let nibble = ((value >> (i * 4)) & 0xF) as u8;
        let c = if nibble < 10 {
            b'0' + nibble
        } else {
            b'a' + nibble - 10
        };
        console.putc(c);
    }
}

/// Output decimal number
pub fn print_dec(value: i64) {
    let console = get_early_console();

    if value < 0 {
        console.putc(b'-');
        print_dec_unsigned((-value) as u64);
    } else {
        print_dec_unsigned(value as u64);
    }
}

/// Output unsigned decimal number
fn print_dec_unsigned(mut value: u64) {
    let console = get_early_console();

    if value == 0 {
        console.putc(b'0');
        return;
    }

    let mut buffer = [0u8; 20];
    let mut i = 0;

    while value > 0 {
        buffer[i] = (value % 10) as u8 + b'0';
        value /= 10;
        i += 1;
    }

    // Output in reverse
    for j in (0..i).rev() {
        console.putc(buffer[j]);
    }
}

/// Output memory region
pub fn print_memory(base: u64, size: usize) {
    let console = get_early_console();

    for i in 0..size {
        if i % 16 == 0 {
            if i > 0 {
                console.puts(
                    "
",
                );
            }
            print_hex(base + i as u64);
            console.puts(": ");
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let byte = ptr::read_volatile((base + i as u64) as *const u8);
            let high = (byte >> 4) & 0xF;
            let low = byte & 0xF;

            console.putc(if high < 10 {
                b'0' + high
            } else {
                b'a' + high - 10
            });
            console.putc(if low < 10 {
                b'0' + low
            } else {
                b'a' + low - 10
            });
            console.putc(b' ');
        }
    }

    console.puts(
        "
",
    );
}

/// Output register values
pub fn print_registers() {
    let console = get_early_console();

    console.puts(
        "=== Registers ===
",
    );

    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        // Read general registers
        macro_rules! print_reg {
            ($name:expr, $reg:expr) => {
                console.puts($name);
                console.puts(": ");
                print_hex($reg);
                console.puts(
                    "
",
                );
            };
        }

        // Read system registers
        console.puts("CurrentEL: ");
        let current_el: u64;
        core::arch::asm!("mrs {}, CurrentEL", out(reg) current_el);
        print_hex(current_el);
        console.puts(
            "
",
        );

        console.puts("SP_EL0: ");
        let sp_el0: u64;
        core::arch::asm!("mrs {}, sp_el0", out(reg) sp_el0);
        print_hex(sp_el0);
        console.puts(
            "
",
        );

        console.puts("SP_EL1: ");
        let sp_el1: u64;
        core::arch::asm!("mrs {}, sp_el1", out(reg) sp_el1);
        print_hex(sp_el1);
        console.puts(
            "
",
        );

        console.puts("ELR_EL1: ");
        let elr_el1: u64;
        core::arch::asm!("mrs {}, elr_el1", out(reg) elr_el1);
        print_hex(elr_el1);
        console.puts(
            "
",
        );

        console.puts("SPSR_EL1: ");
        let spsr_el1: u64;
        core::arch::asm!("mrs {}, spsr_el1", out(reg) spsr_el1);
        print_hex(spsr_el1);
        console.puts(
            "
",
        );

        console.puts("TTBR0_EL1: ");
        let ttbr0_el1: u64;
        core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0_el1);
        print_hex(ttbr0_el1);
        console.puts(
            "
",
        );

        console.puts("TTBR1_EL1: ");
        let ttbr1_el1: u64;
        core::arch::asm!("mrs {}, ttbr1_el1", out(reg) ttbr1_el1);
        print_hex(ttbr1_el1);
        console.puts(
            "
",
        );

        console.puts("SCTLR_EL1: ");
        let sctlr_el1: u64;
        core::arch::asm!("mrs {}, sctlr_el1", out(reg) sctlr_el1);
        print_hex(sctlr_el1);
        console.puts(
            "
",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_early_console() {
        let console = EarlyConsole::new();
        console.puts(
            "Hello, Nuva OS!
",
        );
    }
}

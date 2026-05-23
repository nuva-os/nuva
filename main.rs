/*
 * Nuva OS - Kernel Binary Entry Point
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

#![no_std]
#![no_main]

extern crate alloc;

use nuva::kernel_main;

#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

/// Kernel entry point called by bootloader.
/// This is the first code executed after the bootloader transfers control.
/// It delegates to the shared kernel_main() in lib.rs.
/// The boot_info pointer is architecture-specific:
/// ARM64: FDT pointer (passed in x0 by bootloader)
/// x86_64: Multiboot2 info pointer (passed in ebx by bootloader)
/// LoongArch64: UEFI boot info pointer (passed in a0 by firmware)
#[no_mangle]
pub extern "C" fn _start(boot_info: *const u8) -> ! {
    kernel_main(boot_info);
}

/// FFI entry point for kmain called from assembly start.S.
/// Wrapper that calls _start with the boot info pointer.
#[no_mangle]
pub extern "C" fn kmain(boot_info: *const u8) -> ! {
    kernel_main(boot_info);
}

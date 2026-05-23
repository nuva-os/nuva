/*
 * Nuva OS - Kernel - Main Entry
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
#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

mod config;

/// Kernel version
pub const KERNEL_VERSION: &str = "0.1.0";

/// Kernel name
pub const KERNEL_NAME: &str = "Nuva OS";

/// Architecture
pub const ARCH: &str = "aarch64";

/// Initialization stage enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitStage {
    /// Not initialized
    None = 0,
    /// Early initialization
    Early = 1,
    /// Memory initialization
    Memory = 2,
    /// Interrupt initialization
    Interrupt = 3,
    /// Device initialization
    Device = 4,
    /// Services initialization
    Services = 5,
    /// Complete
    Complete = 6,
}

/// Kernel state structure
pub struct KernelState {
    /// Current stage
    pub stage: AtomicU32,
    /// Boot time (milliseconds)
    pub boot_time: AtomicU32,
    /// Number of CPUs
    pub nr_cpus: AtomicU32,
    /// Total memory (bytes)
    pub total_memory: AtomicU64,
    /// SMP supported
    pub smp_supported: bool,
    /// ACPI supported
    pub acpi_supported: bool,
    /// Device tree supported
    pub dtb_supported: bool,
}

impl KernelState {
    pub const fn new() -> Self {
        KernelState {
            stage: AtomicU32::new(InitStage::None as u32),
            boot_time: AtomicU32::new(0),
            nr_cpus: AtomicU32::new(1),
            total_memory: AtomicU64::new(0),
            smp_supported: false,
            acpi_supported: false,
            dtb_supported: false,
        }
    }
    
    /// Get current stage
    pub fn get_stage(&self) -> InitStage {
        match self.stage.load(Ordering::Acquire) {
            0 => InitStage::None,
            1 => InitStage::Early,
            2 => InitStage::Memory,
            3 => InitStage::Interrupt,
            4 => InitStage::Device,
            5 => InitStage::Services,
            6 => InitStage::Complete,
            _ => InitStage::None,
        }
    }
    
    /// Set current stage
    pub fn set_stage(&self, stage: InitStage) {
        self.stage.store(stage as u32, Ordering::Release);
    }
}

/// Global kernel state
static KERNEL_STATE: core::sync::OnceLock<KernelState> = core::sync::OnceLock::new();

/// Get kernel state reference
pub fn kernel_state() -> &'static KernelState {
    KERNEL_STATE.get_or_init(KernelState::new)
}

/// Early initialization
fn early_init() {
    let state = get_kernel_state();
    state.set_stage(InitStage::Early);
    
    log_info!("=== {} v{} ===", KERNEL_NAME, KERNEL_VERSION);
    log_info!("Architecture: {}", ARCH);
    log_info!("Early initialization...");
    
    // Initialize console
    console_init();
    
    // Parse boot arguments
    parse_boot_args();
    
    log_info!("Early init complete");
}

/// Memory initialization
fn memory_init() {
    let state = get_kernel_state();
    state.set_stage(InitStage::Memory);
    
    log_info!("Memory initialization...");
    
    // Detect physical memory
    let total_memory = detect_memory();
    state.total_memory.store(total_memory, Ordering::Release);
    
    // Initialize physical memory management
    crate::mm::init_phys_mem(total_memory);
    crate::mm::init_buddy((total_memory / 4096) as u32);
    
    // Initialize virtual memory
    crate::mm::init_vm();
    
    log_info!("Memory: {} MB", total_memory / (1024 * 1024));
    log_info!("Memory init complete");
}

/// Interrupt initialization
fn interrupt_init() {
    let state = get_kernel_state();
    state.set_stage(InitStage::Interrupt);
    
    log_info!("Interrupt initialization...");
    
    // Initialize interrupt controller
    crate::driver::init_irq_manager();
    
    // Initialize trap handling
    crate::trap::init_trap();
    
    // Initialize system calls
    crate::syscall::init_syscall();
    
    log_info!("Interrupt init complete");
}

/// Device initialization
fn device_init() {
    let state = get_kernel_state();
    state.set_stage(InitStage::Device);
    
    log_info!("Device initialization...");
    
    // Initialize device manager
    crate::driver::init_device_manager();
    
    // Initialize timer
    crate::timer::init_timer();
    
    // Initialize time
    crate::time::init_time();
    
    // Probe devices
    probe_devices();
    
    log_info!("Device init complete");
}

/// Services initialization
fn services_init() {
    let state = get_kernel_state();
    state.set_stage(InitStage::Services);
    
    log_info!("Services initialization...");
    
    // Initialize scheduler
    let nr_cpus = state.nr_cpus.load(Ordering::Acquire);
    crate::sched::init_scheduler(nr_cpus);
    
    // Initialize file system
    crate::fs::init_vfs();
    
    // Initialize network stack
    crate::net::init_net_stack();
    
    // Initialize IPC
    crate::ipc::init_hybrid_ipc();
    
    // Initialize BSD compatibility layer
    crate::bsd::init_bsd_compat();
    
    // Initialize power management
    crate::power::init_power_manager(nr_cpus);
    
    // Initialize security module
    crate::security::init_security();
    
    log_info!("Services init complete");
}

/// Initialization complete
fn init_complete() {
    let state = get_kernel_state();
    state.set_stage(InitStage::Complete);
    
    log_info!("=== Initialization complete ===");
    log_info!("CPUs: {}", state.nr_cpus.load(Ordering::Acquire));
    log_info!("Memory: {} MB", state.total_memory.load(Ordering::Acquire) / (1024 * 1024));
    
    // Print kernel info
    print_kernel_info();
}

/// Kernel main entry
pub fn kernel_main() -> ! {
    // Early initialization
    early_init();
    
    // Memory initialization
    memory_init();
    
    // Interrupt initialization
    interrupt_init();
    
    // Device initialization
    device_init();
    
    // Services initialization
    services_init();
    
    // Initialization complete
    init_complete();
    
    // Start init process
    start_init_process();
    
    // Main loop
    kernel_loop()
}

/// Kernel main loop
fn kernel_loop() -> ! {
    loop {
        // Handle kernel tasks
        handle_kernel_tasks();
        
        // Yield CPU
        core::hint::spin_loop();
    }
}

/// Handle kernel tasks
fn handle_kernel_tasks() {
    // Process deferred work
    process_deferred_work();
    
    // Check for pending signals
    check_pending_signals();
    
    // Update system statistics
    update_system_stats();
}

/// Process deferred work (softirq, tasklets, workqueue)
fn process_deferred_work() {
    // TODO: Process softirq vectors
    // TODO: Run tasklets
    // TODO: Execute workqueue items
}

/// Check for pending signals
fn check_pending_signals() {
    // TODO: Check current task for pending signals
}

/// Update system statistics
fn update_system_stats() {
    // TODO: Update load average
    // TODO: Update CPU usage
}

/// Start init process
fn start_init_process() {
    log_info!("Starting init process...");
    
    // Create init process (PID 1)
    let init_pid = create_init_process();
    
    if init_pid == 0 {
        log_error!("Failed to create init process!");
        return;
    }
    
    log_info!("Init process created with PID {}", init_pid);
    
    // TODO: Load init program (/sbin/init or /init)
    // TODO: Start init process execution
}

/// Create init process
fn create_init_process() -> u32 {
    // TODO: Use process manager to create init
    // For now, return PID 1
    1
}

/// Initialize console
fn console_init() {
    log_debug!("Initializing console...");
    
    // Initialize early console (UART)
    init_early_console();
    
    // Initialize VT console if available
    init_vt_console();
    
    log_debug!("Console initialized");
}

/// Initialize early console (UART)
fn init_early_console() {
    // TODO: Initialize UART for early boot messages
    // On ARM64, this typically uses PL011 or 8250 UART
    
    // For QEMU virt machine, UART is at 0x09000000
    #[cfg(target_arch = "aarch64")]
    {
        // Initialize PL011 UART
        init_pl011_uart(0x0900_0000);
    }
}

/// Initialize PL011 UART
#[cfg(target_arch = "aarch64")]
fn init_pl011_uart(base: u64) {
    // PL011 UART registers
    const UART_DR: u64 = 0x00;   /* Data Register */
    const UART_FR: u64 = 0x18;   /* Flag Register */
    const UART_IBRD: u64 = 0x24; /* Integer Baud Rate */
    const UART_FBRD: u64 = 0x28; /* Fractional Baud Rate */
    const UART_LCRH: u64 = 0x2C; /* Line Control */
    const UART_CR: u64 = 0x30;   /* Control Register */
    
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        let ptr = base as *mut u32;
        
        // Disable UART
        core::ptr::write_volatile(ptr.add(UART_CR as usize / 4), 0);
        
        // Wait for TX to complete
        while (core::ptr::read_volatile(ptr.add(UART_FR as usize / 4)) & (1 << 3)) != 0 {}
        
        // Set baud rate (115200 for 24MHz clock)
        core::ptr::write_volatile(ptr.add(UART_IBRD as usize / 4), 26);
        core::ptr::write_volatile(ptr.add(UART_FBRD as usize / 4), 3);
        
        // 8 bits, no parity, 1 stop bit
        core::ptr::write_volatile(ptr.add(UART_LCRH as usize / 4), 0x70);
        
        // Enable UART, TX, RX
        core::ptr::write_volatile(ptr.add(UART_CR as usize / 4), 0x301);
    }
}

/// Initialize VT console
fn init_vt_console() {
    // TODO: Initialize virtual terminal console
}

/// Parse boot arguments
fn parse_boot_args() {
    // TODO: Parse kernel command line from bootloader
    // Common parameters:
    // - console=ttyAMA0,115200
    // - root=/dev/mmcblk0p2
    // - mem=1024M
    // - init=/sbin/init
}

/// Detect memory
fn detect_memory() -> u64 {
    // Try to get memory from device tree or ACPI
    #[cfg(target_arch = "aarch64")]
    {
        // Try device tree first
        if let Some(mem) = detect_memory_from_dtb() {
            return mem;
        }
    }
    
    // Default: 1GB
    1024 * 1024 * 1024
}

/// Detect memory from device tree
#[cfg(target_arch = "aarch64")]
fn detect_memory_from_dtb() -> Option<u64> {
    // TODO: Parse device tree for memory nodes
    // Look for /memory@xxxxxxxx nodes
    None
}

/// Probe devices
fn probe_devices() {
    log_debug!("Probing devices...");
    
    // Probe based on platform
    #[cfg(target_arch = "aarch64")]
    {
        // Try device tree first
        probe_devices_from_dtb();
    }
    
    // Probe PCI devices if available
    probe_pci_devices();
    
    // Probe platform devices
    probe_platform_devices();
    
    log_debug!("Device probing complete");
}

/// Probe devices from device tree
#[cfg(target_arch = "aarch64")]
fn probe_devices_from_dtb() {
    // TODO: Parse device tree and create devices
}

/// Probe PCI devices
fn probe_pci_devices() {
    // TODO: Scan PCI bus for devices
}

/// Probe platform devices
fn probe_platform_devices() {
    // TODO: Probe platform-specific devices
}

/// Print kernel information
fn print_kernel_info() {
    log_info!("Kernel Information:");
    log_info!("  Name: {}", KERNEL_NAME);
    log_info!("  Version: {}", KERNEL_VERSION);
    log_info!("  Architecture: {}", ARCH);
}

/// Kernel entry point (called from assembly)
#[no_mangle]
pub extern "C" fn _start() -> ! {
    kernel_main()
}

/// Panic handler
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    log_error!("KERNEL PANIC!");
    
    loop {
        core::hint::spin_loop();
    }
}

/// Kernel memory allocator
/// Uses slab allocator for small allocations and buddy allocator for large ones.
mod allocator {
    use core::alloc::{GlobalAlloc, Layout};
    use core::sync::atomic::{AtomicBool, Ordering};
    
    /// Allocator initialized flag
    static INITIALIZED: AtomicBool = AtomicBool::new(false);
    
    /// Heap start address
    static mut HEAP_START: u64 = 0;
    
    /// Heap size
    static mut HEAP_SIZE: u64 = 0;
    
    /// Heap current pointer
    static mut HEAP_PTR: u64 = 0;
    
    /// Small allocation threshold (use slab)
    const SMALL_THRESHOLD: usize = 4096;
    
    struct KernelAllocator;
    
    // SAFETY: KernelAllocator is the global allocator. It is safe to implement
    // GlobalAlloc because all allocation functions (bump, slab, buddy) return
    // valid memory or null, and deallocation functions handle null and
    // correct-size pointers correctly.
    unsafe impl GlobalAlloc for KernelAllocator {
        // SAFETY: Delegates to bump_alloc (before init) or slab/buddy_alloc
        // (after init). All sub-allocators return valid aligned memory or null.
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let size = layout.size();
            let align = layout.align();
            
            // If allocator not initialized, use bump allocation
            if !INITIALIZED.load(Ordering::Acquire) {
                return bump_alloc(size, align);
            }
            
            // Use slab for small allocations
            if size <= SMALL_THRESHOLD {
                slab_alloc(size, align)
            } else {
                // Use buddy for large allocations
                buddy_alloc(size, align)
            }
        }
        
        // SAFETY: Delegates to slab_free or buddy_free depending on size.
        // Both sub-allocators correctly handle the pointer and layout.
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            let size = layout.size();
            
            if !INITIALIZED.load(Ordering::Acquire) {
                // Bump allocator doesn't free
                return;
            }
            
            if size <= SMALL_THRESHOLD {
                slab_free(ptr, size);
            } else {
                buddy_free(ptr, size);
            }
        }
    }
    
    /// Bump allocator for early boot
    // SAFETY: Caller must ensure HEAP_PTR and bounds are valid. The bump
    // allocator never frees, so all returned pointers remain valid.
    unsafe fn bump_alloc(size: usize, align: usize) -> *mut u8 {
        // Align heap pointer
        let aligned = (HEAP_PTR + align as u64 - 1) & !(align as u64 - 1);
        let new_ptr = aligned + size as u64;
        
        // Check bounds
        if new_ptr > HEAP_START + HEAP_SIZE {
            return core::ptr::null_mut();
        }
        
        HEAP_PTR = new_ptr;
        aligned as *mut u8
    }
    
    /// Slab allocator for small allocations
    // SAFETY: Delegates to buddy_alloc; see its SAFETY comment.
    unsafe fn slab_alloc(size: usize, align: usize) -> *mut u8 {
        // TODO: Use actual slab allocator
        // For now, use buddy allocator
        buddy_alloc(size, align)
    }
    
    /// Slab free
    // SAFETY: Delegates to buddy_free; see its SAFETY comment.
    unsafe fn slab_free(ptr: *mut u8, size: usize) {
        // TODO: Use actual slab allocator
        buddy_free(ptr, size);
    }
    
    /// Buddy allocator for large allocations
    // SAFETY: Currently returns null (unimplemented). When implemented,
    // will return page-aligned memory from the buddy system.
    unsafe fn buddy_alloc(size: usize, align: usize) -> *mut u8 {
        // Calculate order needed
        let page_size = 4096usize;
        let pages_needed = (size + page_size - 1) / page_size;
        let order = pages_needed.next_power_of_two().trailing_zeros() as usize;
        
        // Allocate from buddy allocator
        // TODO: Call actual buddy allocator
        // crate::mm::buddy::alloc_pages(order)
        
        core::ptr::null_mut()
    }
    
    /// Buddy free
    // SAFETY: Currently a no-op (unimplemented). When implemented,
    // will return the page to the buddy system.
    unsafe fn buddy_free(ptr: *mut u8, _size: usize) {
        // TODO: Free to buddy allocator
        // let page = crate::mm::mem_map::virt_to_page(ptr as u64);
        // crate::mm::buddy::free_page(page);
    }
    
    /// Initialize kernel heap
    pub fn init_heap(start: u64, size: u64) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            HEAP_START = start;
            HEAP_SIZE = size;
            HEAP_PTR = start;
            INITIALIZED.store(true, Ordering::Release);
        }
        
        log_info!("Kernel heap initialized: {:#x} - {:#x}", start, start + size);
    }
    
    #[global_allocator]
    static ALLOCATOR: KernelAllocator = KernelAllocator;
}

/// Initialize kernel heap
pub fn init_kernel_heap(start: u64, size: u64) {
    allocator::init_heap(start, size);
}

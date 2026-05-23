# Nuva Kernel CMake Configuration

# Kernel Source Files
set(KERNEL_ASM_SOURCES
    arch/arm64/boot/start.S
    arch/arm64/boot/entry.S
    arch/arm64/mm/page_table.S
    arch/arm64/trap/exception.S
    arch/arm64/trap/entry.S
)

set(KERNEL_C_SOURCES
    arch/arm64/boot/main.c
    arch/arm64/mm/mmu.c
    arch/arm64/trap/trap.c
)

set(KERNEL_RUST_SOURCES
    lib.rs
    arch/arm64/boot/fdt.rs
    arch/arm64/mm/mod.rs
    arch/arm64/mm/page_table.rs
    arch/arm64/trap/mod.rs
    sched/task.rs
    sched/cfs.rs
    sched/rt.rs
    sched/load_balance.rs
    mm/mod.rs
    mm/buddy.rs
    mm/slab.rs
    mm/vma.rs
    mm/fault.rs
    sync/atomic.rs
    sync/spinlock.rs
    sync/mutex.rs
    sync/rwlock.rs
    sync/wait_queue.rs
    fs/vfs/mod.rs
    fs/vfs/file.rs
    fs/vfs/dcache.rs
    fs/vfs/inode.rs
    fs/nuvafs/mod.rs
    # Binder removed - using NuvaIPC instead
    drivers/irqchip/gic.rs
    interrupt/mod.rs
    debug/printk.rs
    panic.rs
)

# Create Kernel Library
add_library(nuva_kernel STATIC
    ${KERNEL_ASM_SOURCES}
    ${KERNEL_C_SOURCES}
)

# Set Compile Options
target_compile_options(nuva_kernel PRIVATE
    -mcmodel=kernel
    -mno-red-zone
    -mno-mmx
    -mno-sse
    -mno-sse2
)

# Include Directories
target_include_directories(nuva_kernel PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_SOURCE_DIR}/sysroot/include
)

# Link Options
target_link_options(nuva_kernel PRIVATE
    -T${NUVA_LINKER_SCRIPT}
    -nostdlib
    -static
    -Wl,--gc-sections
    -Wl,-Map=nuva_kernel.map
)

# Rust Kernel Modules
if(NUVA_BUILD_NUVA_LANG)
    add_custom_target(nuva_kernel_rust
        COMMAND cargo build --target aarch64-unknown-none --release
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/kernel
        COMMENT "Building Nuva Kernel Rust modules..."
    )
    add_dependencies(nuva_kernel nuva_kernel_rust)
endif()

# Generate Kernel Header File
configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/include/nuva/version.h.in
    ${CMAKE_CURRENT_BINARY_DIR}/include/nuva/version.h
)

# Install
install(TARGETS nuva_kernel
    ARCHIVE DESTINATION lib
)

install(FILES ${CMAKE_BINARY_DIR}/nuva_kernel.map
    DESTINATION lib
)

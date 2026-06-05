# CMake cross-compilation toolchain for RISC-V 64-bit (QEMU virt)
# Nuva OS - Copyright (C) 2026 Nuva OS Team
# Licensed under Apache License, Version 2.0

set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR riscv64)

# Toolchain prefix
set(CROSS_COMPILE riscv64-unknown-elf-)

# Compilers
set(CMAKE_C_COMPILER ${CROSS_COMPILE}gcc)
set(CMAKE_CXX_COMPILER ${CROSS_COMPILE}g++)
set(CMAKE_ASM_COMPILER ${CROSS_COMPILE}gcc)

# Flags
set(CMAKE_C_FLAGS "-march=rv64gc -mabi=lp64d -mcmodel=medany -ffreestanding -nostdlib" CACHE STRING "")
set(CMAKE_CXX_FLAGS "${CMAKE_C_FLAGS}" CACHE STRING "")
set(CMAKE_ASM_FLAGS "-march=rv64gc -mabi=lp64d -mcmodel=medany" CACHE STRING "")

# Search paths
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)

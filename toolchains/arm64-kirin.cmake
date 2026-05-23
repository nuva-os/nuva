# Nuva OS Cross-Compilation Toolchain Configuration
# Target Platform: ARM64 (Kirin9020)

set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR aarch64)

# Target Triple
set(TARGET_TRIPLE aarch64-nuva-elf)

# Compiler Paths (using LLVM/Clang)
set(CMAKE_C_COMPILER clang)
set(CMAKE_CXX_COMPILER clang++)
set(CMAKE_ASM_COMPILER clang)

# Set Cross-Compilation Target
set(CMAKE_C_COMPILER_TARGET ${TARGET_TRIPLE})
set(CMAKE_CXX_COMPILER_TARGET ${TARGET_TRIPLE})
set(CMAKE_ASM_COMPILER_TARGET ${TARGET_TRIPLE})

# Linker
set(CMAKE_LINKER lld)
set(CMAKE_AR llvm-ar)
set(CMAKE_OBJCOPY llvm-objcopy)
set(CMAKE_OBJDUMP llvm-objdump)
set(CMAKE_READELF llvm-readelf)
set(CMAKE_NM llvm-nm)
set(CMAKE_RANLIB llvm-ranlib)

# Sysroot Path
set(CMAKE_SYSROOT ${CMAKE_SOURCE_DIR}/sysroot)
set(CMAKE_FIND_ROOT_PATH ${CMAKE_SYSROOT})

# Search Path Settings
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)

# Rust Toolchain Configuration
set(RUSTC rustc)
set(CARGO cargo)
set(RUST_TARGET aarch64-nuva-none)

# Compiler Flags
set(CMAKE_C_FLAGS_INIT "--target=${TARGET_TRIPLE} -ffreestanding -nostdlib")
set(CMAKE_CXX_FLAGS_INIT "--target=${TARGET_TRIPLE} -ffreestanding -nostdlib")
set(CMAKE_ASM_FLAGS_INIT "--target=${TARGET_TRIPLE}")

# Linker Flags
set(CMAKE_EXE_LINKER_FLAGS_INIT "-fuse-ld=lld -nostdlib")

# Print Toolchain Information
message(STATUS "Cross-compile toolchain for ${TARGET_TRIPLE}")
message(STATUS "C Compiler: ${CMAKE_C_COMPILER}")
message(STATUS "C++ Compiler: ${CMAKE_CXX_COMPILER}")
message(STATUS "Linker: ${CMAKE_LINKER}")
message(STATUS "Sysroot: ${CMAKE_SYSROOT}")

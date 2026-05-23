# Nuva OS - CMake Build Configuration
# Next-Generation Operating System

cmake_minimum_required(VERSION 3.20)

project(NuvaOS
    VERSION 1.0.0
    DESCRIPTION "Nuva Operating System for Kirin9020"
    LANGUAGES C CXX ASM
)

# Set C++ Standard
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Project Options
option(NUVA_BUILD_KERNEL "Build Nuva Kernel" ON)
option(NUVA_BUILD_HAL "Build Hardware Abstraction Layer" ON)
option(NUVA_BUILD_SERVICES "Build System Services" ON)
option(NUVA_BUILD_NUVA_BRAIN "Build Nuva Brain AI Engine" ON)
option(NUVA_BUILD_NUVA_LANG "Build Nuva Language Runtime" ON)
option(NUVA_BUILD_TESTS "Build Tests" ON)
option(NUVA_ENABLE_DEBUG "Enable Debug Symbols" ON)
option(NUVA_ENABLE_ASAN "Enable Address Sanitizer" OFF)

# Target Platform Configuration
set(NUVA_TARGET_ARCH "arm64" CACHE STRING "Target architecture")
set(NUVA_TARGET_CPU "kirin9020" CACHE STRING "Target CPU")
set(NUVA_TARGET_TRIPLE "aarch64-nuva-elf" CACHE STRING "Target triple")

# Compilation Options
set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -target ${NUVA_TARGET_TRIPLE}")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -target ${NUVA_TARGET_TRIPLE}")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -target ${NUVA_TARGET_TRIPLE}")

# Common Compile Flags
add_compile_options(
    -Wall
    -Wextra
    -Werror
    -fno-builtin
    -fno-stack-protector
    -fno-pic
    -fno-pie
    -ffreestanding
    -nostdlib
    -nostdinc
    -mcpu=cortex-a76    # Kirin9020 big core
    -mtune=cortex-a76
)

# Debug Options
if(NUVA_ENABLE_DEBUG)
    add_compile_options(-g -O0 -DDEBUG)
else()
    add_compile_options(-O2 -DNDEBUG)
endif()

# Address Sanitizer
if(NUVA_ENABLE_ASAN)
    add_compile_options(-fsanitize=address)
    add_link_options(-fsanitize=address)
endif()

# Include Directories
include_directories(
    ${CMAKE_SOURCE_DIR}/kernel/include
    ${CMAKE_SOURCE_DIR}/hal/include
    ${CMAKE_SOURCE_DIR}/sysroot/include
)

# Linker Script
set(NUVA_LINKER_SCRIPT ${CMAKE_SOURCE_DIR}/kernel/arch/arm64/linker.ld)

# Subdirectories
if(NUVA_BUILD_KERNEL)
    add_subdirectory(kernel)
endif()

if(NUVA_BUILD_HAL)
    add_subdirectory(hal)
endif()

if(NUVA_BUILD_SERVICES)
    add_subdirectory(services)
endif()

if(NUVA_BUILD_TESTS)
    add_subdirectory(tests)
endif()

# Custom Target: Generate System Image
add_custom_target(nuva_image
    DEPENDS nuva_kernel nuva_hal nuva_services
    COMMAND ${CMAKE_COMMAND} -E make_directory ${CMAKE_BINARY_DIR}/images
    COMMAND ${CMAKE_OBJCOPY} -O binary $<TARGET_FILE:nuva_kernel> ${CMAKE_BINARY_DIR}/images/nuva_kernel.bin
    COMMAND ${CMAKE_COMMAND} -E echo "Nuva OS image generated at ${CMAKE_BINARY_DIR}/images/"
    COMMENT "Generating Nuva OS system image..."
)

# Custom Target: Clean All Build Artifacts
add_custom_target(clean_all
    COMMAND ${CMAKE_COMMAND} -E remove_directory ${CMAKE_BINARY_DIR}
    COMMENT "Cleaning all build artifacts..."
)

# Print Configuration Information
message(STATUS "========================================")
message(STATUS "Nuva OS Build Configuration")
message(STATUS "========================================")
message(STATUS "Version:        ${PROJECT_VERSION}")
message(STATUS "Target Arch:    ${NUVA_TARGET_ARCH}")
message(STATUS "Target CPU:     ${NUVA_TARGET_CPU}")
message(STATUS "Target Triple:  ${NUVA_TARGET_TRIPLE}")
message(STATUS "Build Type:     ${CMAKE_BUILD_TYPE}")
message(STATUS "Debug:          ${NUVA_ENABLE_DEBUG}")
message(STATUS "Build Kernel:   ${NUVA_BUILD_KERNEL}")
message(STATUS "Build HAL:      ${NUVA_BUILD_HAL}")
message(STATUS "Build Services: ${NUVA_BUILD_SERVICES}")
message(STATUS "Build Nuva Brain: ${NUVA_BUILD_NUVA_BRAIN}")
message(STATUS "Build Tests:    ${NUVA_BUILD_TESTS}")
message(STATUS "========================================")

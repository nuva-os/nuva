#!/bin/bash
# Nuva OS
#
# Copyright (C) 2026 Nuva OS Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# # Nuva OS Build Script
# # Usage: ./scripts/build.sh [target] [options]

set -e

# # Color Output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# # Project Root Directory
NUVA_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${NUVA_ROOT}/build"
BUILD_TYPE="Release"

# # Print Help Information
print_help() {
    echo "Nuva OS Build System"
    echo ""
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  configure    Configure build system"
    echo "  build        Build project"
    echo "  clean        Clean build artifacts"
    echo "  test         Run tests"
    echo "  image        Generate system image"
    echo "  help         Show help information"
    echo ""
    echo "Options:"
    echo "  --debug      Debug build"
    echo "  --release    Release build (default)"
    echo "  --clean      Clean before building"
    echo "  --verbose    Verbose output"
    echo "  -j N         Parallel compilation jobs"
    echo ""
    echo "Examples:"
    echo "  $0 configure --debug"
    echo "  $0 build -j8"
    echo "  $0 image"
}

# # Check Dependencies
check_dependencies() {
    echo -e "${BLUE}Checking build dependencies...${NC}"

    local missing=()

    # # check CMake
    if ! command -v cmake &> /dev/null; then
        missing+=("cmake")
    fi

    # # check Ninja
    if ! command -v ninja &> /dev/null; then
        missing+=("ninja")
    fi

    # # check Clang
    if ! command -v clang &> /dev/null; then
        missing+=("clang")
    fi

    # # check Rust
    if ! command -v rustc &> /dev/null; then
        missing+=("rustc")
    fi

    if [ ${#missing[@]} -ne 0 ]; then
        echo -e "${RED}Missing dependencies:${NC}"
        for dep in "${missing[@]}"; do
            echo -e "  ${RED}- $dep${NC}"
        done
        echo ""
        echo "Please install the missing dependencies and try again."
        exit 1
    fi

    echo -e "${GREEN}All dependencies satisfied${NC}"
    echo ""
}

# # Configure Build
configure() {
    echo -e "${BLUE}Configuring Nuva OS build system...${NC}"

    local cmake_args=(
        -G Ninja
        -DCMAKE_TOOLCHAIN_FILE="${NUVA_ROOT}/toolchains/arm64-kirin9020.cmake"
        -DCMAKE_BUILD_TYPE="${BUILD_TYPE}"
        -DNUVA_ENABLE_DEBUG=$( [ "${BUILD_TYPE}" = "Debug" ] && echo "ON" || echo "OFF" )
    )

    # # Create Build Directory
    mkdir -p "${BUILD_DIR}"

    # # run CMake
    cd "${BUILD_DIR}"
    cmake "${NUVA_ROOT}" "${cmake_args[@]}"

    echo -e "${GREEN}Configuration complete${NC}"
}

# # Build Project
build() {
    echo -e "${BLUE}Building Nuva OS...${NC}"

    if [ ! -d "${BUILD_DIR}" ]; then
        echo -e "${YELLOW}Build directory does not exist, running configure first...${NC}"
        configure
    fi

    cd "${BUILD_DIR}"

    local parallel_jobs=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

    cmake --build . -- -j${parallel_jobs}

    echo -e "${GREEN}Build complete${NC}"
}

# # Clean Build
clean() {
    echo -e "${YELLOW}Cleaning build artifacts...${NC}"

    if [ -d "${BUILD_DIR}" ]; then
        rm -rf "${BUILD_DIR}"
    fi

    echo -e "${GREEN}Clean complete${NC}"
}

# # Run Tests
run_tests() {
    echo -e "${BLUE}Running tests...${NC}"

    if [ ! -d "${BUILD_DIR}" ]; then
        echo -e "${RED}Please build the project first${NC}"
        exit 1
    fi

    cd "${BUILD_DIR}"
    ctest --output-on-failure

    echo -e "${GREEN}Tests complete${NC}"
}

# # Generate Image
generate_image() {
    echo -e "${BLUE}Generating Nuva OS system image...${NC}"

    if [ ! -d "${BUILD_DIR}" ]; then
        echo -e "${RED}Please build the project first${NC}"
        exit 1
    fi

    cd "${BUILD_DIR}"
    cmake --build . --target nuva_image

    echo -e "${GREEN}System image generated: ${BUILD_DIR}/images/${NC}"
}

# # Parse Arguments
COMMAND=""
CLEAN_FIRST=false
VERBOSE=false
PARALLEL_JOBS=""

while [[ $# -gt 0 ]]; do
    case $1 in
        configure|build|clean|test|image|help)
            COMMAND=$1
            shift
            ;;
        --debug)
            BUILD_TYPE="Debug"
            shift
            ;;
        --release)
            BUILD_TYPE="Release"
            shift
            ;;
        --clean)
            CLEAN_FIRST=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        -j)
            PARALLEL_JOBS=$2
            shift 2
            ;;
        -h|--help)
            print_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            print_help
            exit 1
            ;;
    esac
done

# # Execute Command
case ${COMMAND} in
    configure)
        check_dependencies
        [ "${CLEAN_FIRST}" = true ] && clean
        configure
        ;;
    build)
        check_dependencies
        [ "${CLEAN_FIRST}" = true ] && clean
        build
        ;;
    clean)
        clean
        ;;
    test)
        run_tests
        ;;
    image)
        generate_image
        ;;
    help|"")
        print_help
        ;;
    *)
        echo -e "${RED}Unknown command: ${COMMAND}${NC}"
        print_help
        exit 1
        ;;
esac

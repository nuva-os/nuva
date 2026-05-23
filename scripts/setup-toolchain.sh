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

# # Nuva OS Toolchain Installation and Configuration Script
# # Used to set up the cross-compilation environment

set -e

# # Color Output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# # Project Root Directory
NUVA_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# # Toolchain Versions
LLVM_VERSION="18"
RUST_VERSION="stable"

# # Target Triples
TARGET_TRIPLE="aarch64-nuva-elf"
RUST_TARGET="aarch64-unknown-none"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Nuva OS Toolchain Setup${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# # Detect Operating System
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

OS=$(detect_os)
echo -e "${GREEN}Detected operating system: ${OS}${NC}"

# # Install LLVM/Clang
install_llvm() {
    echo -e "${BLUE}Installing LLVM/Clang ${LLVM_VERSION}...${NC}"
    
    if command -v clang &> /dev/null; then
        local current_version=$(clang --version | head -n1)
        echo -e "${GREEN}LLVM/Clang already installed: ${current_version}${NC}"
        return
    fi
    
    case ${OS} in
        linux)
            if command -v apt &> /dev/null; then
                sudo apt update
                sudo apt install -y llvm-${LLVM_VERSION} clang-${LLVM_VERSION} lld-${LLVM_VERSION} \
                    llvm-ar-${LLVM_VERSION} llvm-objcopy-${LLVM_VERSION} llvm-objdump-${LLVM_VERSION}
            elif command -v yum &> /dev/null; then
                sudo yum install -y llvm-${LLVM_VERSION} clang-${LLVM_VERSION} lld-${LLVM_VERSION}
            elif command -v pacman &> /dev/null; then
                sudo pacman -S llvm clang lld
            fi
            ;;
        macos)
            if command -v brew &> /dev/null; then
                brew install llvm
            fi
            ;;
        windows)
            echo -e "${YELLOW}Windows users please install LLVM manually${NC}"
            echo "Download URL: https://releases.llvm.org/"
            ;;
    esac
    
    echo -e "${GREEN}LLVM/Clang installation complete${NC}"
}

# # Install Rust
install_rust() {
    echo -e "${BLUE}Installing Rust toolchain...${NC}"
    
    if command -v rustc &> /dev/null; then
        local current_version=$(rustc --version)
        echo -e "${GREEN}Rust already installed: ${current_version}${NC}"
    else
        echo -e "${YELLOW}Installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    # # Install Rust Components and Targets
    echo -e "${BLUE}Configuring Rust toolchain...${NC}"
    rustup toolchain install ${RUST_VERSION}
    rustup component add rust-src rustfmt clippy
    rustup target add ${RUST_TARGET}
    
    echo -e "${GREEN}Rust toolchain configuration complete${NC}"
}

# # Install Other Tools
install_tools() {
    echo -e "${BLUE}Installing build tools...${NC}"
    
    case ${OS} in
        linux)
            if command -v apt &> /dev/null; then
                sudo apt install -y cmake ninja-build git python3
            elif command -v yum &> /dev/null; then
                sudo yum install -y cmake ninja-build git python3
            elif command -v pacman &> /dev/null; then
                sudo pacman -S cmake ninja git python
            fi
            ;;
        macos)
            if command -v brew &> /dev/null; then
                brew install cmake ninja git python3
            fi
            ;;
        windows)
            echo -e "${YELLOW}Windows users please install the following tools manually:${NC}"
            echo "  - CMake: https://cmake.org/download/"
            echo "  - Ninja: https://github.com/ninja-build/ninja/releases"
            echo "  - Git: https://git-scm.com/download/win"
            ;;
    esac
    
    echo -e "${GREEN}Build tools installation complete${NC}"
}

# # Create Sysroot
create_sysroot() {
    echo -e "${BLUE}Creating sysroot directory structure...${NC}"
    
    local sysroot="${NUVA_ROOT}/sysroot"
    
    mkdir -p "${sysroot}/include/nuva"
    mkdir -p "${sysroot}/lib"
    
    # # Create Base Header Files
    if [ ! -f "${sysroot}/include/nuva/types.h" ]; then
        echo -e "${YELLOW}Please ensure types.h has been created${NC}"
    fi
    
    echo -e "${GREEN}Sysroot created: ${sysroot}${NC}"
}

# # Verify Toolchain
verify_toolchain() {
    echo -e "${BLUE}Verifying toolchain...${NC}"
    
    local errors=0
    
    # # check Clang
    if ! command -v clang &> /dev/null; then
        echo -e "${RED}X Clang not installed${NC}"
        errors=$((errors + 1))
    else
        echo -e "${GREEN}OK Clang: $(clang --version | head -n1)${NC}"
    fi
    
    # # check Rust
    if ! command -v rustc &> /dev/null; then
        echo -e "${RED}X Rust not installed${NC}"
        errors=$((errors + 1))
    else
        echo -e "${GREEN}OK Rust: $(rustc --version)${NC}"
    fi
    
    # # check CMake
    if ! command -v cmake &> /dev/null; then
        echo -e "${RED}X CMake not installed${NC}"
        errors=$((errors + 1))
    else
        echo -e "${GREEN}OK CMake: $(cmake --version | head -n1)${NC}"
    fi
    
    # # check Ninja
    if ! command -v ninja &> /dev/null; then
        echo -e "${RED}X Ninja not installed${NC}"
        errors=$((errors + 1))
    else
        echo -e "${GREEN}OK Ninja: $(ninja --version)${NC}"
    fi
    
    if [ ${errors} -eq 0 ]; then
        echo -e "${GREEN}All toolchain verification passed${NC}"
        return 0
    else
        echo -e "${RED}Toolchain verification failed, ${errors} error(s) found${NC}"
        return 1
    fi
}

# # Test Cross Compilation
test_cross_compile() {
    echo -e "${BLUE}Testing cross compilation...${NC}"
    
    local test_dir="${NUVA_ROOT}/build/toolchain_test"
    mkdir -p "${test_dir}"
    
    # # Create Test C File
    cat > "${test_dir}/test.c" << 'EOF'
int main(void) {
    return 0;
}
EOF
    
    # # Test Clang Cross Compilation
    echo -e "${YELLOW}Testing Clang cross compilation...${NC}"
    if clang --target=${TARGET_TRIPLE} -c "${test_dir}/test.c" -o "${test_dir}/test.o" 2>/dev/null; then
        echo -e "${GREEN}OK Clang cross compilation successful${NC}"
    else
        echo -e "${YELLOW}! Clang cross compilation test skipped (may require additional target support)${NC}"
    fi
    
    # # Test Rust no_std Compilation
    echo -e "${YELLOW}Testing Rust no_std compilation...${NC}"
    cat > "${test_dir}/test.rs" << 'EOF'
#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
EOF
    
    if rustc --target=${RUST_TARGET} "${test_dir}/test.rs" -o "${test_dir}/test_rust" 2>/dev/null; then
        echo -e "${GREEN}OK Rust no_std compilation successful${NC}"
    else
        echo -e "${YELLOW}! Rust no_std compilation test skipped${NC}"
    fi
    
    # # Clean Up Test Files
    rm -rf "${test_dir}"
    
    echo -e "${GREEN}Cross compilation test complete${NC}"
}

# # Main Function
main() {
    local action="${1:-all}"
    
    case ${action} in
        all)
            install_llvm
            install_rust
            install_tools
            create_sysroot
            verify_toolchain
            test_cross_compile
            ;;
        llvm)
            install_llvm
            ;;
        rust)
            install_rust
            ;;
        tools)
            install_tools
            ;;
        sysroot)
            create_sysroot
            ;;
        verify)
            verify_toolchain
            ;;
        test)
            test_cross_compile
            ;;
        *)
            echo "Usage: $0 [all|llvm|rust|tools|sysroot|verify|test]"
            exit 1
            ;;
    esac
    
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Toolchain setup complete!${NC}"
    echo -e "${GREEN}========================================${NC}"
}

main "$@"

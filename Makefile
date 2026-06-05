# Nuva OS Makefile
#
# Build system for Nuva OS kernel and components

# Project info
PROJECT_NAME := nuva
VERSION := 1.0.0

# Directories
SRCDIR := .
BUILDDIR := build
TARGETDIR := target

# Tools
CARGO := cargo
RUSTC := rustc
CC := gcc
CXX := g++
QEMU := qemu-system-x86_64

# Targets
TARGET_X86 := x86_64-unknown-none
TARGET_ARM := aarch64-unknown-none
TARGET_RISCV := riscv64-unknown-none

# Features
FEATURES_X86 := x64
FEATURES_ARM := arm64
FEATURES_RISCV := riscv64
FEATURES_QEMU_VIRT := riscv64,qemu_virt
FEATURES_KIRIN := arm64,kirin9020
FEATURES_SNAPDRAGON := arm64,snapdragon8gen4

# Colors
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
NC := \033[0m

# Default target
.PHONY: all
all: build

# ============================================================================
# Build targets
# ============================================================================

.PHONY: build
build: build-x86

.PHONY: build-x86
build-x86:
	@echo "$(BLUE)Building for x86_64...$(NC)"
	$(CARGO) build --target $(TARGET_X86) --release --features $(FEATURES_X86)

.PHONY: build-arm
build-arm:
	@echo "$(BLUE)Building for ARM64...$(NC)"
	$(CARGO) build --target $(TARGET_ARM) --release --features $(FEATURES_ARM)

.PHONY: build-kirin
build-kirin:
	@echo "$(BLUE)Building for Kirin 9020...$(NC)"
	$(CARGO) build --target $(TARGET_ARM) --release --features $(FEATURES_KIRIN)

.PHONY: build-snapdragon
build-snapdragon:
	@echo "$(BLUE)Building for Snapdragon 8 Gen 4...$(NC)"
	$(CARGO) build --target $(TARGET_ARM) --release --features $(FEATURES_SNAPDRAGON)

.PHONY: build-all
build-all: build-x86 build-arm build-riscv
	@echo "$(GREEN)All targets built successfully!$(NC)"

.PHONY: build-riscv
build-riscv:
	@echo "$(BLUE)Building for RISC-V 64...$(NC)"
	$(CARGO) build --target $(TARGET_RISCV) --release --features $(FEATURES_RISCV)

# ============================================================================
# Development builds
# ============================================================================

.PHONY: dev
dev:
	@echo "$(YELLOW)Development build...$(NC)"
	$(CARGO) build --target $(TARGET_X86)

.PHONY: debug
debug:
	@echo "$(YELLOW)Debug build with debug info...$(NC)"
	$(CARGO) build --target $(TARGET_X86) --features debug

# ============================================================================
# Testing
# ============================================================================

.PHONY: test
test:
	@echo "$(BLUE)Running tests...$(NC)"
	$(CARGO) test

.PHONY: test-all
test-all: test coverage

.PHONY: coverage
coverage:
	@echo "$(BLUE)Generating test coverage...$(NC)"
	$(CARGO) tarpaulin --out Xml --out Html

# ============================================================================
# Code quality
# ============================================================================

.PHONY: fmt
fmt:
	@echo "$(BLUE)Formatting code...$(NC)"
	$(CARGO) fmt

.PHONY: fmt-check
fmt-check:
	@echo "$(BLUE)Checking code format...$(NC)"
	$(CARGO) fmt --check

.PHONY: clippy
clippy:
	@echo "$(BLUE)Running clippy...$(NC)"
	$(CARGO) clippy -- -D warnings

.PHONY: lint
lint: fmt-check clippy
	@echo "$(GREEN)All lint checks passed!$(NC)"

.PHONY: audit
audit:
	@echo "$(BLUE)Running security audit...$(NC)"
	$(CARGO) audit

# ============================================================================
# Documentation
# ============================================================================

.PHONY: doc
doc:
	@echo "$(BLUE)Building documentation...$(NC)"
	$(CARGO) doc --no-deps

.PHONY: doc-open
doc-open:
	@echo "$(BLUE)Building and opening documentation...$(NC)"
	$(CARGO) doc --no-deps --open

.PHONY: doc-api
doc-api:
	@echo "$(BLUE)Generating API documentation...$(NC)"
	$(CARGO) doc --no-deps --target $(TARGET_X86)
	$(CARGO) doc --no-deps --target $(TARGET_ARM)

# ============================================================================
# Running
# ============================================================================

.PHONY: run
run: build-x86
	@echo "$(BLUE)Running in QEMU...$(NC)"
	$(QEMU) -kernel $(TARGETDIR)/$(TARGET_X86)/release/nuva_kernel

.PHONY: run-debug
run-debug: debug
	@echo "$(BLUE)Running in QEMU with debug...$(NC)"
	$(QEMU) -kernel $(TARGETDIR)/$(TARGET_X86)/debug/nuva_kernel -s -S

.PHONY: run-arm
run-arm: build-arm
	@echo "$(BLUE)Running ARM64 in QEMU...$(NC)"
	qemu-system-aarch64 -kernel $(TARGETDIR)/$(TARGET_ARM)/release/nuva_kernel

.PHONY: run-riscv
run-riscv: build-riscv
	@echo "$(BLUE)Running RISC-V 64 in QEMU virt...$(NC)"
	qemu-system-riscv64 -machine virt -nographic -bios default -kernel $(TARGETDIR)/$(TARGET_RISCV)/release/nuva_kernel

# ============================================================================
# Examples
# ============================================================================

.PHONY: examples
examples: example-hal example-quantum example-npu

.PHONY: example-hal
example-hal:
	@echo "$(BLUE)Building HAL example...$(NC)"
	$(CC) -I hal/ffi/c_api examples/hal_basic.c -o $(BUILDDIR)/hal_basic -L $(BUILDDIR) -lnuva_hal

.PHONY: example-quantum
example-quantum:
	@echo "$(BLUE)Building quantum example...$(NC)"
	$(CC) -I hal/ffi/c_api examples/quantum_crypto.c -o $(BUILDDIR)/quantum_crypto -L $(BUILDDIR) -lnuva_hal

.PHONY: example-npu
example-npu:
	@echo "$(BLUE)Building NPU example...$(NC)"
	$(CXX) -I hal/ffi/cpp_api examples/npu_inference.cpp -o $(BUILDDIR)/npu_inference -L $(BUILDDIR) -lnuva_hal_cpp

# ============================================================================
# Benchmarks
# ============================================================================

.PHONY: bench
bench:
	@echo "$(BLUE)Running benchmarks...$(NC)"
	$(CARGO) bench

.PHONY: bench-ipc
bench-ipc:
	@echo "$(BLUE)Running IPC benchmarks...$(NC)"
	$(CARGO) bench --bench performance_bench

.PHONY: bench-quantum
bench-quantum:
	@echo "$(BLUE)Running quantum benchmarks...$(NC)"
	$(CARGO) bench --bench performance_bench

# ============================================================================
# Clean
# ============================================================================

.PHONY: clean
clean:
	@echo "$(RED)Cleaning...$(NC)"
	$(CARGO) clean
	rm -rf $(BUILDDIR)

.PHONY: clean-all
clean-all: clean
	@echo "$(RED)Deep cleaning...$(NC)"
	rm -rf $(TARGETDIR)
	rm -rf Cargo.lock

# ============================================================================
# Installation
# ============================================================================

.PHONY: install
install: build-x86
	@echo "$(BLUE)Installing...$(NC)"
	install -D -m 755 $(TARGETDIR)/$(TARGET_X86)/release/nuva_kernel /usr/local/bin/nuva_kernel
	install -D -m 644 hal/ffi/c_api/nuva_hal.h /usr/local/include/nuva_hal.h
	install -D -m 644 hal/ffi/cpp_api/nuva_hal.hpp /usr/local/include/nuva_hal.hpp

.PHONY: uninstall
uninstall:
	@echo "$(RED)Uninstalling...$(NC)"
	rm -f /usr/local/bin/nuva_kernel
	rm -f /usr/local/include/nuva_hal.h
	rm -f /usr/local/include/nuva_hal.hpp

# ============================================================================
# Package
# ============================================================================

.PHONY: package
package: build-all
	@echo "$(BLUE)Packaging...$(NC)"
	mkdir -p $(BUILDDIR)/package
	cp $(TARGETDIR)/$(TARGET_X86)/release/nuva_kernel $(BUILDDIR)/package/
	cp $(TARGETDIR)/$(TARGET_ARM)/release/nuva_kernel $(BUILDDIR)/package/nuva_kernel_arm64
	cp $(TARGETDIR)/$(TARGET_RISCV)/release/nuva_kernel $(BUILDDIR)/package/nuva_kernel_riscv64
	cp -r hal/ffi $(BUILDDIR)/package/
	cp -r docs $(BUILDDIR)/package/
	tar -czf $(BUILDDIR)/nuva-$(VERSION).tar.gz -C $(BUILDDIR)/package .

# ============================================================================
# Development
# ============================================================================

.PHONY: watch
watch:
	@echo "$(BLUE)Watching for changes...$(NC)"
	$(CARGO) watch -x "build --target $(TARGET_X86)"

.PHONY: check
check:
	@echo "$(BLUE)Checking...$(NC)"
	$(CARGO) check --target $(TARGET_X86)

.PHONY: expand
expand:
	@echo "$(BLUE)Expanding macros...$(NC)"
	$(CARGO) expand --target $(TARGET_X86)

# ============================================================================
# Help
# ============================================================================

.PHONY: help
help:
	@echo "$(GREEN)Nuva OS Build System$(NC)"
	@echo ""
	@echo "$(YELLOW)Build targets:$(NC)"
	@echo "  make build          - Build for x86_64"
	@echo "  make build-x86      - Build for x86_64"
	@echo "  make build-arm      - Build for ARM64"
	@echo "  make build-kirin    - Build for Kirin 9020"
	@echo "  make build-snapdragon - Build for Snapdragon 8 Gen 4"
	@echo "  make build-all      - Build all targets"
	@echo "  make build-riscv    - Build for RISC-V 64"
	@echo ""
	@echo "$(YELLOW)Testing:$(NC)"
	@echo "  make test           - Run tests"
	@echo "  make test-all       - Run all tests"
	@echo "  make coverage       - Generate coverage report"
	@echo ""
	@echo "$(YELLOW)Code quality:$(NC)"
	@echo "  make fmt            - Format code"
	@echo "  make lint           - Run lint checks"
	@echo "  make audit          - Security audit"
	@echo ""
	@echo "$(YELLOW)Documentation:$(NC)"
	@echo "  make doc            - Build documentation"
	@echo "  make doc-open       - Open documentation"
	@echo ""
	@echo "$(YELLOW)Running:$(NC)"
	@echo "  make run            - Run in QEMU"
	@echo "  make run-arm        - Run ARM64 in QEMU"
	@echo "  make run-riscv      - Run RISC-V 64 in QEMU virt"
	@echo ""
	@echo "$(YELLOW)Examples:$(NC)"
	@echo "  make examples       - Build all examples"
	@echo ""
	@echo "$(YELLOW)Other:$(NC)"
	@echo "  make clean          - Clean build"
	@echo "  make package        - Create package"
	@echo "  make help           - Show this help"

# ============================================================================
# Phony targets
# ============================================================================

.PHONY: all build build-x86 build-arm build-riscv build-kirin build-snapdragon build-all
.PHONY: dev debug
.PHONY: test test-all coverage
.PHONY: fmt fmt-check clippy lint audit
.PHONY: doc doc-open doc-api
.PHONY: run run-debug run-arm run-riscv
.PHONY: examples example-hal example-quantum example-npu
.PHONY: bench bench-ipc bench-quantum
.PHONY: clean clean-all
.PHONY: install uninstall
.PHONY: package
.PHONY: watch check expand
.PHONY: help

# Nuva OS Development Environment
#
# This Dockerfile provides a complete development environment for Nuva OS

FROM ubuntu:22.04

# Avoid interactive prompts
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    # Build essentials
    build-essential \
    cmake \
    make \
    ninja-build \
    # Compilers
    gcc \
    g++ \
    clang \
    lld \
    # Rust
    curl \
    # QEMU for testing
    qemu-system-x86 \
    qemu-system-arm \
    # Development tools
    git \
    vim \
    gdb \
    # Documentation
    doxygen \
    graphviz \
    # Python for scripts
    python3 \
    python3-pip \
    # Other tools
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Rust components
RUN rustup component add \
    rust-src \
    llvm-tools-preview \
    rustfmt \
    clippy

# Install Rust tools
RUN cargo install \
    cargo-xbuild \
    cargo-binutils \
    cargo-watch \
    cargo-tarpaulin \
    cargo-audit \
    cargo-outdated \
    cargo-bloat \
    cargo-flamegraph \
    criterion

# Set up working directory
WORKDIR /workspace

# Copy project files
COPY . .

# Build the project
RUN cargo build --target x86_64-unknown-none

# Set up environment variables
ENV NUVA_SDK=/workspace
ENV NUVA_HAL_INCLUDE=/workspace/hal/ffi/c_api
ENV NUVA_HAL_LIB=/workspace/target/x86_64-unknown-none/release

# Default command
CMD ["/bin/bash"]

# Contributing to Nuva OS

Thank you for your interest in contributing to Nuva OS! This document provides guidelines and instructions for contributing.

> Last updated: 2026-05-17

## Code of Conduct

Please be respectful and constructive in all interactions. We are committed to providing a friendly and inclusive experience for everyone.

## How to Contribute

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Use the Bug Report template
3. Provide detailed information:
   - System information
   - Steps to reproduce
   - Expected behavior vs. actual behavior
   - Logs and screenshots

### Suggesting Features

1. Check existing feature requests
2. Use the Feature Request template
3. Describe the feature and use case
4. Explain why it would be useful
5. Improve the core components
6. Chip manufacturer support
7. UI framework support

### Submitting Code

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Follow the coding standards
5. Write/update tests
6. Update documentation
7. Submit a Pull Request

## Development Environment Setup

### Prerequisites

```bash
# Install Rust toolchain
rustup override set nightly

# Install required components
rustup component add rust-src
rustup component add llvm-tools-preview

# Install tools
cargo install cargo-binutils
```

### Building

```bash
# Build for x86_64
cargo build --target x86_64-unknown-none

# Build for ARM64
cargo build --target aarch64-unknown-none

# Build with features
cargo build --features kirin9020
```

### Testing

```bash
# Run unit tests
cargo test

# Run specific tests
cargo test --test plugin_tests

# Run with coverage
cargo tarpaulin
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy

# Run linter with all warnings
cargo clippy -- -W clippy::all
```

## Coding Standards

### Rust Code

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Kernel code uses `#![no_std]`
- Document all public APIs with doc comments
- Use meaningful variable names
- Keep functions short and focused
- Prefer functional style over imperative

### Nuva Language Code (.nv files)

- All Nuva source files must use the `.nv` extension
- Use the **declarative paradigm** — no imperative UI/concurrency/resource patterns
- Declare UI components with `component`, not `new Widget()` or `build()`
- Use `signal` for reactive state, not `setState()` or manual notification
- Use `effect` for reactive side effects with automatic dependency tracking
- Use `async`/`await` for concurrency, not callbacks or `Thread.start()`
- Use `resource`/`with` for resource management, not manual acquire/release
- Code comments must be in English (same as Rust code)
- Component names use `PascalCase`, signal names use `snake_case`
- See [docs/NUVA_LANG.md](docs/NUVA_LANG.md) for the full language reference

### C/C++ Code

- Follow C99/C++14 standards
- Use consistent naming conventions
- Document all public APIs
- Handle all error conditions
- Avoid memory leaks

### Documentation

- Use clear and concise language
- Provide examples where appropriate
- Keep documentation up to date
- Use proper Markdown formatting
- English documentation uses `.md` extension
- Chinese documentation uses `_zh.md` extension

## Project Structure

```
nuva/
├── kernel/           # Kernel implementation
│   ├── arch/        # Architecture-specific (arm64, loongarch64, x64)
│   ├── mm/          # Memory management
│   ├── process/     # Process management
│   ├── sched/       # Scheduler
│   ├── fs/          # File system
│   ├── ipc/         # IPC subsystem
│   ├── driver/      # Driver framework
│   └── security/    # Security subsystem
├── hal/              # Hardware Abstraction Layer
│   ├── cpu/         # CPU abstraction
│   ├── gpu/         # GPU abstraction
│   ├── npu/         # NPU abstraction
│   └── quantum/     # Quantum cryptography
├── lib/              # Core libraries
│   ├── brain/       # AI engine
│   ├── lang/        # Nuva language compiler & runtime
│   ├── net/         # Network library
│   └── ml/          # Machine learning
├── application/      # Application framework
├── services/         # System services
├── fs/               # File system implementations
├── tools/            # Development tools
├── sdk/              # Software Development Kit
├── docs/             # Documentation
│   ├── ARCHITECTURE.md
│   ├── CODING_STANDARD.md
│   ├── NUVA_LANG.md        # Nuva language reference (English)
│   └── NUVA_LANG_zh.md     # Nuva language reference (Chinese)
├── tests/            # Test suites
└── examples/         # Example code
```

### Nuva Language Source Files (.nv)

Nuva application source code uses the `.nv` extension and follows the declarative paradigm:

```
app/
├── main.nv              # Application entry point
├── components/          # Declarative UI components
│   ├── header.nv
│   ├── footer.nv
│   └── sidebar.nv
├── services/            # Async services
│   └── api.nv
└── styles/              # Styling and themes
    └── theme.nv
```

## Commit Convention

### Commit Messages

Format:
```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code refactoring
- `test`: Tests
- `chore`: Maintenance

Example:
```
feat(quantum): add Kyber-1024 support

Implement Kyber-1024 variant for post-quantum
key encapsulation.

Closes #123
```

## Pull Request Process

1. **Create a branch**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make changes**
   - Write code
   - Add tests
   - Update documentation

3. **Test changes**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **Commit changes**
   ```bash
   git add .
   git commit -m "feat: my feature"
   ```

5. **Push changes**
   ```bash
   git push origin feature/my-feature
   ```

6. **Create PR**
   - Use the PR template
   - Link related issues
   - Request review

7. **Address feedback**
   - Make changes as requested
   - Push new commits
   - Mark conversations as resolved

8. **Merge**
   - Squash and merge
   - Delete branch

## Review Process

All PRs require:
- At least 1 approval
- All CI checks passing
- No unresolved conversations

Reviewers check for:
- Code correctness
- Test coverage
- Documentation
- Performance impact
- Security impact

## Release Process

1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Create a release PR
4. Tag the release: `git tag v1.0.0`
5. Push the tag: `git push --tags`
6. CI creates release artifacts

## Getting Help

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/nuva-os/nuva/issues)
- **Discussions**: [GitHub Discussions](https://github.com/nuva-os/nuva/discussions)
- **Email**: zhangyujie_china@163.com

## Acknowledgments

Contributors are recognized in:
- CONTRIBUTORS file
- Release notes
- Project website

Thank you for contributing to Nuva OS! 🎉

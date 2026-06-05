# 为 Nuva OS 贡献

感谢您对为 Nuva OS 贡献的关注！本文档提供了贡献的指南和说明。

> 最后更新：2026-05-30

## 行为准则

请在所有互动中保持尊重和建设性。我们致力于为每个人提供友好和包容的体验。

## 如何贡献

### 报告 Bug

1. 检查现有 issue 以避免重复
2. 使用 Bug 报告模板
3. 提供详细信息：
   - 系统信息
   - 复现步骤
   - 期望行为与实际行为
   - 日志和截图

### 建议功能

1. 检查现有功能请求
2. 使用功能请求模板
3. 描述功能和使用场景
4. 说明为什么它会有用
5. 完善内核部分
6. 芯片厂商支持
7. UI框架支持

### 提交代码

1. Fork 仓库
2. 创建特性分支
3. 进行修改
4. 遵循编码规范
5. 编写/更新测试
6. 更新文档
7. 提交 Pull Request

## 开发环境设置

### 前置条件

```bash
# Install Rust toolchain
rustup override set nightly

# Install required components
rustup component add rust-src
rustup component add llvm-tools-preview

# Install tools
cargo install cargo-binutils
```

### 构建

```bash
# Build for x86_64
cargo build --target x86_64-unknown-none

# Build for ARM64
cargo build --target aarch64-unknown-none

# Build for RISC-V 64
cargo build --target riscv64-unknown-none --features riscv64

# Build with features
cargo build --features kirin9020
```

### 测试

```bash
# Run unit tests
cargo test

# Run specific tests
cargo test --test plugin_tests

# Run with coverage
cargo tarpaulin
```

### 代码质量

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

## 编码规范

### Rust 代码

- 遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- kernel 代码使用 `#![no_std]`
- 使用文档注释记录所有公共 API
- 使用有意义的变量名
- 保持函数短小且专注
- 优先使用函数式风格而非命令式

### C/C++ 代码

- 遵循 C99/C++14 标准
- 使用一致的命名约定
- 记录所有公共 API
- 处理所有错误情况
- 避免内存泄漏

### Nuva 语言代码 (.nv 文件)

- 所有 Nuva 源文件必须使用 `.nv` 扩展名
- 使用**声明式范式** — 禁止命令式 UI/并发/资源模式
- 使用 `component` 声明 UI 组件，而非 `new Widget()` 或 `build()`
- 使用 `signal` 实现响应式状态，而非 `setState()` 或手动通知
- 使用 `effect` 实现带自动依赖追踪的响应式副作用
- 使用 `async`/`await` 实现并发，而非回调或 `Thread.start()`
- 使用 `resource`/`with` 管理资源，而非手动 acquire/release
- 代码注释必须使用英文（与 Rust 代码一致）
- 组件名使用 `PascalCase`，信号名使用 `snake_case`
- 详见 [docs/NUVA_LANG_zh.md](docs/NUVA_LANG_zh.md) 获取完整语言参考

### 文档

- 使用清晰简洁的语言
- 在适当处提供示例
- 保持文档最新
- 使用正确的 Markdown 格式
- 英文文档使用 `.md` 扩展名
- 中文文档使用 `_zh.md` 扩展名

## 项目结构

```
nuva/
├── kernel/           # Kernel implementation
│   ├── arch/        # Architecture-specific (arm64, loongarch64, riscv64, x64)
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
├── syslib/           # 系统库
│   ├── brain/       # AI engine
│   ├── lang/        # Nuva language
│   ├── net/         # Network library
│   └── ml/          # Machine learning
├── application/      # Application framework
├── services/         # System services
├── fs/               # File system implementations
├── tools/            # Development tools
├── sdk/              # Software Development Kit
├── docs/             # 文档
│   ├── ARCHITECTURE.md
│   ├── CODING_STANDARD.md
│   ├── NUVA_LANG.md        # Nuva 语言参考（英文）
│   └── NUVA_LANG_zh.md     # Nuva 语言参考（中文）
├── tests/            # 测试套件
└── examples/         # 示例代码
```

### Nuva 语言源文件 (.nv)

Nuva 应用源代码使用 `.nv` 扩展名，遵循声明式范式：

```
app/
├── main.nv              # 应用程序入口
├── components/          # 声明式 UI 组件
│   ├── header.nv
│   ├── footer.nv
│   └── sidebar.nv
├── services/            # 异步服务
│   └── api.nv
└── styles/              # 样式和主题
    └── theme.nv
```

## 提交规范

### 提交信息

格式：
```
<type>(<scope>): <subject>

<body>

<footer>
```

类型：
- `feat`：新功能
- `fix`：Bug 修复
- `docs`：文档
- `style`：格式化
- `refactor`：代码重构
- `test`：测试
- `chore`：维护

示例：
```
feat(quantum): add Kyber-1024 support

Implement Kyber-1024 variant for post-quantum
key encapsulation.

Closes #123
```

## Pull Request 流程

1. **创建分支**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **进行修改**
   - 编写代码
   - 添加测试
   - 更新文档

3. **测试修改**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **提交修改**
   ```bash
   git add .
   git commit -m "feat: my feature"
   ```

5. **推送修改**
   ```bash
   git push origin feature/my-feature
   ```

6. **创建 PR**
   - 使用 PR 模板
   - 关联相关 issue
   - 请求评审

7. **处理反馈**
   - 按要求进行修改
   - 推送新提交
   - 标记对话已解决

8. **合并**
   - Squash and merge
   - 删除分支

## 评审流程

所有 PR 需要：
- 至少 1 个批准
- 所有 CI 检查通过
- 无未解决的对话

评审者检查：
- 代码正确性
- 测试覆盖率
- 文档
- 性能影响
- 安全影响

## 发布流程

1. 更新 Cargo.toml 中的版本
2. 更新 CHANGELOG.md
3. 创建发布 PR
4. 打标签：`git tag v1.0.0`
5. 推送标签：`git push --tags`
6. CI 创建发布制品

## 获取帮助

- **文档**：[docs/](docs/)
- **Issue**：[GitHub Issues](https://github.com/nuva-os/nuva/issues)
- **讨论**：[GitHub Discussions](https://github.com/nuva-os/nuva/discussions)
- **邮箱**：kellen9903@gmail.com

## 致谢

贡献者在以下位置获得认可：
- CONTRIBUTORS 文件
- 发布说明
- 项目网站

感谢您为 Nuva OS 贡献！🎉

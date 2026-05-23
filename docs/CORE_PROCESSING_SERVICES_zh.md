# Nuva OS 核心处理服务

## 架构概述

Nuva OS在L3（系统服务）层提供六个核心处理服务，全部以微内核服务进程形式运行，通过**Nuva IPC**（原生IPC机制）对外提供能力。此设计与宏内核框架集成方式根本不同，确保服务隔离、故障不传播、独立生命周期管理。

### 服务层位置

```
L4 - 应用框架
L3 - 系统服务  <-- 核心处理服务（opengl, web, sqlite, video, audio, image）
L2 - 系统库
L1 - 内核（微内核）
L0 - 硬件抽象
```

### 服务注册表

| 服务 | Nuva IPC 名称 | 描述 |
|------|---------------|------|
| OpenGL | `nuva.service.opengl` | GPU加速图形渲染 |
| Web | `nuva.service.web` | Web引擎与JS沙箱 |
| SQLite | `nuva.service.sqlite` | 加密嵌入式数据库 |
| Video | `nuva.service.video` | 视频编解码与硬件/软件降级 |
| Audio | `nuva.service.audio` | 音频编解码与多流混音 |
| Image | `nuva.service.image` | 图像编解码与变换管线 |

## 共享框架（`core_processing/`）

六个服务共享通用框架，提供：

- **服务节点注册**（`CoreProcessingService` trait）- 统一Nuva IPC服务生命周期
- **零拷贝传输**（`ShmTransferManager`）- 共享内存区域用于大数据传递
- **硬件加速**（`HwAccelManager`）- `execute_with_fallback()`自动软件降级
- **功耗协同**（`PowerCoordManager`）- 向`nuva.service.power`报告Idle/Active/Burst状态
- **权限校验** - 每次Nuva IPC调用携带CallerIdentity（PID/UID）
- **格式探测** - Magic bytes匹配自动识别媒体格式
- **错误模型** - `ServiceError` + `ServiceSpecificError`统一错误类型

## 服务详情

### OpenGL图形渲染服务

**模块**：`services/opengl/`

通过Nuva IPC提供OpenGL ES 3.2子集渲染。核心特性：

- **渲染上下文**：每调用方独立隔离上下文，拥有独立GPU命令缓冲
- **GPU命令编码**：`GlCommand`枚举 → `GpuCommandBuffer` → `dyn GpuDevice`提交
- **资源管理**：GPU内存超限时LRU淘汰策略
- **栅栏同步**：帧级GPU/CPU同步
- **软件降级**：`GpuDevice::initialize()`失败时自动切换

**性能目标**：GPU命令提交端到端延迟 ≤ 2ms

### SQLite嵌入式数据库服务

**模块**：`services/sqlite/`

带WAL持久化的完整嵌入式SQL数据库。核心特性：

- **SQL解析器**：递归下降解析器，支持DDL/DML/事务/索引
- **B-Tree存储**：表和索引存储，支持页面分裂/合并
- **WAL持久化**：仅追加写前日志，崩溃恢复和自动checkpoint
- **ACID事务**：BEGIN/COMMIT/ROLLBACK，并发写冲突处理
- **加密**：AES-256-XTS页面级加密
- **连接池**：有界并发连接，可配置最大连接数

**性能目标**：10万行数据集单条SQL查询 ≤ 5ms（索引命中）

### Web引擎服务

**模块**：`services/web/`

带安全隔离的Web页面渲染引擎。核心特性：

- **HTML5/CSS3解析器**：DOM树构建与样式级联
- **布局引擎**：Box模型、Flexbox和Grid布局计算
- **JS沙箱**：隔离执行，堆预算和超时，禁止直接syscall
- **安全**：同源策略、CORS白名单、安全上下文HTTPS强制
- **页面管线**：Fetching→Parsing→Styling→Layout→ScriptExecution→Rendering→Loaded状态机
- **HTTP缓存**：NuvaFS后端缓存，支持再验证

**性能目标**：首屏加载 ≤ 3s（本地缓存命中）

### 视频编解码服务

**模块**：`services/video/`

带硬件加速的视频编解码。核心特性：

- **格式**：H.264/AVC、H.265/HEVC、VP9、AV1
- **编解码器注册表**：优先选择硬件编解码器
- **硬件加速**：通过`dyn GpuDevice`/`dyn NpuHal`提交GPU/NPU命令
- **软件降级**：100ms内自动降级
- **帧缓冲**：共享内存零拷贝帧传输

**性能目标**：1080p@30fps ≥ 35fps（硬件路径）

### 音频编解码服务

**模块**：`services/audio/`

带多流混音的音频处理。核心特性：

- **格式**：AAC-LC、Opus、FLAC、PCM
- **重采样**：Linear和Sinc插值采样率转换
- **多流混音**：N通道混合，硬限幅溢出保护
- **音量控制**：按流Q16.16定点增益，原子更新

**性能目标**：48kHz实时处理延迟 ≤ 10ms

### 图像编解码服务

**模块**：`services/image/`

带变换管线的图像编解码。核心特性：

- **格式**：JPEG（基线+渐进式）、PNG（Adam7隔行）、WebP（VP8/VP8L）、BMP、GIF（多帧+LZW）
- **变换管线**：Scale/Rotate/Crop/ColorSpaceConvert，Nearest/Bilinear/Lanczos3滤波
- **渐进式解码**：多遍渐进JPEG/PNG解码
- **硬件加速**：通过`dyn GpuDevice` GPU加速解码

**性能目标**：4K JPEG解码 ≤ 50ms（硬件路径）

## 初始化

服务在`init_core_processing_services()`中按依赖顺序初始化：

```
阶段1（并行）：opengl, sqlite, audio, image, video
阶段2（顺序）：web（依赖net服务）
```

所有核心处理服务要求IPC服务先初始化。

## 错误处理

所有服务使用`Result<T, ServiceError>`返回类型。生产路径无`panic!`/`unwrap()`/`expect()`。硬件故障触发自动软件降级并记录警告日志。

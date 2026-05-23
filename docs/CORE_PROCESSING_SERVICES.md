# Nuva OS Core Processing Services

## Architecture Overview

Nuva OS provides six core processing services at the L3 (Services) layer, all implemented as microkernel service processes communicating via **Nuva IPC** (the native IPC mechanism). This design differs fundamentally from monolithic kernel frameworks by ensuring service isolation, fault containment, and independent lifecycle management.

### Service Layer Position

```
L4 - Application Framework
L3 - System Services  <-- Core Processing Services (opengl, web, sqlite, video, audio, image)
L2 - System Libraries
L1 - Kernel (microkernel)
L0 - Hardware Abstraction
```

### Service Registry

| Service | Nuva IPC Name | Description |
|---------|---------------|-------------|
| OpenGL | `nuva.service.opengl` | GPU-accelerated graphics rendering |
| Web | `nuva.service.web` | Web engine with JS sandbox |
| SQLite | `nuva.service.sqlite` | Embedded database with encryption |
| Video | `nuva.service.video` | Video codec with HW/SW fallback |
| Audio | `nuva.service.audio` | Audio codec with multi-stream mixing |
| Image | `nuva.service.image` | Image codec with transform pipeline |

## Shared Framework (`core_processing/`)

All six services share a common framework providing:

- **Service Node Registration** (`CoreProcessingService` trait) - Unified Nuva IPC service lifecycle
- **Zero-Copy Transfer** (`ShmTransferManager`) - Shared memory regions for large data
- **Hardware Acceleration** (`HwAccelManager`) - `execute_with_fallback()` with automatic SW degradation
- **Power Coordination** (`PowerCoordManager`) - Reports Idle/Active/Burst to `nuva.service.power`
- **Permission Verification** - CallerIdentity (PID/UID) carried on every Nuva IPC call
- **Format Detection** - Magic bytes matching for media format auto-detection
- **Error Model** - `ServiceError` + `ServiceSpecificError` unified error types

## Service Details

### OpenGL Graphics Rendering Service

**Module**: `services/opengl/`

Provides OpenGL ES 3.2 subset rendering via Nuva IPC. Key features:

- **Render Contexts**: Per-caller isolated contexts with independent GPU command buffers
- **GPU Command Encoding**: `GlCommand` enum → `GpuCommandBuffer` → `dyn GpuDevice` submission
- **Resource Management**: LRU eviction when GPU memory is over limit
- **Fence Synchronization**: Frame-level GPU/CPU synchronization
- **Software Fallback**: Automatic degradation when `GpuDevice::initialize()` fails

**Performance Target**: GPU command submission E2E latency ≤ 2ms

### SQLite Embedded Database Service

**Module**: `services/sqlite/`

Full-featured embedded SQL database with WAL persistence. Key features:

- **SQL Parser**: Recursive descent parser for DDL/DML/transactions/indexes
- **B-Tree Storage**: Table and index storage with page split/merge
- **WAL Persistence**: Append-only write-ahead log with crash recovery and auto-checkpoint
- **ACID Transactions**: BEGIN/COMMIT/ROLLBACK with concurrent write conflict handling
- **Encryption**: AES-256-XTS per-page encryption for sensitive databases
- **Connection Pool**: Bounded concurrent connections with configurable limits

**Performance Target**: Single SQL query on 100K-row dataset ≤ 5ms (index hit)

### Web Engine Service

**Module**: `services/web/`

Web page rendering engine with security isolation. Key features:

- **HTML5/CSS3 Parsers**: DOM tree construction with style cascade
- **Layout Engine**: Box model, Flexbox, and Grid layout computation
- **JS Sandbox**: Isolated execution with heap budget and timeout, no direct syscall access
- **Security**: Same-origin policy enforcement, CORS whitelist, HTTPS mandatory in secure contexts
- **Page Pipeline**: Fetching→Parsing→Styling→Layout→ScriptExecution→Rendering→Loaded state machine
- **HTTP Cache**: NuvaFS-backed cache with revalidation support

**Performance Target**: First paint ≤ 3s (local cache hit)

### Video Codec Service

**Module**: `services/video/`

Video encode/decode with hardware acceleration. Key features:

- **Formats**: H.264/AVC, H.265/HEVC, VP9, AV1
- **Codec Registry**: Priority selection of hardware codecs
- **HW Acceleration**: GPU/NPU acceleration via `dyn GpuDevice`/`dyn NpuHal`
- **Software Fallback**: Automatic degradation within 100ms
- **Frame Buffer**: Zero-copy shared memory frame transfer

**Performance Target**: 1080p@30fps ≥ 35fps (hardware path)

### Audio Codec Service

**Module**: `services/audio/`

Audio processing with multi-stream mixing. Key features:

- **Formats**: AAC-LC, Opus, FLAC, PCM
- **Resampler**: Linear and Sinc interpolation for sample rate conversion
- **Multi-stream Mixer**: N-channel mixing with hard clipping overflow protection
- **Volume Control**: Per-stream Q16.16 fixed-point gain with atomic updates

**Performance Target**: 48kHz real-time processing latency ≤ 10ms

### Image Codec Service

**Module**: `services/image/`

Image encode/decode with transform pipeline. Key features:

- **Formats**: JPEG (baseline + progressive), PNG (Adam7 interlace), WebP (VP8/VP8L), BMP, GIF (multi-frame + LZW)
- **Transform Pipeline**: Scale/Rotate/Crop/ColorSpaceConvert with Nearest/Bilinear/Lanczos3 filters
- **Progressive Decode**: Multi-pass progressive JPEG/PNG decoding
- **HW Acceleration**: GPU-accelerated decode via `dyn GpuDevice`

**Performance Target**: 4K JPEG decode ≤ 50ms (hardware path)

## Initialization

Services are initialized in dependency order within `init_core_processing_services()`:

```
Phase 1 (parallel): opengl, sqlite, audio, image, video
Phase 2 (sequential): web (depends on net service)
```

All core processing services require IPC service to be initialized first.

## Usage Example

```rust
use crate::services::opengl::service_node::OpenGLService;
use crate::services::core_processing::service_node::{CoreProcessingService, CallerIdentity};

// Service is initialized automatically during kernel boot
// Access via Nuva IPC from application layer

let caller = CallerIdentity::new(pid, uid);
// Send request via Nuva IPC to nuva.service.opengl
```

## Error Handling

All services use `Result<T, ServiceError>` return types. No `panic!`/`unwrap()`/`expect()` in production paths. Hardware failures trigger automatic software fallback with warning logs.

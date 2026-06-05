# Services — 系统服务层 (L3)

## 概述

Services 层（Layer 3）提供操作系统级的系统服务，在微内核架构中运行于用户空间。包括应用管理、IPC、网络、电源、安全等核心服务。

## 子模块

| 子模块 | 说明 |
|--------|------|
| app/ | 应用管理服务（安装、生命周期、包管理、屏幕管理） |
| ipc/ | IPC 服务（Binder、Channel、共享内存） |
| net/ | 网络服务（DNS、TCP/UDP、接口管理） |
| power/ | 电源管理服务（PM 策略、唤醒锁、挂起） |
| security/ | 安全服务（Gatekeeper、Keymaster、权限管理、TEE 客户端） |
| form_factor/ | 形态因子适配服务（手机/平板/PC 检测） |
| audio/ | 音频服务（编解码器、混音器、重采样、音量、电源管理） |
| video/ | 视频服务（H.264/H.265/VP9/AV1 编解码、硬件加速、帧缓冲） |
| web/ | Web 服务（HTML/CSS 解析器、JS 引擎、DOM、布局、页面渲染） |
| opengl/ | OpenGL 服务（GPU 渲染、管线、资源管理、软件回退） |
| sqlite/ | SQLite 数据库服务（B 树、WAL、连接池、查询执行器） |
| image/ | 图像服务（JPEG/PNG/GIF/WebP/BMP 编解码、变换、硬件加速） |
| core_processing/ | 核心处理服务（格式检测、共享内存传输、电源协调） |

## 依赖关系

- **下层依赖**：hal (L0)、kernel (L1)、syslib (L2)
- **上层被依赖**：application (L4)

## 构建配置

服务层随内核一起编译，通过 feature flags 控制服务包含：
- `mobile` profile：为手机/平板部署启用全部 13 项核心服务
- `server` profile：启用 network、security、sqlite 和 web 服务
- 服务通过 Nuva IPC 端口通信，采用 capability-gated 访问控制

## 公开接口

提供以下服务接口：`AppService`、`IpcService`、`NetService`、`PowerService`、`SecurityService`、`AudioService`、`VideoService`、`WebService`、`OpenGLService`、`SqliteService`、`ImageService`、`CoreProcessingService`、`FormFactorService`。

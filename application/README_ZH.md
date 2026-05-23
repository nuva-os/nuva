# Application — 应用框架层 (L4)

## 概述

Application 层（Layer 4）是 Nuva OS 的最上层，提供面向应用的 UI 框架和渲染引擎。为应用开发者提供窗口管理、事件处理、UI 组件、资源管理等接口。

## 子模块

| 子模块 | 说明 |
|--------|------|
| ui/ | UI 框架：自适应布局、布局引擎、样式系统、组件库 |
| window/ | 窗口管理：管理器、窗口、Surface |
| event/ | 事件系统：分发器、事件类型、处理器 |
| render/ | 渲染引擎：合成器、渲染上下文、画笔 |
| resource/ | 资源管理：缓存、加载器、解码器（JPEG/PNG/TTF/WAV） |

## 依赖关系

- **下层依赖**：hal (L0)、kernel (L1)、syslib (L2)、services (L3)
- **上层被依赖**：无（最顶层）

## 构建配置

应用框架层随内核一起编译，形态因子通过 `services::form_factor` 自适应：
- 手机/平板：触摸优化 UI、紧凑布局
- 桌面：窗口化 UI、多显示器支持
- 服务器：无 UI，仅 API 接口

## 公开接口

- `Application` — 应用入口和生命周期
- `Window` — 窗口创建和管理
- `View` — UI 组件基类
- `EventDispatcher` — 事件分发和处理
- `Renderer` — 渲染上下文和绘制

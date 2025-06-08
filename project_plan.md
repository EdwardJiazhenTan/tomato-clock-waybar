# Tomato Clock Project Plan

## Completed Features

- [x] Basic timer functionality
- [x] Start, stop, pause, resume commands
- [x] Status management
- [x] Waybar integration with JSON output
- [x] Daemon mode for background operation
- [x] State persistence
- [x] Command-line interface
- [x] Configuration file
- [x] Multiple workflow support
- [x] Socket-based communication for improved reliability
- [x] Systemd service integration

## Current Focus

- [x] Robust error handling for Waybar integration
- [x] Fix "Failed to send event" errors with socket communication
- [x] Improve documentation and examples
- [x] Create user-friendly installation process

## Future Development

### Short-term Goals

- [ ] Desktop notifications with customizable sounds
- [ ] Improved statistics tracking
- [ ] More detailed timer information
- [ ] Support for different time formats

### Medium-term Goals

- [ ] Web interface for remote control
- [ ] API for third-party integrations
- [ ] Mobile app companion
- [ ] Advanced workflow scheduling

### Long-term Goals

- [ ] Task management integration
- [ ] Time tracking and reporting
- [ ] Team synchronization features
- [ ] Cloud backup and sync

## Technical Improvements

- [x] Socket-based communication architecture
- [x] Systemd service management
- [ ] Comprehensive test suite
- [ ] Performance optimizations
- [ ] Plugin system

## Integration Plans

- [x] Waybar (primary focus)
- [x] Socket client-server communication
- [ ] i3status / i3blocks
- [ ] Polybar
- [ ] GNOME Shell extension
- [ ] KDE Plasma widget

## Notes

The project has evolved from a simple Pomodoro timer to a more comprehensive time management tool. The socket-based communication system has significantly improved reliability when integrating with Waybar, resolving the "Failed to send event" errors that were previously encountered.

Socket architecture provides several benefits:

- More reliable communication between components
- Better error handling and recovery
- Centralized state management
- Simplified client implementation
- Reduced file I/O overhead

Next major focus will be on the notification system as outlined in notification_plan.md.

## 项目概述

一个基于 Rust 的番茄钟（Pomodoro 计时器）应用程序，与 Linux 系统上的 Waybar 集成。该应用程序允许用户管理不同的工作/休息循环，并在 Waybar 界面中显示当前计时器状态。

## 核心组件

### 1. Rust 后端 (tomato-clock-core)

- [x] 计时器管理系统
- [x] 状态跟踪（工作、学习、休息等）
- [x] 工作流定义（例如，工作 30 分钟，休息 5 分钟，工作 25 分钟）
- [x] 状态持久化
- [x] CLI 界面

### 2. Waybar 集成

- [x] Waybar 的自定义模块
- [x] Waybar 使用的 JSON 输出格式
- [x] 点击交互以控制计时器

## 功能需求

### 计时器功能

- [x] **状态管理**：设置和跟踪不同的状态（工作、学习、休息等）
- [x] **工作流配置**：定义具有多个活动和持续时间的自定义工作流
- [x] **计时器控制**：启动、暂停、恢复和停止计时器
- [ ] **通知**：状态变化和完成间隔的系统通知

### Waybar 集成

- [x] 显示当前计时器状态、剩余时间和当前活动
- [x] 直接从 Waybar 控制计时器（启动/停止/跳过）
- [x] 不同状态的视觉指示器
- [x] 带有附加信息的工具提示

## 技术架构

### 后端实现

1. **核心计时器逻辑**

   - [x] 用于 Timer、Status、Workflow 的 Rust 结构体
   - [x] 线程安全的状态管理
   - [x] 跟踪经过时间的系统

2. **配置管理**

   - [x] TOML 配置文件
   - [x] 用户定义的工作流和首选项

3. **持久层**

   - [x] 在会话之间保存和恢复计时器状态
   - [x] 记录已完成的会话

4. **IPC 机制**
   - [x] 基于文件的与 Waybar 的通信

### Waybar 集成

1. **自定义模块**

   - [x] Waybar 模块的脚本执行模式
   - [x] 输出的格式规范

2. **交互处理**
   - [x] 点击事件映射到计时器命令

## 已实现和未来计划

### 已实现功能

- **核心计时器功能**

  - [x] 基本计时器机制
  - [x] 状态和工作流数据结构
  - [x] 用于手动测试的 CLI 接口

- **状态管理和持久化**

  - [x] 配置文件解析
  - [x] 会话之间的状态持久化
  - [x] 基本日志功能

- **Waybar 集成**
  - [x] Waybar 模块格式
  - [x] 计时器和 Waybar 之间的基于文件的 IPC
  - [x] 交互处理程序

### 计划中的功能

- **通知系统**

  - [ ] 桌面通知集成
  - [ ] 自定义通知设置
  - [ ] 声音通知选项

- **高级工作流管理**

  - [ ] 工作流统计和报告
  - [ ] 工作流模板和导入/导出
  - [ ] 可视化工作流编辑器

- **用户界面增强**

  - [ ] 改进的 Waybar 视觉风格
  - [ ] 自定义主题支持
  - [ ] 交互式统计显示
  - [ ] 悬停时显示设置选项

- **系统集成**

  - [ ] 系统级"勿扰"模式集成
  - [ ] 系统睡眠/休眠处理
  - [ ] 自动启动配置

- **性能优化**
  - [ ] 减少资源使用
  - [ ] 改进多线程处理
  - [ ] 内存占用优化

## 目录结构

```
tomato-clock-waybar/
├── src/
│   ├── main.rs              # 应用程序入口点
│   ├── timer.rs             # 计时器实现
│   ├── status.rs            # 状态管理
│   ├── workflow.rs          # 工作流定义和管理
│   ├── config.rs            # 配置处理
│   ├── persistence.rs       # 状态持久化
│   └── waybar.rs            # Waybar集成
├── config/
│   └── default_config.toml  # 默认配置
├── scripts/
│   ├── waybar-module.sh     # Waybar模块脚本
│   ├── toggle.sh            # 计时器控制脚本
│   └── install.sh           # 安装脚本
├── Cargo.toml               # Rust项目配置
└── README.md                # 项目文档
```

## 技术

- **Rust**: 核心应用程序
- **serde**: 序列化/反序列化
- **clap**: 命令行参数解析
- **tokio**: 异步运行时
- **notify-rust**: 桌面通知（计划中）
- **Waybar**: 状态栏集成

## 部署

应用程序将作为二进制文件安装在用户路径中，Waybar 模块将在 Waybar 配置文件中配置。应用程序将作为后台服务运行。

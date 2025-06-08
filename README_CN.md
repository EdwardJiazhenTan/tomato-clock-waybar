# Tomato Clock for Waybar

一个设计用于与 Linux 系统上的[Waybar](https://github.com/Alexays/Waybar)集成的简单番茄钟计时器。

## 功能

- 经典番茄工作法计时器
- 无缝 Waybar 集成
- 可配置的工作和休息时长
- 重启后状态持久化
- 多种通知方式
- 基于 Socket 的通信（提高可靠性）

## 安装

### 前提条件

- Rust 工具链（通过[rustup](https://rustup.rs/)安装）
- Waybar

### 从源代码构建

1. 克隆仓库：

   ```
   git clone https://github.com/yourusername/tomato-clock-waybar.git
   cd tomato-clock-waybar
   ```

2. 构建项目：

   ```
   cargo build --release
   ```

3. 安装二进制文件：

   ```
   cp target/release/tomato-clock ~/.local/bin/
   ```

4. 复制 Waybar 模块脚本：
   ```
   mkdir -p ~/.config/waybar/scripts
   cp scripts/waybar-module.sh ~/.config/waybar/scripts/
   cp scripts/toggle.sh ~/.config/waybar/scripts/
   chmod +x ~/.config/waybar/scripts/waybar-module.sh
   chmod +x ~/.config/waybar/scripts/toggle.sh
   ```

### 使用安装脚本

或者，您可以使用包含的安装脚本：

```
./scripts/install.sh
```

这将：

1. 构建 tomato-clock 二进制文件
2. 将其安装到 ~/.local/bin/
3. 将 Waybar 模块脚本复制到 ~/.config/waybar/scripts/
4. 创建必要的配置目录

## 使用方法

### 基本命令

```bash
# 启动计时器
tomato-clock start

# 停止计时器
tomato-clock stop

# 暂停计时器
tomato-clock pause

# 恢复计时器
tomato-clock resume

# 跳过当前阶段
tomato-clock skip

# 显示计时器信息
tomato-clock info

# 运行守护进程（Waybar集成所需）
tomato-clock daemon
```

### 与 Waybar 集成

有两种方法可以与 Waybar 集成：

#### 方法 1：直接集成（简单）

在 Waybar 配置中添加：

```json
"custom/tomato": {
    "exec": "cat ~/.config/tomato-clock/waybar-output.json",
    "return-type": "json",
    "interval": 1,
    "on-click": "~/.local/bin/tomato-clock start",
    "on-click-middle": "~/.local/bin/tomato-clock stop",
    "on-click-right": "~/.local/bin/tomato-clock skip"
}
```

#### 方法 2：基于 Socket 的集成（推荐）

为了提高可靠性并修复"Failed to send xxx event"错误，使用基于 Socket 的集成：

1. 安装 Socket 服务器：

```bash
cd sockets
chmod +x install_socket_server.sh
./install_socket_server.sh
```

2. 这将自动：
   - 安装所需脚本
   - 配置 Waybar
   - 创建 systemd 服务以自动启动 Socket 服务器
   - 应用必要的样式

有关基于 Socket 的集成的更多详细信息，请参阅`sockets/README.md`。

### Waybar 样式

将这些样式添加到您的 Waybar CSS 中：

```css
#custom-tomato {
  padding: 0 0.6em;
  margin: 0 5px;
  border-radius: 5px;
  background-color: #2d3436;
  color: #e84393;
}

#custom-tomato.running {
  background-color: #55efc4;
  color: #2d3436;
  font-weight: bold;
}

#custom-tomato.paused {
  background-color: #ffeaa7;
  color: #2d3436;
}

#custom-tomato.completed {
  background-color: #74b9ff;
  color: #2d3436;
}
```

## 配置

在`~/.config/tomato-clock/config.toml`创建配置文件：

```toml
[timer]
work_duration = 25    # 工作时长（分钟）
break_duration = 5    # 休息时长（分钟）
long_break_duration = 15    # 长休息时长（分钟）
long_break_interval = 4    # 长休息前的工作阶段数

[notification]
sound = true    # 启用声音通知
desktop = true  # 启用桌面通知
```

## 故障排除

如果您在 Waybar 集成中遇到问题：

1. 确保守护进程正在运行：

```bash
tomato-clock daemon
```

2. 检查输出文件是否存在：

```bash
cat ~/.config/tomato-clock/waybar-output.json
```

3. 如果遇到"Failed to send xxx event"错误，请使用上述基于 Socket 的集成方法。

## 项目结构

- `src/` - Rust 源代码
- `config/` - 配置示例
- `scripts/` - Waybar 集成辅助脚本
- `sockets/` - 基于 Socket 的集成文件

## 开发路线图

查看[project_plan.md](project_plan.md)了解开发路线图。

## 通知系统

查看[notification_plan.md](notification_plan.md)了解通知系统的详细信息。

## 许可证

本项目基于 MIT 许可证 - 详见[LICENSE](LICENSE)文件。

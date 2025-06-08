# Tomato Clock Socket 集成方案

本文档描述了一个基于 Unix Socket 的通信方案，用于提高 Tomato Clock 与 Waybar 的集成稳定性，解决"Failed to send xxx event"错误。

## 架构概述

新的集成方案采用客户端-服务器架构，包含以下组件：

1. **Socket 服务器** (`socket_server.py`)：

   - 维护与 tomato-clock 守护进程的通信
   - 提供统一的命令接口
   - 管理状态并生成 Waybar 输出
   - 防止状态同步问题

2. **Socket 客户端** (`socket_client.py`)：

   - 提供命令行接口
   - 自动启动服务器（如果未运行）
   - 处理命令并返回结果

3. **Waybar 模块脚本** (`waybar_socket_module.sh`)：

   - 使用客户端获取 Waybar 输出
   - 确保始终有可用输出

4. **服务管理脚本** (`socket_service.sh`)：

   - 管理 Socket 服务器的启动、停止和状态查询

5. **Systemd 服务** (`socket_systemd.service`)：
   - 确保服务器在系统启动时自动运行
   - 管理服务生命周期

## 文件说明

| 文件名                     | 描述                              |
| -------------------------- | --------------------------------- |
| `socket_server.py`         | Socket 服务器，管理状态并处理命令 |
| `socket_client.py`         | Socket 客户端，发送命令到服务器   |
| `waybar_socket_module.sh`  | Waybar 模块脚本，获取状态输出     |
| `socket_service.sh`        | 服务管理脚本，控制服务器运行      |
| `socket_systemd.service`   | Systemd 服务配置文件              |
| `install_socket_server.sh` | 安装脚本，自动配置环境            |

## 工作流程

1. **启动过程**：

   - Systemd 启动 Socket 服务器
   - 服务器确保 tomato-clock 守护进程运行
   - 服务器读取初始状态并开始监听命令

2. **状态更新**：

   - 服务器每秒读取一次状态文件
   - 生成 Waybar 输出文件
   - 保持状态一致性

3. **命令处理**：
   - Waybar 点击事件触发 socket_client.py
   - 客户端发送命令到服务器
   - 服务器执行命令并更新状态

## 优势

1. **更可靠的通信**：

   - 使用 Socket 替代直接文件 IO
   - 集中处理状态更新和命令
   - 避免并发问题

2. **更好的状态同步**：

   - 服务器保持状态一致性
   - 防止状态回退
   - 提供更丰富的错误处理

3. **错误处理改进**：
   - 详细日志记录
   - 自动重启机制
   - 更明确的错误消息

## 安装方法

执行安装脚本即可自动安装并配置：

```bash
chmod +x test_tomato/install_socket_server.sh
./test_tomato/install_socket_server.sh
```

安装过程包括：

1. 设置必要文件权限
2. 安装 Systemd 服务
3. 配置 Waybar
4. 启动服务器
5. 重启 Waybar

## 使用方法

### 通过 Waybar 使用

Waybar 集成后，可以通过点击 Waybar 上的番茄图标来控制计时器：

- 左键点击：开始计时器
- 中键点击：停止计时器
- 右键点击：跳过当前阶段

### 命令行使用

也可以通过命令行直接控制：

```bash
# 查看状态
./test_tomato/socket_client.py status

# 启动计时器
./test_tomato/socket_client.py start

# 停止计时器
./test_tomato/socket_client.py stop

# 暂停计时器
./test_tomato/socket_client.py pause

# 恢复计时器
./test_tomato/socket_client.py resume

# 跳过当前阶段
./test_tomato/socket_client.py skip
```

### 服务管理

可以使用服务管理脚本控制服务器：

```bash
# 查看服务状态
./test_tomato/socket_service.sh status

# 启动服务
./test_tomato/socket_service.sh start

# 停止服务
./test_tomato/socket_service.sh stop

# 重启服务
./test_tomato/socket_service.sh restart
```

## 故障排除

1. **服务器未运行**：

   - 检查 systemd 服务状态：`systemctl --user status tomato-clock-socket.service`
   - 手动启动服务：`./test_tomato/socket_service.sh start`
   - 检查日志：`cat ~/.config/tomato-clock/socket_server.log`

2. **Waybar 模块不显示**：

   - 确认服务器运行：`./test_tomato/socket_service.sh status`
   - 检查 Waybar 配置
   - 重启 Waybar：`killall waybar && waybar &`

3. **命令无效**：
   - 检查客户端权限：`chmod +x ./test_tomato/socket_client.py`
   - 确认服务器状态：`./test_tomato/socket_service.sh status`
   - 尝试重启服务：`./test_tomato/socket_service.sh restart`

## 卸载方法

如需卸载，请执行以下步骤：

1. 停止并禁用服务：

   ```bash
   systemctl --user stop tomato-clock-socket.service
   systemctl --user disable tomato-clock-socket.service
   ```

2. 恢复 Waybar 配置
3. 删除服务文件：
   ```bash
   rm ~/.config/systemd/user/tomato-clock-socket.service
   ```

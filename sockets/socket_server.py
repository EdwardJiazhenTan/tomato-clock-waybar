#!/usr/bin/env python3

import os
import socket
import json
import time
import sys
import signal
import threading
import subprocess
from pathlib import Path

# 配置
SOCKET_PATH = os.path.expanduser("~/.config/tomato-clock/tomato.sock")
OUTPUT_FILE = os.path.expanduser("~/.config/tomato-clock/waybar-output.json")
STATE_FILE = os.path.expanduser("~/.config/tomato-clock/state.json")
TOMATO_BIN = os.path.expanduser("~/.local/bin/tomato-clock")
LOG_FILE = os.path.expanduser("~/.config/tomato-clock/socket_server.log")

# 确保目录存在
os.makedirs(os.path.dirname(SOCKET_PATH), exist_ok=True)

# 设置日志
def log(message):
    timestamp = time.strftime("%Y-%m-%d %H:%M:%S")
    with open(LOG_FILE, "a") as f:
        f.write(f"{timestamp} - {message}\n")

# 从状态文件获取最新状态
def get_current_state():
    try:
        if os.path.exists(STATE_FILE):
            with open(STATE_FILE, "r") as f:
                return json.load(f)
        return {"timer_state": "Stopped", "workflow_name": "Default Pomodoro", "current_status": "work"}
    except Exception as e:
        log(f"获取状态出错: {str(e)}")
        return {"timer_state": "Error", "error": str(e)}

# 生成Waybar输出
def generate_waybar_output(state):
    try:
        timer_state = state.get("timer_state", "Stopped")
        workflow_name = state.get("workflow_name", "Default Pomodoro")
        current_status = state.get("current_status", "work")
        start_time = state.get("start_time", "")
        elapsed_seconds = state.get("elapsed_seconds", 0)
        
        # 计算剩余时间
        phase_duration = 25 * 60  # 默认25分钟
        if current_status == "break":
            phase_duration = 5 * 60  # 默认5分钟休息
        
        remaining_seconds = max(0, phase_duration - elapsed_seconds)
        minutes = remaining_seconds // 60
        seconds = remaining_seconds % 60
        
        # 根据状态设置图标和类
        if timer_state == "Running":
            if current_status == "work":
                icon = "🔨"
                css_class = "running"
                alt_color = "#ff5555"  # 工作时为红色
            else:
                icon = "☕"
                css_class = "running"
                alt_color = "#50fa7b"  # 休息时为绿色
        elif timer_state == "Paused":
            icon = "⏸️"
            css_class = "paused"
            alt_color = "#f1fa8c"
        else:
            icon = "🍅"
            css_class = "idle"
            alt_color = "#bd93f9"
        
        # 计算百分比
        percentage = min(100, int((elapsed_seconds / phase_duration) * 100)) if phase_duration > 0 else 0
        
        # 生成文本和提示
        text = f"{icon} {current_status}: {minutes:02d}:{seconds:02d}"
        tooltip = f"{current_status}: {workflow_name}\nRemaining: {minutes:02d}:{seconds:02d}"
        if timer_state == "Running" or timer_state == "Paused":
            elapsed_min = elapsed_seconds // 60
            elapsed_sec = elapsed_seconds % 60
            tooltip += f"\nElapsed: {elapsed_min:02d}:{elapsed_sec:02d}"
        
        # 创建Waybar输出
        output = {
            "text": text,
            "tooltip": tooltip,
            "class": css_class,
            "percentage": percentage,
            "alt": alt_color
        }
        
        return output
    except Exception as e:
        log(f"生成输出出错: {str(e)}")
        return {"text": "🍅 Error", "tooltip": f"Error: {str(e)}", "class": "error"}

# 更新Waybar输出文件
def update_waybar_output(output):
    try:
        with open(OUTPUT_FILE, "w") as f:
            json.dump(output, f)
    except Exception as e:
        log(f"更新输出文件出错: {str(e)}")

# 检查守护进程并在需要时启动
def ensure_daemon_running():
    try:
        result = subprocess.run(["pgrep", "-f", "tomato-clock daemon"], capture_output=True, text=True)
        if result.returncode != 0:
            log("守护进程未运行，正在启动...")
            subprocess.Popen([TOMATO_BIN, "daemon"], 
                            stdout=subprocess.DEVNULL, 
                            stderr=subprocess.DEVNULL)
            time.sleep(1)
            return True
        return True
    except Exception as e:
        log(f"检查守护进程出错: {str(e)}")
        return False

# 处理命令
def handle_command(cmd):
    try:
        if cmd.startswith("status"):
            # 返回当前状态
            state = get_current_state()
            return json.dumps(state)
        elif cmd.startswith("output"):
            # 返回当前Waybar输出
            state = get_current_state()
            output = generate_waybar_output(state)
            return json.dumps(output)
        elif cmd in ["start", "stop", "pause", "resume", "skip"]:
            # 执行tomato-clock命令
            log(f"执行命令: {cmd}")
            result = subprocess.run([TOMATO_BIN, cmd], capture_output=True, text=True)
            if result.returncode == 0:
                time.sleep(0.5)  # 等待状态更新
                state = get_current_state()
                output = generate_waybar_output(state)
                update_waybar_output(output)
                return "OK"
            else:
                log(f"命令执行失败: {result.stderr}")
                return f"ERROR: {result.stderr}"
        else:
            return f"未知命令: {cmd}"
    except Exception as e:
        log(f"处理命令出错: {str(e)}")
        return f"ERROR: {str(e)}"

# 后台任务：定期更新Waybar输出
def update_task():
    while not exit_event.is_set():
        try:
            if ensure_daemon_running():
                state = get_current_state()
                output = generate_waybar_output(state)
                update_waybar_output(output)
        except Exception as e:
            log(f"更新任务出错: {str(e)}")
        time.sleep(1)  # 每秒更新一次

# 主socket服务器
def run_server():
    # 移除已存在的socket文件
    try:
        if os.path.exists(SOCKET_PATH):
            os.unlink(SOCKET_PATH)
    except OSError:
        log(f"无法删除已存在的socket: {SOCKET_PATH}")
        sys.exit(1)

    # 创建socket服务器
    server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    server.bind(SOCKET_PATH)
    server.listen(5)
    os.chmod(SOCKET_PATH, 0o666)  # 允许所有用户访问socket
    
    log(f"Socket服务器已启动，监听: {SOCKET_PATH}")
    
    try:
        while not exit_event.is_set():
            try:
                server.settimeout(1.0)  # 设置超时，以便能检查exit_event
                conn, addr = server.accept()
                
                data = conn.recv(1024).decode('utf-8').strip()
                if data:
                    response = handle_command(data)
                    conn.send(response.encode('utf-8'))
                
                conn.close()
            except socket.timeout:
                continue
            except Exception as e:
                log(f"处理连接出错: {str(e)}")
    finally:
        server.close()
        if os.path.exists(SOCKET_PATH):
            os.unlink(SOCKET_PATH)

# 处理退出信号
def signal_handler(sig, frame):
    log("接收到退出信号，正在关闭...")
    exit_event.set()
    sys.exit(0)

if __name__ == "__main__":
    # 初始化
    log("启动socket服务器...")
    
    # 设置退出事件和信号处理
    exit_event = threading.Event()
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # 确保tomato-clock守护进程在运行
    ensure_daemon_running()
    
    # 启动更新线程
    update_thread = threading.Thread(target=update_task)
    update_thread.daemon = True
    update_thread.start()
    
    # 运行服务器
    run_server() 
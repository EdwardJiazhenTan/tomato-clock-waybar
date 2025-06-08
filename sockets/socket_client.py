#!/usr/bin/env python3

import os
import socket
import sys
import json
import time
import subprocess

# 配置
SOCKET_PATH = os.path.expanduser("~/.config/tomato-clock/tomato.sock")
SERVER_SCRIPT = os.path.expanduser("~/Documents/personal/tomato-clock-waybar/sockets/socket_server.py")

def ensure_server_running():
    """确保socket服务器正在运行"""
    # 检查socket文件是否存在
    if not os.path.exists(SOCKET_PATH):
        print("Socket服务器未运行，正在启动...")
        # 启动服务器
        try:
            subprocess.Popen(["python3", SERVER_SCRIPT], 
                            stdout=subprocess.DEVNULL, 
                            stderr=subprocess.DEVNULL)
            # 等待socket文件创建
            for _ in range(5):
                if os.path.exists(SOCKET_PATH):
                    break
                time.sleep(0.5)
            else:
                print("启动socket服务器超时")
                return False
        except Exception as e:
            print(f"启动socket服务器失败: {e}")
            return False
    
    return True

def send_command(command):
    """发送命令到socket服务器"""
    if not ensure_server_running():
        return None
    
    # 创建socket客户端
    client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    
    try:
        # 连接到服务器
        client.connect(SOCKET_PATH)
        
        # 发送命令
        client.send(command.encode('utf-8'))
        
        # 接收响应
        response = client.recv(4096).decode('utf-8')
        
        return response
    except Exception as e:
        print(f"发送命令失败: {e}")
        return None
    finally:
        client.close()

def get_status():
    """获取当前状态"""
    response = send_command("status")
    if response:
        try:
            return json.loads(response)
        except:
            return {"error": response}
    return {"error": "无法获取状态"}

def get_output():
    """获取Waybar输出"""
    response = send_command("output")
    if response:
        try:
            return json.loads(response)
        except:
            return {"text": "🍅 Error", "tooltip": response, "class": "error"}
    return {"text": "🍅 Error", "tooltip": "无法连接到服务器", "class": "error"}

def control_timer(action):
    """控制计时器: start, stop, pause, resume, skip"""
    if action not in ["start", "stop", "pause", "resume", "skip"]:
        print(f"无效的操作: {action}")
        return False
    
    response = send_command(action)
    if response == "OK":
        return True
    else:
        print(f"操作失败: {response}")
        return False

def main():
    """主函数"""
    if len(sys.argv) < 2:
        print("用法: socket_client.py <命令>")
        print("可用命令: status, output, start, stop, pause, resume, skip")
        return
    
    command = sys.argv[1]
    
    if command == "status":
        status = get_status()
        print(json.dumps(status, indent=2, ensure_ascii=False))
    elif command == "output":
        output = get_output()
        print(json.dumps(output, ensure_ascii=False))
    elif command in ["start", "stop", "pause", "resume", "skip"]:
        success = control_timer(command)
        if success:
            print(f"命令 '{command}' 执行成功")
        else:
            print(f"命令 '{command}' 执行失败")
    else:
        print(f"未知命令: {command}")

if __name__ == "__main__":
    main() 
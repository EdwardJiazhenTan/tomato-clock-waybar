#!/bin/bash

# Waybar模块脚本 - 使用socket客户端获取番茄钟状态

CLIENT_PATH="$HOME/Documents/personal/tomato-clock-waybar/sockets/socket_client.py"

# 默认输出，以防无法连接
DEFAULT_OUTPUT='{"text":"🍅","tooltip":"Tomato Clock","class":"idle"}'

# 检查客户端是否存在
if [ ! -f "$CLIENT_PATH" ]; then
    echo "$DEFAULT_OUTPUT"
    exit 1
fi

# 检查客户端是否可执行
if [ ! -x "$CLIENT_PATH" ]; then
    chmod +x "$CLIENT_PATH"
fi

# 获取输出
OUTPUT=$("$CLIENT_PATH" output 2>/dev/null)

# 检查输出是否有效
if [ -z "$OUTPUT" ]; then
    echo "$DEFAULT_OUTPUT"
else
    echo "$OUTPUT"
fi 
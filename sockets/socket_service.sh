#!/bin/bash

# Socket服务管理脚本

SERVER_SCRIPT="$HOME/Documents/personal/tomato-clock-waybar/sockets/socket_server.py"
PID_FILE="/tmp/tomato_socket_server.pid"
SOCKET_PATH="$HOME/.config/tomato-clock/tomato.sock"
LOG_FILE="$HOME/.config/tomato-clock/socket_server.log"

# 确保目录存在
mkdir -p "$HOME/.config/tomato-clock"

# 检查服务器是否运行
is_running() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null; then
            return 0  # 运行中
        fi
    fi
    return 1  # 未运行
}

# 启动服务器
start_server() {
    if is_running; then
        echo "Socket服务器已经在运行中，PID: $(cat "$PID_FILE")"
        return 0
    fi
    
    echo "启动Socket服务器..."
    if [ ! -f "$SERVER_SCRIPT" ]; then
        echo "错误: 找不到服务器脚本: $SERVER_SCRIPT"
        return 1
    fi
    
    python3 "$SERVER_SCRIPT" > /dev/null 2>&1 &
    PID=$!
    echo $PID > "$PID_FILE"
    
    # 等待socket文件创建
    for i in {1..10}; do
        if [ -S "$SOCKET_PATH" ]; then
            echo "Socket服务器已启动，PID: $PID"
            return 0
        fi
        sleep 0.5
    done
    
    echo "启动Socket服务器超时，请检查日志: $LOG_FILE"
    kill $PID 2>/dev/null
    return 1
}

# 停止服务器
stop_server() {
    if ! is_running; then
        echo "Socket服务器未运行"
        return 0
    fi
    
    PID=$(cat "$PID_FILE")
    echo "停止Socket服务器，PID: $PID"
    kill $PID
    
    # 等待进程结束
    for i in {1..5}; do
        if ! ps -p "$PID" > /dev/null; then
            echo "Socket服务器已停止"
            rm -f "$PID_FILE"
            return 0
        fi
        sleep 1
    done
    
    # 如果进程仍在运行，强制终止
    echo "Socket服务器未响应，强制终止"
    kill -9 $PID 2>/dev/null
    rm -f "$PID_FILE"
    return 0
}

# 重启服务器
restart_server() {
    stop_server
    sleep 1
    start_server
}

# 显示服务器状态
status_server() {
    if is_running; then
        PID=$(cat "$PID_FILE")
        echo "Socket服务器正在运行，PID: $PID"
        echo "Socket路径: $SOCKET_PATH"
        echo "日志文件: $LOG_FILE"
        
        # 检查socket文件
        if [ -S "$SOCKET_PATH" ]; then
            echo "Socket文件存在且有效"
        else
            echo "警告: Socket文件不存在或无效"
        fi
        
        # 显示最近的日志
        if [ -f "$LOG_FILE" ]; then
            echo "最近日志 (最后10行):"
            tail -n 10 "$LOG_FILE"
        else
            echo "日志文件不存在"
        fi
    else
        echo "Socket服务器未运行"
    fi
}

# 主函数
case "$1" in
    start)
        start_server
        ;;
    stop)
        stop_server
        ;;
    restart)
        restart_server
        ;;
    status)
        status_server
        ;;
    *)
        echo "用法: $0 {start|stop|restart|status}"
        exit 1
        ;;
esac

exit 0 
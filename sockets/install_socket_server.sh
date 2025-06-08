#!/bin/bash

# Tomato Clock Socket服务器安装脚本

echo "=== 安装Tomato Clock Socket服务器 ==="
echo "当前时间: $(date)"

# 配置路径
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WAYBAR_CONFIG_DIR="$HOME/.config/waybar"
SYSTEMD_DIR="$HOME/.config/systemd/user"
TOMATO_CONFIG_DIR="$HOME/.config/tomato-clock"

# 1. 确保目录存在
echo "1. 创建必要目录..."
mkdir -p "$WAYBAR_CONFIG_DIR"
mkdir -p "$SYSTEMD_DIR"
mkdir -p "$TOMATO_CONFIG_DIR"

# 2. 设置脚本权限
echo "2. 设置脚本权限..."
chmod +x "$SCRIPT_DIR/socket_server.py"
chmod +x "$SCRIPT_DIR/socket_client.py"
chmod +x "$SCRIPT_DIR/waybar_socket_module.sh"
chmod +x "$SCRIPT_DIR/socket_service.sh"

# 3. 安装systemd服务
echo "3. 安装systemd服务..."
cp "$SCRIPT_DIR/socket_systemd.service" "$SYSTEMD_DIR/tomato-clock-socket.service"
systemctl --user daemon-reload
systemctl --user enable tomato-clock-socket.service
echo "systemd服务已安装并启用"

# 4. 添加Waybar配置
echo "4. 配置Waybar..."

# 检查现有配置
if [ -f "$WAYBAR_CONFIG_DIR/config" ]; then
    # 备份现有配置
    cp "$WAYBAR_CONFIG_DIR/config" "$WAYBAR_CONFIG_DIR/config.bak.$(date +%Y%m%d%H%M%S)"
    
    # 更新tomato模块配置
    if grep -q "custom/tomato" "$WAYBAR_CONFIG_DIR/config"; then
        echo "更新现有的tomato模块配置..."
        # 移除现有配置
        sed -i '/^\s*"custom\/tomato": {/,/^\s*}/d' "$WAYBAR_CONFIG_DIR/config"
    fi
    
    # 添加新配置
    sed -i '$ s/}/,\n    "custom\/tomato": {\n        "exec": "'"$SCRIPT_DIR"'\/waybar_socket_module.sh",\n        "return-type": "json",\n        "interval": 1,\n        "on-click": "'"$SCRIPT_DIR"'\/socket_client.py start",\n        "on-click-middle": "'"$SCRIPT_DIR"'\/socket_client.py stop",\n        "on-click-right": "'"$SCRIPT_DIR"'\/socket_client.py skip"\n    }\n}/' "$WAYBAR_CONFIG_DIR/config"
    
    echo "Waybar配置已更新"
else
    echo "创建新的Waybar配置..."
    # 创建基本配置
    cat > "$WAYBAR_CONFIG_DIR/config" << EOF
{
    "layer": "top",
    "position": "top",
    "height": 30,
    "modules-left": ["sway/workspaces", "sway/mode"],
    "modules-center": ["sway/window"],
    "modules-right": ["pulseaudio", "network", "battery", "clock", "custom/tomato"],
    
    "clock": {
        "format": "{:%H:%M}",
        "tooltip-format": "{:%Y-%m-%d | %H:%M}"
    },
    
    "custom/tomato": {
        "exec": "$SCRIPT_DIR/waybar_socket_module.sh",
        "return-type": "json",
        "interval": 1,
        "on-click": "$SCRIPT_DIR/socket_client.py start",
        "on-click-middle": "$SCRIPT_DIR/socket_client.py stop",
        "on-click-right": "$SCRIPT_DIR/socket_client.py skip"
    }
}
EOF
    echo "已创建新的Waybar配置"
fi

# 5. 添加CSS样式
echo "5. 添加CSS样式..."
if [ -f "$WAYBAR_CONFIG_DIR/style.css" ]; then
    # 检查是否已有tomato样式
    if ! grep -q "#custom-tomato" "$WAYBAR_CONFIG_DIR/style.css"; then
        # 添加tomato样式到现有样式文件
        cat >> "$WAYBAR_CONFIG_DIR/style.css" << 'EOF'

/* Tomato Clock 样式 */
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

#custom-tomato.error {
    background-color: #ff7675;
    color: #ffffff;
}
EOF
        echo "添加了Tomato Clock样式到现有样式文件"
    else
        echo "样式文件已包含Tomato Clock样式"
    fi
else
    echo "创建新的样式文件..."
    # 创建基本样式
    cat > "$WAYBAR_CONFIG_DIR/style.css" << 'EOF'
* {
    border: none;
    border-radius: 0;
    font-family: "Noto Sans", "Font Awesome 6 Free";
    font-size: 14px;
}

window#waybar {
    background: rgba(43, 48, 59, 0.8);
    color: #ffffff;
}

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

#custom-tomato.error {
    background-color: #ff7675;
    color: #ffffff;
}
EOF
    echo "已创建新的样式文件"
fi

# 6. 启动socket服务器
echo "6. 启动socket服务器..."
"$SCRIPT_DIR/socket_service.sh" start

# 7. 重启Waybar
echo "7. 重启Waybar..."
if pgrep -x waybar > /dev/null; then
    killall waybar
    sleep 1
    waybar > /dev/null 2>&1 &
    echo "Waybar已重启"
else
    waybar > /dev/null 2>&1 &
    echo "Waybar已启动"
fi

echo "=== 安装完成 ==="
echo "Tomato Clock Socket服务器已安装并启动"
echo "Waybar配置已更新"
echo ""
echo "提示:"
echo "1. 使用 ${SCRIPT_DIR}/socket_service.sh status 检查服务器状态"
echo "2. 使用 ${SCRIPT_DIR}/socket_client.py status 查看当前计时器状态"
echo "3. 如需卸载，请运行 systemctl --user disable tomato-clock-socket.service" 
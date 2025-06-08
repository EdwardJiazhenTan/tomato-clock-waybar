#!/bin/bash

# Tomato Clock Waybar Installation Script

set -e

echo "Building Tomato Clock..."
cargo build --release

echo "Creating necessary directories..."
mkdir -p ~/.config/waybar/scripts
mkdir -p ~/.local/bin

echo "Installing Tomato Clock binary..."
cp target/release/tomato-clock ~/.local/bin/

echo "Installing Waybar module scripts..."
cp scripts/waybar-module.sh ~/.config/waybar/scripts/
chmod +x ~/.config/waybar/scripts/waybar-module.sh

# Copy toggle script
cp scripts/toggle.sh ~/.config/waybar/scripts/
chmod +x ~/.config/waybar/scripts/toggle.sh

echo "Creating configuration directory..."
mkdir -p ~/.config/tomato-clock

# Copy default config if it doesn't exist
if [ ! -f ~/.config/tomato-clock/config.toml ]; then
    echo "Installing default configuration..."
    cp config/default_config.toml ~/.config/tomato-clock/config.toml
fi

echo "Installation completed!"
echo ""
echo "Add the following to your Waybar config file (~/.config/waybar/config):"
echo ""
echo '"custom/tomato": {'
echo '    "exec": "~/.config/waybar/scripts/waybar-module.sh",'
echo '    "return-type": "json",'
echo '    "interval": 1,'
echo '    "on-click": "~/.config/waybar/scripts/waybar-module.sh 1",'
echo '    "on-click-middle": "~/.config/waybar/scripts/waybar-module.sh 2",'
echo '    "on-click-right": "~/.config/waybar/scripts/waybar-module.sh 3"'
echo '}'
echo ""
echo "And add CSS styling to your Waybar style file (~/.config/waybar/style.css)"
echo ""
echo "To start the daemon, run:"
echo "tomato-clock daemon &"
echo ""
echo "Enjoy your Tomato Clock!" 
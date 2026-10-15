#!/usr/bin/env bash
set -e

# Change to repository root
ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT_DIR"

echo "==> [1/5] Building release binary with Cargo..."
cargo build --release --bin esperavox

APP_DIR="$ROOT_DIR/AppDir"
OUTPUT_APPIMAGE="$ROOT_DIR/EsperaVox-x86_64.AppImage"

echo "==> [2/5] Preparing AppDir structure..."
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/usr/bin"
mkdir -p "$APP_DIR/usr/share/applications"
mkdir -p "$APP_DIR/usr/share/icons/hicolor/256x256/apps"

# Copy binary & strip debug symbols
cp "$ROOT_DIR/target/release/esperavox" "$APP_DIR/usr/bin/esperavox"
strip "$APP_DIR/usr/bin/esperavox" 2>/dev/null || true

# Copy icon
if [ -f "$ROOT_DIR/examples/Bot_Transparent.png" ]; then
    cp "$ROOT_DIR/examples/Bot_Transparent.png" "$APP_DIR/esperavox.png"
    cp "$ROOT_DIR/examples/Bot_Transparent.png" "$APP_DIR/usr/share/icons/hicolor/256x256/apps/esperavox.png"
elif [ -f "$ROOT_DIR/examples/Retro_Bot.png" ]; then
    cp "$ROOT_DIR/examples/Retro_Bot.png" "$APP_DIR/esperavox.png"
    cp "$ROOT_DIR/examples/Retro_Bot.png" "$APP_DIR/usr/share/icons/hicolor/256x256/apps/esperavox.png"
fi

echo "==> [3/5] Creating Desktop entry and AppRun..."
cat << 'EOF' > "$APP_DIR/esperavox.desktop"
[Desktop Entry]
Type=Application
Name=EsperaVox
GenericName=Voxel Art Studio
Comment=Native, lightweight Voxel Art Studio for Linux
Exec=esperavox %F
Icon=esperavox
Categories=Graphics;3DGraphics;
Terminal=false
StartupNotify=true
MimeType=model/vnd.magicavoxel.vox;
EOF

cp "$APP_DIR/esperavox.desktop" "$APP_DIR/usr/share/applications/esperavox.desktop"

cat << 'EOF' > "$APP_DIR/AppRun"
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
exec "${HERE}/usr/bin/esperavox" "$@"
EOF

chmod +x "$APP_DIR/AppRun"

echo "==> [4/5] Locating appimagetool..."
APPIMAGETOOL=""
if command -v appimagetool >/dev/null 2>&1; then
    APPIMAGETOOL="$(command -v appimagetool)"
elif [ -x "$HOME/Desktop/SCRIPTS/bin/appimagetool" ]; then
    APPIMAGETOOL="$HOME/Desktop/SCRIPTS/bin/appimagetool"
elif [ -x "$HOME/.local/bin/appimagetool" ]; then
    APPIMAGETOOL="$HOME/.local/bin/appimagetool"
fi

if [ -z "$APPIMAGETOOL" ]; then
    echo "appimagetool not found locally. Downloading standalone appimagetool..."
    TOOL_URL="https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
    curl -L "$TOOL_URL" -o /tmp/appimagetool
    chmod +x /tmp/appimagetool
    APPIMAGETOOL="/tmp/appimagetool"
fi

echo "==> [5/5] Packaging AppImage using $APPIMAGETOOL..."
ARCH=x86_64 "$APPIMAGETOOL" "$APP_DIR" "$OUTPUT_APPIMAGE"

# Clean up temporary AppDir
rm -rf "$APP_DIR"

echo "==> SUCCESS: Created AppImage at: $OUTPUT_APPIMAGE"
ls -lh "$OUTPUT_APPIMAGE"

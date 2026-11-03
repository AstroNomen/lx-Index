#!/usr/bin/env bash
set -euo pipefail

echo "============================================="
echo " Packaging lx Index for macOS (Apple Silicon)"
echo "============================================="

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_BIN="${PROJECT_ROOT}/target/release/lx-index"
DIST_DIR="${PROJECT_ROOT}/target/release/bundle/osx"
APP_NAME="lx Index.app"
APP_DIR="${DIST_DIR}/${APP_NAME}"
DMG_PATH="${PROJECT_ROOT}/target/release/lx-index-0.1.0-macOS.dmg"

# 1. Ensure binary exists
if [[ ! -f "${RELEASE_BIN}" ]]; then
    echo "Release binary not found. Building with cargo build --release..."
    cargo build --release
fi

# 2. Setup .app Bundle directory structure
echo "Creating .app bundle structure at: ${APP_DIR}"
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

# 3. Copy executable
cp "${RELEASE_BIN}" "${APP_DIR}/Contents/MacOS/lx-index"
chmod +x "${APP_DIR}/Contents/MacOS/lx-index"

# 4. Generate Info.plist
cat > "${APP_DIR}/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>lx Index</string>
    <key>CFBundleExecutable</key>
    <string>lx-index</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.lucentia.lxindex</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>lx Index</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
EOF

# 5. Create a clean AppIcon if possible, or iconset
# Create an empty .icns or minimal icon
ICONSET_DIR="/tmp/lx_index_icon.iconset"
rm -rf "${ICONSET_DIR}"
mkdir -p "${ICONSET_DIR}"

# Create a gold Lucentia 1024x1024 base PNG using sips or python
python3 -c '
from PIL import Image, ImageDraw
img = Image.new("RGBA", (1024, 1024), (24, 25, 28, 255))
draw = ImageDraw.Draw(img)
# Draw Lucentia Gold Accent border
draw.rounded_rectangle([32, 32, 992, 992], radius=220, fill=(28, 30, 36, 255), outline=(197, 160, 89, 255), width=28)
# Save base
img.save("/tmp/lx_index_icon.iconset/icon_512x512@2x.png")
' 2>/dev/null || true

if [[ -f "/tmp/lx_index_icon.iconset/icon_512x512@2x.png" ]]; then
    sips -z 16 16     /tmp/lx_index_icon.iconset/icon_512x512@2x.png --out "${ICONSET_DIR}/icon_16x16.png" > /dev/null 2>&1 || true
    sips -z 32 32     /tmp/lx_index_icon.iconset/icon_512x512@2x.png --out "${ICONSET_DIR}/icon_16x16@2x.png" > /dev/null 2>&1 || true
    sips -z 32 32     /tmp/lx_index_icon.iconset/icon_512x512@2x.png --out "${ICONSET_DIR}/icon_32x32.png" > /dev/null 2>&1 || true
    sips -z 64 64     /tmp/lx_index_icon.iconset/icon_512x512@2x.png --out "${ICONSET_DIR}/icon_32x32@2x.png" > /dev/null 2>&1 || true
    sips -z 128 128   /tmp/lx_index_icon.iconset/icon_512x512@2x.png --out "${ICONSET_DIR}/icon_128x128.png" > /dev/null 2>&1 || true
    sips -z 256 256   /tmp/lx_index_icon.iconset/icon_512x512@2x.png --out "${ICONSET_DIR}/icon_128x128@2x.png" > /dev/null 2>&1 || true
    sips -z 256 256   /tmp/lx_index_icon.iconset/icon_512x512@2x.png --out "${ICONSET_DIR}/icon_256x256.png" > /dev/null 2>&1 || true
    sips -z 512 512   /tmp/lx_index_icon.iconset/icon_512x512@2x.png --out "${ICONSET_DIR}/icon_256x256@2x.png" > /dev/null 2>&1 || true
    sips -z 512 512   /tmp/lx_index_icon.iconset/icon_512x512@2x.png --out "${ICONSET_DIR}/icon_512x512.png" > /dev/null 2>&1 || true
    iconutil -c icns "${ICONSET_DIR}" -o "${APP_DIR}/Contents/Resources/AppIcon.icns" 2>/dev/null || true
    rm -rf "${ICONSET_DIR}"
fi

# 6. Verify Bundle validity with plutil
plutil -lint "${APP_DIR}/Contents/Info.plist"

# 7. Create .dmg Image
echo "Building macOS .dmg disk image..."
rm -f "${DMG_PATH}"

# Create temporary staging directory for DMG with /Applications symlink
DMG_STAGING="/tmp/lx_index_dmg_staging"
rm -rf "${DMG_STAGING}"
mkdir -p "${DMG_STAGING}"
cp -R "${APP_DIR}" "${DMG_STAGING}/"
ln -s /Applications "${DMG_STAGING}/Applications"

hdiutil create \
    -volname "lx Index" \
    -srcfolder "${DMG_STAGING}" \
    -ov \
    -format UDZO \
    "${DMG_PATH}"

rm -rf "${DMG_STAGING}"

echo "============================================="
echo " Packaging Complete!"
echo " App Bundle: ${APP_DIR}"
echo " Disk Image: ${DMG_PATH}"
echo "============================================="

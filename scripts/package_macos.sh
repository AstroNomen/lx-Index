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
DMG_PATH="${PROJECT_ROOT}/target/release/lx-index-0.5.0-macOS.dmg"

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
    <string>0.5.0</string>
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

# 5. Install official AppIcon.icns
if [[ -f "${PROJECT_ROOT}/assets/AppIcon.icns" ]]; then
    echo "Installing AppIcon.icns to ${APP_DIR}/Contents/Resources/AppIcon.icns"
    cp "${PROJECT_ROOT}/assets/AppIcon.icns" "${APP_DIR}/Contents/Resources/AppIcon.icns"
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

# PowerShell Script for Packaging lx Index for Windows (x86_64-pc-windows-msvc)
param (
    [switch]$SkipBuild = $false
)

$ErrorActionPreference = "Stop"

Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "  Packaging lx Index for Windows (x64 Release)" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
Set-Location $ProjectRoot

# Ensure cargo is in PATH
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    if (Test-Path "$env:USERPROFILE\.cargo\bin\cargo.exe") {
        $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    } else {
        Write-Error "Cargo was not found in PATH or in %USERPROFILE%\.cargo\bin"
    }
}

$Version = "0.6.0"
$ReleaseExe = Join-Path $ProjectRoot "target\release\lx-index.exe"
$DistDir = Join-Path $ProjectRoot "target\release\bundle\windows\lx-index-windows-x64"
$ZipOutput = Join-Path $ProjectRoot "target\release\lx-index-$Version-windows-x64.zip"

# 1. Build release binary if requested or missing
if (-not $SkipBuild -or -not (Test-Path $ReleaseExe)) {
    Write-Host "[1/4] Compiling optimized release binary (cargo build --release)..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Cargo release build failed with exit code $LASTEXITCODE"
    }
} else {
    Write-Host "[1/4] Skipping build (existing release binary found)..." -ForegroundColor Gray
}

# 2. Setup Staging Directory
Write-Host "[2/4] Setting up portable distribution folder at: $DistDir" -ForegroundColor Yellow
Get-Process lx-index -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300

if (Test-Path $DistDir) {
    Remove-Item -Recurse -Force $DistDir
}
New-Item -ItemType Directory -Path $DistDir -Force | Out-Null

# 3. Copy files to distribution folder
Write-Host "[3/4] Copying standalone executable and auxiliary documents..." -ForegroundColor Yellow
Copy-Item $ReleaseExe -Destination (Join-Path $DistDir "lx-index.exe") -Force

if (Test-Path (Join-Path $ProjectRoot "LICENSE")) {
    Copy-Item (Join-Path $ProjectRoot "LICENSE") -Destination (Join-Path $DistDir "LICENSE.txt") -Force
}

# Create a clean Windows Quick Start README
$ReadmeContent = @"
========================================================================
lx Index - Literature Management HUD & Companion Workspace (v$Version)
Lucentia Aesthetics Laboratory | Pure Compiled Native Desktop Application
========================================================================

【软件特性 / Features】
- 零私有数据库依赖：以本地文件系统与目录结构为唯一真理来源（SSOT）。
- 无损 Frontmatter 协议：智能增删修改 YAML 标签与元数据，无损保留个人见解与 Markdown 正文。
- Windows 8 Modern/Metro UI：深度融合 Panorama（全景长卷）与 Pivot（枢轴视图）排版美学。
- 多学科分类与项目看板：支持物理文件夹识别与 YAML 项目标签聚合。
- 系统级无缝集成：直接调用 Windows 资源管理器精准定位 PDF 与笔记，支持 BibTeX/Citekey 复制。

【快速上手 / Quick Start】
1. 双击运行 `lx-index.exe` 启动软件。
2. 首次运行可点击顶部左侧的文献库胶囊或右上角设置，选择本地文件夹作为文献库（Vault）。
   - 例如文献库目录结构：
     <Vault_Root>/
       ├── Optical_Computing/
       │   ├── notes/
       │   │   └── 2026_Author_Title.md
       │   └── 2026_Author_Title.pdf
3. 顶栏 Pivot 标签可在【全景画卷】、【看板概览】、【文献列表】、【文献检视】、【分类矩阵】间灵活切换。
4. 全局配置文件默认保存在 `%APPDATA%\lx-index\config.yaml`，便于跨机漫游备份。

【关于 / About】
- Designed & Developed by Lucentia Aesthetics Laboratory
- Engine: Rust 1.80+ & Slint UI Native Hardware Acceleration (Direct3D / Skia)
- Copyright (c) 2026 Lucentia. All rights reserved.
"@
Set-Content -Path (Join-Path $DistDir "README.txt") -Value $ReadmeContent -Encoding UTF8

# 4. Create ZIP archive
Write-Host "[4/4] Creating ZIP distribution package: $ZipOutput" -ForegroundColor Yellow
if (Test-Path $ZipOutput) {
    Remove-Item -Force $ZipOutput
}
Compress-Archive -Path "$DistDir\*" -DestinationPath $ZipOutput -CompressionLevel Optimal

# Output summary and hash
$FileHash = Get-FileHash -Path $ZipOutput -Algorithm SHA256
$ExeSize = (Get-Item (Join-Path $DistDir "lx-index.exe")).Length / 1MB
$ZipSize = (Get-Item $ZipOutput).Length / 1MB

Write-Host "=============================================" -ForegroundColor Green
Write-Host " Windows Production Packaging Succeeded!" -ForegroundColor Green
Write-Host " Standalone Executable: $(Join-Path $DistDir 'lx-index.exe') ($("{0:N2}" -f $ExeSize) MB)" -ForegroundColor White
Write-Host " Portable ZIP Archive : $ZipOutput ($("{0:N2}" -f $ZipSize) MB)" -ForegroundColor White
Write-Host " SHA256 Hash          : $($FileHash.Hash)" -ForegroundColor Gray
Write-Host "=============================================" -ForegroundColor Green

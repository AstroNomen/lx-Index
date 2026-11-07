<div align="center">
  <img src="assets/icon_256.png" width="128" height="128" alt="lx-Index Logo" />
  <h1>lx-Index</h1>
  <p>面向学术研究者的本地优先文献管理与沉浸式工作台。</p>

  <p>
    <a href="README.md">English</a> | <a href="README_zh.md">简体中文</a>
  </p>

  <p>
    <a href="https://github.com/AstroNomen/lx-Index/releases"><img src="https://img.shields.io/github/v/release/AstroNomen/lx-Index?style=flat-square" alt="Latest Release" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-blue.svg?style=flat-square" alt="License: GPL-3.0" /></a>
    <img src="https://img.shields.io/badge/rust-2021%20edition-orange.svg?style=flat-square" alt="Rust Edition" />
    <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey.svg?style=flat-square" alt="Platform Support" />
  </p>
</div>

---

## 概述

lx-Index 是一款原生编译的本地优先（Local-First）桌面文献管理工具。它将本地文件系统与目录结构视为**唯一的真理来源（Single Source of Truth）**，彻底摒弃专有私有数据库对文件的封装与厂商锁定。文献元数据、阅读状态与项目归属通过标准 Markdown YAML Frontmatter 进行无损同步与读写。

针对多平台采用独立解耦的 UI 架构：在 macOS 端提供符合 Apple HIG 规范的三栏式连续 HUD 界面，在 Windows 端提供深度融合 Panorama/Pivot 审美的 Modern Metro 工作台。

---

## 系统环境与硬件要求

| 操作系统平台 | 最低系统版本要求 | 支持的 CPU 架构 | 预编译二进制文件 | 说明 |
|---|---|---|---|---|
| **macOS** | macOS 11.0 (Big Sur) 及以上 | Apple Silicon (`aarch64`) | 提供 (`.dmg`) | 支持 M1/M2/M3/M4 系列芯片。Intel 架构 Mac 需自行从源码编译。 |
| **Windows** | Windows 10 (1809+ / Build 17763+) 或 Windows 11 | x86_64 (64位) | 提供 (`.zip` 便携包) | 不支持 32 位 x86 架构。 |

---

## 核心特性

- **文件系统即数据库**：直接扫描物理目录，论文 PDF 与笔记均为磁盘上的常规文件，支持随时通过第三方工具同步、备份与迁移。
- **无损 Frontmatter 同步协议**：精准增删与修改 YAML 前言元数据（状态、星标评级、领域、标签、项目、BibTeX），绝对保留正文中的个人阅读笔记与批注内容。
- **多选标签流式过滤与实时补全**：支持标签交集筛选（AND 逻辑）、按频次或字母聚合标签云，并在录入标签时提供毫秒级前缀候选自动补全。
- **双端原生人机交互设计**：
  - **macOS**：严格遵循苹果人机界面指南（Apple HIG），具备同心圆大标题栏、Cocoa AppKit 窗口系统级集成、非活动窗口灰色交通灯、系统原生菜单栏操作与分屏平铺支持。
  - **Windows**：融合 Windows 8 Metro 设计语言，连续 Pivot 枢轴栏、底部 App Bar 浮层检索与无边框 Aero Snap 窗口吸附。
- **学术工作流集成**：支持一键复制 BibTeX / Citekey 到系统剪贴板，自动为待入库 PDF 生成笔记脚手架，系统文件管理器双向跳转定位。

---

## 安装与快速上手

可前往 [GitHub Releases](https://github.com/AstroNomen/lx-Index/releases) 页面下载对应系统的预编译包。

### macOS (Apple Silicon 芯片)

1. 下载 `lx-index-1.0.0-macOS.dmg`。
2. 双击打开磁盘镜像，将 `lx Index.app` 拖移至 `/Applications` 应用程序文件夹。
3. **安全拦截提示（Gatekeeper 说明）**：由于开源社区分发版本未购买昂贵的苹果开发者商业签名证书，首次运行可能弹出“无法验证开发者”或“已损坏”提示。可通过以下方式正常启动：
   - 在访达（Finder）的“应用程序”中，**右键（或按住 Control 点击）** `lx Index.app`，并在弹出菜单中点击 **打开**；或
   - 打开“终端（Terminal）”，执行以下命令移除隔离属性：
     ```bash
     xattr -cr "/Applications/lx Index.app"
     ```

*Intel 处理器 Mac 用户说明*：官方预编译 DMG 镜像专为 Apple Silicon (`aarch64`) 编译。若您使用的是 Intel 芯片的 Mac，请参照下文从源码编译。

### Windows (x64 64位系统)

1. 下载 `lx-index-1.0.0-windows-x64.zip`。
2. 解压压缩包到任意本地文件夹。
3. 直接双击运行 `lx-index.exe`。
   - 本软件为绿色便携应用，无需安装、不写注册表、无需管理员权限。

### 从源码编译

确保本地已安装 [Rust](https://www.rust-lang.org/)（1.80 或更高版本）开发环境。

```bash
# 克隆代码仓库
git clone https://github.com/AstroNomen/lx-Index.git
cd lx-Index

# 编译优化后的 Release 二进制文件
cargo build --release
```

编译产物位于：
- **macOS**：`target/release/lx-index`
- **Windows**：`target\release\lx-index.exe`

---

## 文献库（Vault）结构与元数据协议

lx-Index 将文献组织在被称为 **Vault（文献库）** 的本地文件夹中。Vault 按照学术领域建立子目录，由原版 PDF 文件与配套的 Markdown 伴生笔记组成。

### 推荐目录结构

```text
My_Literature_Vault/
├── Condensed_Matter/
│   ├── notes/
│   │   └── 2026_Smith_Superconductivity.md
│   └── 2026_Smith_Superconductivity.pdf
├── Quantum_Optics/
│   ├── notes/
│   │   └── 2025_Doe_Photonics.md
│   └── 2025_Doe_Photonics.pdf
└── Uncatalogued_Paper.pdf
```

### Frontmatter 元数据规范

每篇伴生笔记（`notes/<filename>.md`）顶部包含被软件解析与维护的 YAML Frontmatter 数据块：

```yaml
---
title: "High-Temperature Superconductivity in Twisted Bilayer Graphene"
citekey: "Smith2026Superconductivity"
authors:
  - "Smith, John"
  - "Doe, Jane"
year: 2026
doi: "10.1038/s41586-026-00000-0"
status: 2
importance: 5
field: "Condensed_Matter"
tags:
  - "graphene"
  - "superconductivity"
  - "strongly-correlated"
projects:
  - "QuantumMaterials"
summary: "Demonstrates unconventional pairing mechanisms using angle-resolved photoemission."
bibtex: |
  @article{Smith2026Superconductivity,
    title={High-Temperature Superconductivity in Twisted Bilayer Graphene},
    author={Smith, John and Doe, Jane},
    journal={Nature},
    year={2026}
  }
---

# 个人阅读笔记

从此处开始撰写论文摘要、核心公式推导与实验见解。
关闭分隔符 `---` 以下的所有 Markdown 文本在软件读写过程中均会被严格逐字节完整保留。
```

### 文献阅读状态代码

| 状态码 (`status`) | 状态名称 | 说明 |
|---|---|---|
| `0` | 未读 (Unread) | 刚导入但未开始阅读的文献储备 |
| `1` | 略读 (Skimmed) | 已通读摘要或完成初步速览 |
| `2` | 精读 (Deep-Read) | 深入阅读并完成核心笔记撰写 |
| `3` | 废弃 (Discarded) | 与当前研究方向不符，移出常规视图 |

---

## 配置文件路径

全局软件设置（已注册的文献库路径、当前激活主题、界面语言和默认视图状态）保存在系统标准用户配置目录：

| 操作系统平台 | 配置文件路径 |
|---|---|
| macOS | `~/Library/Application Support/lx-index/config.yaml` |
| Windows | `%APPDATA%\lx-index\config.yaml` |

配置文件为标准人类可读的 YAML 格式，可直接备份或在不同工作站之间迁移。

---

## 分发打包脚本

仓库 `scripts/` 目录中提供了各平台自动化打包脚本：

- **macOS**：自动生成 `.app` 应用程序包并构建压缩的 UDZO `.dmg` 磁盘镜像：
  ```bash
  bash scripts/package_macos.sh
  ```
- **Windows**：自动编译 Release 二进制文件并归档为独立的便携 `.zip` 压缩包：
  ```powershell
  powershell -ExecutionPolicy Bypass -File scripts\package_windows.ps1
  ```

---

## 参与贡献

欢迎提交 Issue 与 Pull Request。向 `main` 分支提交 PR 前，请确保代码通过检查与单元测试：

```bash
cargo check
cargo test
```

---

## 开源协议

本项目采用 **GNU General Public License v3.0** (GPL-3.0) 协议开源。详细条款请参阅 [LICENSE](LICENSE) 文件。

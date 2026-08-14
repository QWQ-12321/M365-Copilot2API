# M365 Copilot2API — 桌面应用构建指南

将 M365-Copilot2API 打包为普通用户可直接安装的原生桌面应用。

## 产出物

| 平台 | 产物 |
|------|------|
| Windows | `.msi` / `.exe` 安装包（NSIS，中英双语） |
| macOS | `.app` / `.dmg` |
| Linux | `.deb` / `.AppImage` |

## 方式一：GitHub Actions 自动构建（推荐，零本地依赖）

不需要在本地安装任何工具。只需将代码推送到 GitHub，CI 会自动完成全部构建并发布到 Releases。

### 触发方式

```bash
# 1. 推送 tag 触发
git tag app-v0.2.0
git push origin app-v0.2.0

# 2. 或在 GitHub 网页上手动触发
# Actions → Tauri Desktop App Release → Run workflow
```

CI 会：
1. 自动安装 Go + Rust + 系统依赖
2. 编译 Go sidecar 并放入 `binaries/`
3. 生成图标
4. 执行 `cargo tauri build`
5. 将安装包上传到 GitHub Releases

构建完成后在 **Releases** 页面下载对应平台的安装包。

## 方式二：本地构建（需要安装工具）

### 前置依赖

#### Windows
- Go 1.21+：https://go.dev/dl/
- Rust：https://rustup.rs
- WebView2（Windows 10 1803+ 自带，无需手动安装）

#### macOS
- Xcode Command Line Tools：`xcode-select --install`
- Go 1.21+
- Rust

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  python3-pip
```

### 构建步骤

#### 第一步：生成图标（纯 Go，无需 Python）

```bash
go run scripts/generate_icons.go
```

#### 第二步：编译 Go sidecar

```bash
# Windows
scripts\build-sidecar.bat windows-x86_64

# Linux / macOS
bash scripts/build-sidecar.sh windows-x86_64
```

#### 第三步：编译 Tauri 应用

```bash
cd src-tauri
cargo tauri build
```

产物输出到 `src-tauri/target/release/bundle/`：
- Windows: `nsis/*.exe`
- macOS: `dmg/*.dmg`
- Linux: `deb/*.deb`, `appimage/*.AppImage`

## 用户安装后会发生什么

1. 启动应用 → 系统托盘出现蓝色图标
2. Go sidecar 自动在后台启动（端口 4141，冲突时自动递增）
3. WebView 窗口显示管理界面（`http://127.0.0.1:{port}/`）
4. 用户可最小化到托盘，随时通过托盘菜单打开管理界面

## 目录结构

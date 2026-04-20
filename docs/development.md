<div align="right">
  <span>[<a href="./development.en.md">English</a>]</span>
  <span>[<a href="./development.md">简体中文</a>]</span>
</div>

# 开发与构建

## 环境要求

- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/) 9+
- [Rust](https://www.rust-lang.org/tools/install) stable

macOS 如需读取“正在播放”媒体信息，额外安装：

```bash
brew install nowplaying-cli
```

Linux 建议安装：

```bash
# X11 前台窗口读取
sudo apt install x11-utils

# MPRIS 媒体信息读取
sudo apt install playerctl
```

## 本地开发

安装依赖：

```bash
pnpm install
```

启动桌面开发环境：

```bash
pnpm tauri dev
```

仅启动前端开发服务器：

```bash
pnpm dev
```

## 移动端开发

### Android

```bash
pnpm tauri android init --ci
pnpm tauri android dev
```

如果初始化失败并提示 `Android NDK not found`，请先在 Android Studio 的 SDK Manager 安装 NDK，并设置 `NDK_HOME` 后重试。

### iOS

iOS 需在 `macOS + Xcode` 环境中执行：

```bash
pnpm tauri ios init --ci
pnpm tauri ios dev
pnpm tauri ios build
```

## 构建

构建桌面应用：

```bash
pnpm tauri build
```

构建前端静态资源：

```bash
pnpm build
```

构建产物默认位于：

```text
src-tauri/target/release/bundle/
```

## CI

仓库中的 [`.github/workflows/tauri-ci.yml`](../.github/workflows/tauri-ci.yml) 会执行基础检查，并产出：

- Windows 安装包
- macOS DMG
- Linux AppImage / deb
- Android 构建产物

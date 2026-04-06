## Waken-Wa Client

适用于 [Waken-Wa](https://github.com/MoYoez/Waken-Wa) 的客户端：提供连接配置、活动手动上报、活动流查看、灵感内容发布，并在桌面端支持后台实时同步与系统托盘常驻能力。

- 此版本为 `Tauri + Vue + Rust` 的多端版本（Desktop + Mobile）。
- 支持导入 Waken-Wa 后台的一键接入配置，也支持直接复用本机已有的 `waken-wa-reporter` 配置。
- 当前支持：
  - **桌面端（Windows/macOS）**：支持后台实时同步、托盘驻留与平台能力自检。
  - **桌面端（Linux）**：支持后台实时同步与平台能力自检（前台窗口在 Wayland 下需桥接，见下文）。
  - **移动端（Android/iOS）**：支持手机/平板自适应布局，默认关闭后台实时上报能力，仅保留手动活动提交与灵感相关能力。

> 🤔 目前为 早期 的 构建，由 AI 编写的 MVP 实现，可能存在不少的bug :(



## 主要功能

>在 MacOS 上可能会遇到无法运行的问题，请使用 xattr -c <应用名> 来解决此问题

- 通过图形界面完成站点地址、Token、设备名称、轮询/心跳间隔等设置
- 手动提交当前活动状态到 Waken-Wa 后端
- 在后台持续采集前台应用、窗口标题与媒体信息并自动上报
- 查看实时同步日志、最近心跳、最近错误与当前活动快照
- 导入后台复制的一键接入配置（Base64 / JSON）
- 自动发现并导入本机现有 `waken-wa-reporter` 配置
- 关闭主窗口后最小化到系统托盘，继续在后台驻留
- 提供平台能力自检，检查前台应用、窗口标题、媒体采集是否正常

## 环境要求

- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/) 9+
- [Rust](https://www.rust-lang.org/tools/install) stable

macOS 如需读取“正在播放”媒体信息，额外需要安装：

```bash
brew install nowplaying-cli
```

Linux 端建议安装：

```bash
# 读取前台窗口（X11）
sudo apt install x11-utils

# 读取媒体信息（MPRIS）
sudo apt install playerctl
```

> MacOS 26 仅 2.0.0 支持

## 开发与运行

首次拉取后安装依赖：

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

初始化 Android 目标：

```bash
pnpm tauri android init --ci
```

如果初始化失败并提示 `Android NDK not found`，请先在 Android Studio 的 SDK Manager 安装 NDK，并设置 `NDK_HOME` 后重试。

启动 Android 开发模式：

```bash
pnpm tauri android dev
```

### iOS

iOS 需要在 **macOS + Xcode** 环境中执行。当前 Windows 环境不直接产出 iOS 包。

在 macOS 上可使用：

```bash
pnpm tauri ios init --ci
pnpm tauri ios dev
pnpm tauri ios build
```

## 构建

在项目根目录执行：

```bash
pnpm tauri build
```

前端静态构建：

```bash
pnpm build
```

构建产物位于 `src-tauri/target/release/bundle/` 下，具体格式取决于当前操作系统。

## 使用

首次打开应用时，会进入引导流程。你可以：

- 直接手动填写站点地址、API Token、设备名称等配置
- 粘贴从 Waken-Wa 后台复制的一键接入配置（支持 Base64 / JSON）
- 如果本机存在 `waken-wa-reporter` 的配置文件，应用会提示一键导入

配置完成后，可在应用内使用以下区域：

| 页面 | 说明 |
|------|------|
| **概览** | 查看当前连接状态、后台同步状态与最近活动信息 |
| **灵感** | 撰写并发布内容到 Waken-Wa |
| **活动同步** | 手动编辑并提交当前活动状态 |
| **实时同步** | 查看后台自动上报日志、当前活动、心跳与错误 |
| **设置** | 管理连接、设备、同步参数，并执行平台能力自检 |

## 后台同步

在“设置”页可开启后台同步。应用会周期性采集当前前台应用，并在支持的平台上附带媒体信息，然后向：

```text
{baseUrl}/api/activity
```

发送活动数据。

默认参数与当前代码一致：

- 轮询间隔：`2000ms`
- 心跳间隔：`60000ms`

含义与 CLI 版本基本一致：

| 概念 | 含义 |
|------|------|
| **轮询间隔（poll）** | 每隔多久检查一次前台应用、窗口标题和媒体信息是否相对上一次上报发生变化；有变化则立即上报 |
| **心跳间隔（heartbeat）** | 若活动内容未变化，仍按固定间隔重复上报一次，用于刷新在线状态；设为 `0` 表示关闭 |

注意：

- 当前桌面版要求轮询间隔至少为 `1000ms`
- 心跳间隔可为 `0`
- “启动后自动开启后台同步”开启后，下次打开应用会自动启动同步

## 设备待审核（HTTP 202）

当站点关闭“自动接收新设备”且当前设备尚未通过审核时，后台可能返回 **202**。

桌面客户端会：

- 在实时同步日志中记录“设备待审核”
- 在界面中弹出待审核提示
- 展示后台返回的 `approvalUrl`（若有）
- 保持后台同步运行，等待再次上报成功后自动恢复正常状态

## 系统托盘

桌面版包含系统托盘能力：

- 关闭主窗口时不会直接退出，而是隐藏到托盘
- 单击托盘图标可重新打开主界面
- 托盘菜单支持“打开主界面”“隐藏到后台”“退出应用”

如果你希望客户端长期运行，推荐直接保持托盘驻留。

## 配置方式

桌面版主要通过应用内界面保存配置，不依赖 CLI 参数。

当前可配置内容包括：

| key | 说明 |
|------|------|
| `baseUrl` | 后端根地址 |
| `apiToken` | API Token |
| `generatedHashKey` | 稳定设备身份标识；首次使用时自动生成 |
| `device` | 展示用设备名 |
| `deviceType` | 设备类型，默认 `desktop` |
| `pushMode` | 推送模式，默认 `realtime` |
| `pollIntervalMs` | 轮询间隔（毫秒，需 `>=1000`） |
| `heartbeatIntervalMs` | 心跳间隔（毫秒，`0` 表示关闭） |
| `reporterMetadataJson` | 附加到活动上报中的 JSON 元数据 |
| `reporterEnabled` | 是否在应用启动后自动开启后台同步 |

## 本地配置文件

应用自身状态会保存在操作系统用户配置目录下的 `client-state.json` 中。

此外，应用会尝试发现并导入 CLI Reporter 的配置文件：

- Windows：`%APPDATA%\waken-wa\config.json`
- macOS：`$XDG_CONFIG_HOME/waken-wa/config.json`（未设置时通常位于 `~/.config/waken-wa/config.json`）

导入时会复用以下字段：

- `base_url`
- `api_token`
- `device_name`
- `generated_hash_key`
- `poll_interval_ms`
- `heartbeat_interval_ms`
- `metadata`

## 平台说明

| 平台 | 状态 |
|------|------|
| **Windows** | 支持前台应用、窗口标题、媒体读取与后台同步 |
| **macOS** | 支持前台应用、媒体读取与后台同步；窗口标题当前暂未实现 |
| **Linux** | 支持后台同步与平台自检；X11 支持前台窗口采集，Wayland 需桥接文件；媒体采集依赖 `playerctl` |
| **Android** | 支持手机/平板 UI、手动活动同步、灵感相关能力；不提供后台实时同步 |
| **iOS** | 支持手机/平板 UI、手动活动同步、灵感相关能力；需在 macOS 上构建 |
| **其它平台** | 当前默认不支持实时采集能力 |

macOS 说明：

- 媒体读取依赖 `nowplaying-cli`
- 某些能力可能依赖系统“辅助功能”或“自动化”授权
- 可在“设置”页运行“检查平台能力”查看详细提示

Linux 说明：

- X11 会话通过 `xprop` 读取前台应用和窗口标题
- Wayland 会话无法直接读取全局前台窗口，需接入桥接文件机制
- 媒体读取通过 `playerctl`（MPRIS）
- Wayland 桥接规范见：[`docs/linux-wayland-foreground-bridge.md`](docs/linux-wayland-foreground-bridge.md)

## 上报说明

客户端会向 `{baseUrl}/api/activity` 发送 JSON，请求体核心字段包括：

- `generatedHashKey`
- `device`
- `process_name`
- `process_title`
- `device_type`
- `push_mode`
- `metadata`

其中：

- `metadata.source` 默认为 `waken-wa-desktop`
- 若系统存在正在播放的媒体，`metadata.media` 中会附带标题、歌手、专辑等信息
- 请求使用 Bearer Token 认证

## CI 构建产物

本仓库的 [`.github/workflows/tauri-ci.yml`](.github/workflows/tauri-ci.yml) 会在 push / PR 时执行检查，并产出：

- Windows bundle
- macOS bundle
- Linux bundle（AppImage / deb）
- Android APK/AAB（debug, no-sign）
- iOS no-sign 构建产物（基于 `src-tauri/tauri.conf.ci.json`）

产物以 GitHub Actions Artifacts 的形式上传。

## License

MIT

<div align="right">
  <span>[<a href="./README_EN.md">English</a>]</span>
  <span>[<a href="./README.md">简体中文</a>]</span>
</div>

> 把今天做过的事，安静地同步给自己。

<p align="center">
  <img src="./src/assets/Logo.png" alt="Waken-Wa Reporter Client" width="96" height="96">
  <h2 align="center">Waken-Wa Reporter Client</h2>
  <p align="center">
    <img alt="License" src="https://img.shields.io/github/license/MoYoez/waken-wa-reporter" />
    <img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/MoYoez/waken-wa-reporter/tauri-ci.yml" />
  </p>
</p>

适用于 [Waken-Wa](https://github.com/MoYoez/Waken-Wa) 的跨端客户端，基于 `Tauri 2 + Vue 3 + Rust` 构建，提供连接配置、活动上报、后台实时同步、灵感发布、Discord Rich Presence 同步，以及桌面端托盘驻留与开机自启动能力。

当前文档提供简体中文和 English 两个版本；应用界面本身也支持中英文切换。

## 功能特点

1. 图形化接入 Waken-Wa，支持站点地址、Token、设备名、轮询/心跳间隔等配置
2. 支持导入后台复制的一键接入配置，也可复用本机已有的 `waken-wa-reporter` 配置
3. 支持手动提交活动状态，并在桌面端持续采集前台应用、窗口标题与媒体信息
4. 支持查看实时同步日志、最近心跳、最近错误与当前活动快照
5. 支持灵感内容撰写与发布
6. 支持 Discord Rich Presence 同步，读取当前客户端的 public feed 活动
7. 支持中英文界面切换；前端文案即时切换，后端文案重启后生效
8. 提供平台能力自检、系统托盘驻留与开机自启动等桌面能力

## 平台支持

| 平台 | 状态 |
| --- | --- |
| Windows / macOS | 支持后台实时同步、托盘驻留、开机自启动、平台自检、手动活动上报、灵感发布 |
| Linux | 支持后台实时同步、平台自检、手动活动上报、灵感发布；Wayland 适配 GNOME / KDE，媒体信息依赖 `playerctl` |
| Android / iOS | 提供手机与平板自适应 UI，支持手动活动上报与灵感功能；默认不提供后台实时同步、托盘与 Discord 同步 |

## 安装

请直接前往 [Releases](https://github.com/MoYoez/waken-wa-reporter/releases) 下载对应平台的安装包或构建产物。

## 文档

- 开发与构建说明：[`docs/development.md`](./docs/development.md)
- 配置与使用说明：[`docs/configuration.md`](./docs/configuration.md)
- Linux Wayland 前台窗口桥接说明：[`docs/linux-wayland-foreground-bridge.md`](./docs/linux-wayland-foreground-bridge.md)
- 后端错误码与前端文案映射建议：[`docs/backend-i18n.md`](./docs/backend-i18n.md)

## License

MIT

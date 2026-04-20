<div align="right">
  <span>[<a href="./README_EN.md">English</a>]</span>
  <span>[<a href="./README.md">简体中文</a>]</span>
</div>

> Quietly sync what you did today back to yourself.

<p align="center">
  <img src="./src/assets/Logo.png" alt="Waken-Wa Reporter Client" width="96" height="96">
  <h2 align="center">Waken-Wa Reporter Client</h2>
  <p align="center">
    <img alt="License" src="https://img.shields.io/github/license/MoYoez/waken-wa-reporter" />
    <img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/MoYoez/waken-wa-reporter/tauri-ci.yml" />
  </p>
</p>

This is a cross-platform client for [Waken-Wa](https://github.com/MoYoez/Waken-Wa), built with `Tauri 2 + Vue 3 + Rust`. It covers connection setup, manual activity reporting, realtime background sync, inspiration publishing, Discord Rich Presence sync, and desktop features such as system tray persistence and launch on startup.

Both Simplified Chinese and English are available in the docs, and the app UI itself also supports switching between the two languages.

## Highlights

1. Graphical setup for Waken-Wa, including site URL, token, device name, polling, and heartbeat settings
2. Import the quick-connect payload copied from the backend, or reuse an existing local `waken-wa-reporter` config
3. Manually submit activities and, on desktop, continuously capture the foreground app, window title, and media info
4. Review realtime sync logs, recent heartbeats, recent errors, and the current activity snapshot
5. Write and publish inspiration content
6. Sync this client's public feed activity to Discord Rich Presence
7. Switch between Chinese and English; frontend copy updates immediately, backend copy updates after restart
8. Desktop utilities such as platform self-checks, system tray persistence, and launch on startup

## Platform Support

| Platform | Status |
| --- | --- |
| Windows / macOS | Supports realtime background sync, system tray, launch on startup, platform self-checks, manual activity reporting, and inspiration publishing |
| Linux | Supports realtime background sync, platform self-checks, manual activity reporting, and inspiration publishing; Wayland is adapted for GNOME / KDE, and media capture depends on `playerctl` |
| Android / iOS | Provides a phone and tablet friendly UI with manual activity reporting and inspiration features; realtime background sync, tray, and Discord sync are disabled by default |

## Installation

Download the package for your platform from [Releases](https://github.com/MoYoez/waken-wa-reporter/releases).

## Docs

- Development and build guide: [`docs/development.en.md`](./docs/development.en.md)
- Configuration and usage guide: [`docs/configuration.en.md`](./docs/configuration.en.md)
- Linux Wayland foreground bridge notes: [`docs/linux-wayland-foreground-bridge.en.md`](./docs/linux-wayland-foreground-bridge.en.md)
- Backend error-code and frontend copy mapping notes: [`docs/backend-i18n.en.md`](./docs/backend-i18n.en.md)

## License

MIT

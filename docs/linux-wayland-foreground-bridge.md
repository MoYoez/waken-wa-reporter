<div align="right">
  <span>[<a href="./linux-wayland-foreground-bridge.en.md">English</a>]</span>
  <span>[<a href="./linux-wayland-foreground-bridge.md">简体中文</a>]</span>
</div>

# Linux Wayland 前台窗口桥接

在 Wayland 下，普通桌面应用无法直接读取全局前台窗口。  
Waken-Wa Reporter Client 当前采用两层内建策略读取前台窗口信息：

1. GNOME：直接调用 `Focused Window D-Bus`
2. KDE Plasma：直接调用 `kdotool`

## 内建 Wayland 适配

### GNOME

优先支持：

- GNOME Shell 扩展 [Focused Window D-Bus](https://extensions.gnome.org/extension/5592/focused-window-d-bus/)
- `gdbus`

客户端会直接调用：

```bash
gdbus call --session --dest org.gnome.Shell --object-path /org/gnome/shell/extensions/FocusedWindow --method org.gnome.shell.extensions.FocusedWindow.Get
```

并从返回值中提取：

- `wm_class_instance` / `wm_class` 作为 `processName`
- `title` 作为 `processTitle`

### KDE Plasma

优先支持：

- [`kdotool`](https://github.com/jinliu/kdotool)

客户端会直接调用：

- `kdotool getactivewindow`
- `kdotool getwindowclassname <window-id>`
- `kdotool getwindowname <window-id>`

并映射为：

- `getwindowclassname` -> `processName`
- `getwindowname` -> `processTitle`

## 降级策略

- X11 会话：客户端优先用 `xprop` 直接读取前台窗口
- Wayland 会话：优先尝试 `Focused Window D-Bus` / `kdotool`

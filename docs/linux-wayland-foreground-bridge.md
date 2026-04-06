# Linux Wayland 前台窗口桥接

在 Wayland 下，普通桌面应用无法直接读取全局前台窗口。  
Waken-Wa Web Client 支持通过“桥接文件”读取由桌面侧组件（GNOME Shell 扩展 / KWin 脚本）写入的前台窗口信息。

## 桥接文件路径

```text
$XDG_RUNTIME_DIR/waken-wa/foreground.json
```

例如：

```text
/run/user/1000/waken-wa/foreground.json
```

## JSON 格式

```json
{
  "processName": "firefox",
  "processTitle": "Rust docs - Mozilla Firefox"
}
```

字段要求：

- `processName`: 必填，前台应用标识（建议使用稳定的 app id / class / desktop file 名）
- `processTitle`: 选填，前台窗口标题

## 刷新频率

- 建议在焦点窗口变化时立即写入
- 客户端会将“超过 15 秒未更新”的桥接文件视为过期

## GNOME Shell 扩展建议

- 监听 `global.display` 的 `notify::focus-window` 或 `focus-window` 变化
- 通过 `Shell.WindowTracker.get_default().get_window_app(window)` 获取 app
- 写入上面的 JSON 到桥接文件路径

## KDE Plasma / KWin 脚本建议

- 监听 `workspace.windowActivated`
- 从 `window` 提取 `desktopFileName` / `resourceClass` / `caption`
- 写入上面的 JSON 到桥接文件路径

## 降级策略

- X11 会话：客户端优先用 `xprop` 直接读取前台窗口
- Wayland 会话：优先读取桥接文件；若无桥接则会返回“前台应用采集失败”

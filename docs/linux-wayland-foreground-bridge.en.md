<div align="right">
  <span>[<a href="./linux-wayland-foreground-bridge.en.md">English</a>]</span>
  <span>[<a href="./linux-wayland-foreground-bridge.md">简体中文</a>]</span>
</div>

# Linux Wayland Foreground Window Bridge

On Wayland, regular desktop apps cannot directly read the global foreground window.  
Waken-Wa Reporter Client currently uses two built-in strategies to resolve foreground window data:

1. GNOME: call `Focused Window D-Bus`
2. KDE Plasma: call `kdotool`

## Built-in Wayland Adapters

### GNOME

Preferred support:

- GNOME Shell extension [Focused Window D-Bus](https://extensions.gnome.org/extension/5592/focused-window-d-bus/)
- `gdbus`

The client calls:

```bash
gdbus call --session --dest org.gnome.Shell --object-path /org/gnome/shell/extensions/FocusedWindow --method org.gnome.shell.extensions.FocusedWindow.Get
```

Then extracts:

- `wm_class_instance` / `wm_class` as `processName`
- `title` as `processTitle`

### KDE Plasma

Preferred support:

- [`kdotool`](https://github.com/jinliu/kdotool)

The client calls:

- `kdotool getactivewindow`
- `kdotool getwindowclassname <window-id>`
- `kdotool getwindowname <window-id>`

Then maps:

- `getwindowclassname` -> `processName`
- `getwindowname` -> `processTitle`

## Fallback Strategy

- On X11 sessions, the client prefers direct foreground-window reads via `xprop`
- On Wayland sessions, it first tries `Focused Window D-Bus` / `kdotool`

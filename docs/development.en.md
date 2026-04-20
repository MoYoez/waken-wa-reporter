<div align="right">
  <span>[<a href="./development.en.md">English</a>]</span>
  <span>[<a href="./development.md">简体中文</a>]</span>
</div>

# Development and Build

## Requirements

- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/) 9+
- [Rust](https://www.rust-lang.org/tools/install) stable

On macOS, install this extra dependency if you want "Now Playing" media capture:

```bash
brew install nowplaying-cli
```

On Linux, these packages are recommended:

```bash
# Foreground window capture on X11
sudo apt install x11-utils

# MPRIS media capture
sudo apt install playerctl
```

## Local Development

Install dependencies:

```bash
pnpm install
```

Start the desktop development environment:

```bash
pnpm tauri dev
```

Run only the frontend dev server:

```bash
pnpm dev
```

## Mobile Development

### Android

```bash
pnpm tauri android init --ci
pnpm tauri android dev
```

If initialization fails with `Android NDK not found`, install the NDK from Android Studio's SDK Manager, set `NDK_HOME`, and try again.

### iOS

iOS must be built on `macOS + Xcode`:

```bash
pnpm tauri ios init --ci
pnpm tauri ios dev
pnpm tauri ios build
```

## Build

Build the desktop app:

```bash
pnpm tauri build
```

Build static frontend assets:

```bash
pnpm build
```

Bundled outputs are generated under:

```text
src-tauri/target/release/bundle/
```

## CI

The workflow at [`.github/workflows/tauri-ci.yml`](../.github/workflows/tauri-ci.yml) runs core checks and produces:

- Windows installers
- macOS DMG bundles
- Linux AppImage / deb packages
- Android build artifacts

<div align="right">
  <span>[<a href="./configuration.en.md">English</a>]</span>
  <span>[<a href="./configuration.md">简体中文</a>]</span>
</div>

# Configuration and Usage

## First-Time Setup

On first launch, the app walks through onboarding. You can:

- Enter the site URL, API token, device name, and related settings manually
- Paste the quick-connect payload copied from the backend in Base64 or JSON form
- Import an existing local `waken-wa-reporter` config

After setup, the main areas are:

| Page | Description |
| --- | --- |
| Overview | Shows connection status, background sync status, Discord status, and recent activity summary |
| Inspiration | Write and publish content to Waken-Wa |
| Activity | Manually edit and submit the current activity |
| Realtime | Inspect automatic reporting logs, current activity, heartbeats, and errors |
| Settings | Manage connection, language, background sync, Discord sync, launch on startup, and platform checks |

## Realtime Background Sync

Background sync can be enabled in Settings. The client periodically captures the current foreground app and, on supported platforms, adds media info before sending activity data to:

```text
{baseUrl}/api/activity
```

The current default values are:

- Poll interval: `2000ms`
- Heartbeat interval: `60000ms`

Notes:

- Polling must be at least `1000ms`
- Heartbeat may be set to `0`
- If "auto-start background sync on launch" is enabled, sync starts automatically next time the app opens

## Device Pending Approval

When the site does not auto-accept new devices and the current device has not been approved yet, the backend may return HTTP `202`. In that case, the client will:

- Record the pending state in the realtime log
- Show the pending approval hint and `approvalUrl` in the UI
- Keep the sync runtime alive and recover automatically after approval

## Discord Rich Presence

Desktop builds can sync this client's public feed activity to Discord Rich Presence. Before starting it, fill in these fields in Settings:

- Site URL
- `Discord Application ID`

You can also enable auto-start so the client tries to connect to Discord Desktop on the next launch.

## Local State and Config Files

- App state is stored as `client-state.json` in the system app config directory
- The client can import an existing CLI reporter config from `waken-wa/config.json` in the user config directory

## Notes

- On macOS, if the app cannot be opened normally, try `xattr -c <AppName>`
- Desktop builds are designed to stay in the background; closing the main window minimizes the app to the system tray

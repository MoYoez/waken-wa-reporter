import {
  disable as disableAutostartPlugin,
  enable as enableAutostartPlugin,
  isEnabled as isAutostartEnabledPlugin,
} from "@tauri-apps/plugin-autostart";

export async function isAutostartEnabled(): Promise<boolean> {
  return isAutostartEnabledPlugin();
}

export async function setAutostartEnabled(enabled: boolean): Promise<void> {
  if (enabled) {
    await enableAutostartPlugin();
    return;
  }

  await disableAutostartPlugin();
}

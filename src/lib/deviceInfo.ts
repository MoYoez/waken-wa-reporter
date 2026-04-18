import { getBatteryInfo, getDeviceInfo } from "tauri-plugin-device-info-api";

import { translate } from "../i18n";

export interface BatterySnapshot {
  levelPercent: number;
  charging: boolean;
}

export async function readBatterySnapshot(): Promise<BatterySnapshot> {
  const battery = await getBatteryInfo();
  const level = battery.level;

  if (typeof level !== "number" || !Number.isFinite(level)) {
    throw new Error(translate("deviceInfo.batteryUnavailable"));
  }

  return {
    levelPercent: Math.max(0, Math.min(100, Math.round(level))),
    charging: Boolean(battery.isCharging),
  };
}

export async function readDeviceName(): Promise<string> {
  const info = await getDeviceInfo();
  const candidates = [info.device_name, info.model, info.manufacturer]
    .map((value) => String(value ?? "").trim())
    .filter(Boolean);

  if (candidates.length === 0) {
    throw new Error(translate("deviceInfo.deviceNameUnavailable"));
  }

  return candidates[0];
}

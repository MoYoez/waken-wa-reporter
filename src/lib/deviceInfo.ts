import { getBatteryInfo, getDeviceInfo } from "tauri-plugin-device-info-api";

export interface BatterySnapshot {
  levelPercent: number;
  charging: boolean;
}

export async function readBatterySnapshot(): Promise<BatterySnapshot> {
  const battery = await getBatteryInfo();
  const level = battery.level;

  if (typeof level !== "number" || !Number.isFinite(level)) {
    throw new Error("当前设备未返回可用的电量信息。");
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
    throw new Error("当前设备未返回可用的设备名称。");
  }

  return candidates[0];
}

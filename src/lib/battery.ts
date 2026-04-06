export interface BatterySnapshot {
  levelPercent: number;
  charging: boolean;
}

interface BatteryManagerLike {
  level: number;
  charging: boolean;
}

interface NavigatorWithBattery extends Navigator {
  getBattery?: () => Promise<BatteryManagerLike>;
}

export async function readBatterySnapshot(): Promise<BatterySnapshot> {
  const batteryApi = (navigator as NavigatorWithBattery).getBattery;
  if (!batteryApi) {
    throw new Error("当前运行环境未开放电量读取接口（navigator.getBattery）。");
  }

  const battery = await batteryApi();
  return {
    levelPercent: Math.round(battery.level * 100),
    charging: battery.charging,
  };
}

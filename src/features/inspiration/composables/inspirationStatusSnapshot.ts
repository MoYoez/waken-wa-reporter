import type { Ref } from "vue";

import type { ActivityFeedItem, ClientConfig } from "@/types";
import {
  activityBatteryPercent,
  activityLineText,
} from "@/features/inspiration/composables/inspirationWorkspaceShared";

type TranslateFn = (key: string, params?: Record<string, unknown>) => string;

interface UseInspirationStatusSnapshotOptions {
  config: ClientConfig;
  statusBatteryPercent: Ref<number | null>;
  statusSnapshotDeviceName: Ref<string>;
  t: TranslateFn;
}

export function useInspirationStatusSnapshot(options: UseInspirationStatusSnapshotOptions) {
  function snapshotWithDeviceAndBattery(base: string, device: string, battery: string) {
    return options.t("inspiration.snapshot.withDeviceAndBattery", {
      base,
      device,
      battery,
    });
  }

  function snapshotWithDevice(base: string, device: string) {
    return options.t("inspiration.snapshot.withDevice", { base, device });
  }

  function snapshotWithBattery(base: string, battery: string) {
    return options.t("inspiration.snapshot.withBattery", { base, battery });
  }

  function buildManualSnapshot(input: string, includeDeviceInfo: boolean) {
    const base = input.trim();
    if (!base) {
      return "";
    }
    if (!includeDeviceInfo) {
      return base;
    }

    const deviceName =
      options.statusSnapshotDeviceName.value.trim() || options.config.device.trim();
    const batteryPart =
      typeof options.statusBatteryPercent.value === "number"
        ? `${options.statusBatteryPercent.value}%`
        : "";

    if (deviceName && batteryPart) {
      return snapshotWithDeviceAndBattery(base, deviceName, batteryPart);
    }
    if (deviceName) {
      return snapshotWithDevice(base, deviceName);
    }
    if (batteryPart) {
      return snapshotWithBattery(base, batteryPart);
    }
    return base;
  }

  function buildSnapshotText(item: ActivityFeedItem, includeDeviceInfo: boolean) {
    const base = activityLineText(item, options.t("inspiration.common.unnamedActivity")).trim();
    if (!base) {
      return "";
    }
    if (!includeDeviceInfo) {
      return base;
    }

    const deviceName = String(item.device ?? "").trim();
    if (!deviceName) {
      return base;
    }

    const battery = activityBatteryPercent(item);
    if (typeof battery === "number") {
      return snapshotWithDeviceAndBattery(base, deviceName, `${battery}%`);
    }

    return snapshotWithDevice(base, deviceName);
  }

  return {
    buildManualSnapshot,
    buildSnapshotText,
  };
}

import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getPublicActivityFeed } from "@/lib/api";
import { readBatterySnapshot } from "@/lib/deviceInfo";
import type { ActivityFeedItem, ClientCapabilities, ClientConfig } from "@/types";
import type { ActivitySelectOption } from "@/features/inspiration/types";
import {
  activityBatteryPercent,
  activityLineText,
} from "@/features/inspiration/composables/inspirationWorkspaceShared";

interface InspirationStatusOptions {
  config: ClientConfig;
  capabilities: ClientCapabilities;
  draftStore: {
    statusSnapshotInput: string;
    statusSnapshotDeviceName: string;
    selectedActivityKey: string;
    attachCurrentStatus: boolean;
    attachStatusIncludeDeviceInfo: boolean;
    patchDraft: (payload: Record<string, unknown>) => void;
  };
  apiErrorDetail: (
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) => string;
}

export function useInspirationStatus(options: InspirationStatusOptions) {
  const { t } = useI18n();

  const statusSnapshotInput = ref(options.draftStore.statusSnapshotInput);
  const statusSnapshotDeviceName = ref(options.draftStore.statusSnapshotDeviceName);
  const selectedActivityKey = ref(options.draftStore.selectedActivityKey);
  const attachCurrentStatus = ref(options.draftStore.attachCurrentStatus);
  const attachStatusIncludeDeviceInfo = ref(options.draftStore.attachStatusIncludeDeviceInfo);
  const statusBatteryPercent = ref<number | null>(null);
  const activityOptions = ref<ActivitySelectOption[]>([]);
  const activityLoading = ref(false);
  const activityLoadError = ref("");

  const mobileRuntime = computed(() => !options.capabilities.realtimeReporter);
  const selectedActivityOption = computed(() =>
    activityOptions.value.find((item) => item.value === selectedActivityKey.value) ?? null,
  );
  const selectedSnapshotPreview = computed(() => {
    if (!attachCurrentStatus.value) {
      return "";
    }
    if (mobileRuntime.value) {
      return buildManualSnapshot(statusSnapshotInput.value, attachStatusIncludeDeviceInfo.value);
    }
    return selectedActivityOption.value?.snapshot ?? "";
  });

  function snapshotWithDeviceAndBattery(base: string, device: string, battery: string) {
    return t("inspiration.snapshot.withDeviceAndBattery", {
      base,
      device,
      battery,
    });
  }

  function snapshotWithDevice(base: string, device: string) {
    return t("inspiration.snapshot.withDevice", { base, device });
  }

  function snapshotWithBattery(base: string, battery: string) {
    return t("inspiration.snapshot.withBattery", { base, battery });
  }

  function buildManualSnapshot(input: string, includeDeviceInfo: boolean) {
    const base = input.trim();
    if (!base) {
      return "";
    }
    if (!includeDeviceInfo) {
      return base;
    }

    const deviceName = statusSnapshotDeviceName.value.trim() || options.config.device.trim();
    const batteryPart =
      typeof statusBatteryPercent.value === "number"
        ? `${statusBatteryPercent.value}%`
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
    const base = activityLineText(item, t("inspiration.common.unnamedActivity")).trim();
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

  function toActivityOptions(items: ActivityFeedItem[], group: "active" | "recent") {
    return items
      .map((item, index) => {
        const idPart = String(item.id ?? `${item.processName ?? "item"}-${index}`);
        const snapshot = buildSnapshotText(item, attachStatusIncludeDeviceInfo.value);
        const device = String(item.device ?? "").trim();
        const prefix = t(`inspiration.activityGroup.${group}`);
        return {
          value: `${group}:${idPart}`,
          label: device ? `${prefix} · ${snapshot} · ${device}` : `${prefix} · ${snapshot}`,
          snapshot,
          group,
          item,
          deviceName: device,
        } satisfies ActivitySelectOption;
      })
      .filter((item) => item.snapshot.trim().length > 0);
  }

  async function loadActivityOptions() {
    if (!options.config.baseUrl.trim()) {
      activityOptions.value = [];
      activityLoadError.value = "";
      return;
    }

    activityLoading.value = true;
    activityLoadError.value = "";

    const result = await getPublicActivityFeed(options.config);
    activityLoading.value = false;

    if (!result.success || !result.data) {
      activityLoadError.value = options.apiErrorDetail(
        result.error,
        t("inspiration.notify.activityLoadFailed"),
      );
      activityOptions.value = [];
      return;
    }

    const activeStatuses = Array.isArray(result.data.activeStatuses)
      ? result.data.activeStatuses
      : [];
    const recentActivities = Array.isArray(result.data.recentActivities)
      ? result.data.recentActivities
      : [];

    const optionsList = [
      ...toActivityOptions(activeStatuses as ActivityFeedItem[], "active"),
      ...toActivityOptions((recentActivities as ActivityFeedItem[]).slice(0, 20), "recent"),
    ];

    activityOptions.value = optionsList.filter(
      (item, index, all) => index === all.findIndex((candidate) => candidate.value === item.value),
    );

    if (
      selectedActivityKey.value
      && !activityOptions.value.some((item) => item.value === selectedActivityKey.value)
    ) {
      selectedActivityKey.value = "";
    }
    pickDefaultActivity();
  }

  function pickDefaultActivity() {
    if (!attachCurrentStatus.value || selectedActivityKey.value || !activityOptions.value.length) {
      return;
    }

    const preferred = activityOptions.value.find((item) => item.group === "active") ?? activityOptions.value[0];
    selectedActivityKey.value = preferred?.value ?? "";
  }

  async function ensureBatteryPercentLoaded() {
    if (
      !attachCurrentStatus.value
      || !mobileRuntime.value
      || !attachStatusIncludeDeviceInfo.value
      || statusBatteryPercent.value !== null
    ) {
      return;
    }

    try {
      const battery = await readBatterySnapshot();
      statusBatteryPercent.value = battery.levelPercent;
    } catch {
      // Ignore battery read failure and fallback to device name only.
    }
  }

  function applyDraftStateFromStore() {
    statusSnapshotInput.value = options.draftStore.statusSnapshotInput;
    statusSnapshotDeviceName.value = options.draftStore.statusSnapshotDeviceName;
    selectedActivityKey.value = options.draftStore.selectedActivityKey;
    attachCurrentStatus.value = options.draftStore.attachCurrentStatus;
    attachStatusIncludeDeviceInfo.value = options.draftStore.attachStatusIncludeDeviceInfo;
    statusBatteryPercent.value = null;
  }

  onMounted(() => {
    if (options.config.baseUrl.trim() && !mobileRuntime.value) {
      void loadActivityOptions();
    }
  });

  watch(statusSnapshotInput, (value) => {
    options.draftStore.patchDraft({ statusSnapshotInput: value });
  });

  watch(statusSnapshotDeviceName, (value) => {
    options.draftStore.patchDraft({ statusSnapshotDeviceName: value });
  });

  watch(selectedActivityKey, (value) => {
    options.draftStore.patchDraft({ selectedActivityKey: value });
  });

  watch(attachCurrentStatus, (value) => {
    options.draftStore.patchDraft({ attachCurrentStatus: value });
    if (value) {
      pickDefaultActivity();
    }
  });

  watch(attachStatusIncludeDeviceInfo, (value) => {
    options.draftStore.patchDraft({ attachStatusIncludeDeviceInfo: value });
    if (!mobileRuntime.value && options.config.baseUrl.trim()) {
      void loadActivityOptions();
    }
  });

  watch(
    () => options.config.baseUrl.trim(),
    (nextBaseUrl, previousBaseUrl) => {
      if (!nextBaseUrl) {
        activityOptions.value = [];
        activityLoadError.value = "";
        return;
      }

      if (!mobileRuntime.value && nextBaseUrl !== previousBaseUrl) {
        void loadActivityOptions();
      }
    },
  );

  return {
    activityLoadError,
    activityLoading,
    activityOptions,
    applyDraftStateFromStore,
    attachCurrentStatus,
    attachStatusIncludeDeviceInfo,
    buildManualSnapshot,
    ensureBatteryPercentLoaded,
    loadActivityOptions,
    mobileRuntime,
    selectedActivityKey,
    selectedActivityOption,
    selectedSnapshotPreview,
    statusSnapshotDeviceName,
    statusSnapshotInput,
  };
}

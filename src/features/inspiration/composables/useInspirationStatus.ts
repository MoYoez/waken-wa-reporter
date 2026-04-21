import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import { readBatterySnapshot } from "@/lib/deviceInfo";
import type { ClientCapabilities, ClientConfig } from "@/types";
import { useInspirationActivityOptions } from "@/features/inspiration/composables/useInspirationActivityOptions";
import { useInspirationStatusSnapshot } from "@/features/inspiration/composables/inspirationStatusSnapshot";

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

  const mobileRuntime = computed(() => !options.capabilities.realtimeReporter);
  const { buildManualSnapshot, buildSnapshotText } = useInspirationStatusSnapshot({
    config: options.config,
    statusBatteryPercent,
    statusSnapshotDeviceName,
    t,
  });
  const {
    activityLoadError,
    activityLoading,
    activityOptions,
    loadActivityOptions,
    pickDefaultActivity,
    selectedActivityOption,
  } = useInspirationActivityOptions({
    apiErrorDetail: options.apiErrorDetail,
    attachCurrentStatus,
    attachStatusIncludeDeviceInfo,
    buildSnapshotText,
    config: options.config,
    mobileRuntime,
    selectedActivityKey,
    t,
  });
  const selectedSnapshotPreview = computed(() => {
    if (!attachCurrentStatus.value) {
      return "";
    }
    if (mobileRuntime.value) {
      return buildManualSnapshot(statusSnapshotInput.value, attachStatusIncludeDeviceInfo.value);
    }
    return selectedActivityOption.value?.snapshot ?? "";
  });

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

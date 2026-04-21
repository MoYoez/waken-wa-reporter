import { computed, ref, type ComputedRef, type Ref } from "vue";

import { getPublicActivityFeed } from "@/lib/api";
import type { ActivityFeedItem, ClientConfig } from "@/types";
import type { ActivitySelectOption } from "@/features/inspiration/types";

type TranslateFn = (key: string, params?: Record<string, unknown>) => string;

interface UseInspirationActivityOptionsOptions {
  apiErrorDetail: (
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) => string;
  attachCurrentStatus: Ref<boolean>;
  attachStatusIncludeDeviceInfo: Ref<boolean>;
  buildSnapshotText: (item: ActivityFeedItem, includeDeviceInfo: boolean) => string;
  config: ClientConfig;
  mobileRuntime: ComputedRef<boolean>;
  selectedActivityKey: Ref<string>;
  t: TranslateFn;
}

export function useInspirationActivityOptions(options: UseInspirationActivityOptionsOptions) {
  const activityOptions = ref<ActivitySelectOption[]>([]);
  const activityLoading = ref(false);
  const activityLoadError = ref("");

  const selectedActivityOption = computed(() =>
    activityOptions.value.find((item) => item.value === options.selectedActivityKey.value) ?? null,
  );

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
        options.t("inspiration.notify.activityLoadFailed"),
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
      ...toActivityOptions(
        activeStatuses as ActivityFeedItem[],
        "active",
        options.attachStatusIncludeDeviceInfo.value,
        options.buildSnapshotText,
        options.t,
      ),
      ...toActivityOptions(
        (recentActivities as ActivityFeedItem[]).slice(0, 20),
        "recent",
        options.attachStatusIncludeDeviceInfo.value,
        options.buildSnapshotText,
        options.t,
      ),
    ];

    activityOptions.value = optionsList.filter(
      (item, index, all) => index === all.findIndex((candidate) => candidate.value === item.value),
    );

    if (
      options.selectedActivityKey.value
      && !activityOptions.value.some((item) => item.value === options.selectedActivityKey.value)
    ) {
      options.selectedActivityKey.value = "";
    }

    pickDefaultActivity();
  }

  function pickDefaultActivity() {
    if (
      !options.attachCurrentStatus.value
      || options.selectedActivityKey.value
      || !activityOptions.value.length
    ) {
      return;
    }

    const preferred =
      activityOptions.value.find((item) => item.group === "active")
      ?? activityOptions.value[0];
    options.selectedActivityKey.value = preferred?.value ?? "";
  }

  return {
    activityLoadError,
    activityLoading,
    activityOptions,
    loadActivityOptions,
    pickDefaultActivity,
    selectedActivityOption,
  };
}

function toActivityOptions(
  items: ActivityFeedItem[],
  group: "active" | "recent",
  includeDeviceInfo: boolean,
  buildSnapshotText: UseInspirationActivityOptionsOptions["buildSnapshotText"],
  t: TranslateFn,
) {
  return items
    .map((item, index) => {
      const idPart = String(item.id ?? `${item.processName ?? "item"}-${index}`);
      const snapshot = buildSnapshotText(item, includeDeviceInfo);
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

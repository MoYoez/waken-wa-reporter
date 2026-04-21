import { computed, ref } from "vue";

import {
  requestAccessibilityPermission,
  runPlatformSelfTest,
} from "@/lib/api";
import type { NotifyPayload } from "@/lib/notify";
import type { PlatformSelfTestResult } from "@/types";
import {
  buildSelfTestCardViews,
  resolveSelfTestPlatformHintKey,
} from "@/features/settings/composables/settingsWorkspaceProbeText";

interface UseSettingsWorkspaceSelfTestOptions {
  apiErrorDetail: (
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) => string;
  notify: (payload: NotifyPayload) => void;
  t: (key: string, params?: Record<string, unknown>) => string;
}

export function useSettingsWorkspaceSelfTest(options: UseSettingsWorkspaceSelfTestOptions) {
  const selfTestLoading = ref(false);
  const accessibilityPermissionLoading = ref(false);
  const selfTestResult = ref<PlatformSelfTestResult | null>(null);

  const selfTestCards = computed(() =>
    buildSelfTestCardViews(selfTestResult.value, options.t),
  );
  const selfTestPlatformHintKey = computed(() =>
    resolveSelfTestPlatformHintKey(selfTestResult.value),
  );

  async function handleSelfTest() {
    selfTestLoading.value = true;
    const result = await runPlatformSelfTest();
    selfTestLoading.value = false;

    if (!result.success || !result.data) {
      options.notify({
        severity: "error",
        summary: options.t("settings.notify.selfTestFailed"),
        detail: options.apiErrorDetail(result.error, options.t("settings.notify.selfTestFailedDetail")),
        life: 4000,
      });
      return;
    }

    selfTestResult.value = result.data;
    options.notify({
      severity: result.data.foreground.success && result.data.media.success ? "success" : "warn",
      summary: options.t("settings.notify.selfTestDone"),
      detail: options.t("settings.selfTest.platformDetail", { platform: result.data.platform }),
      life: 3000,
    });
  }

  async function handleRequestAccessibilityPermission() {
    accessibilityPermissionLoading.value = true;
    const result = await requestAccessibilityPermission();
    accessibilityPermissionLoading.value = false;

    if (!result.success) {
      options.notify({
        severity: "error",
        summary: options.t("settings.notify.permissionFailed"),
        detail: options.apiErrorDetail(result.error, options.t("settings.notify.permissionFailedDetail")),
        life: 4000,
      });
      return;
    }

    options.notify({
      severity: result.data ? "success" : "info",
      summary: result.data
        ? options.t("settings.notify.permissionGranted")
        : options.t("settings.notify.permissionRequested"),
      detail: result.data
        ? options.t("settings.notify.permissionGrantedDetail")
        : options.t("settings.notify.permissionRequestedDetail"),
      life: 5000,
    });

    await handleSelfTest();
  }

  return {
    accessibilityPermissionLoading,
    handleRequestAccessibilityPermission,
    handleSelfTest,
    selfTestCards,
    selfTestLoading,
    selfTestPlatformHintKey,
    selfTestResult,
  };
}

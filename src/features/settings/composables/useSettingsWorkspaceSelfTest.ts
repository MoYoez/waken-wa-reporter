import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

import {
  getAndroidPermissionStatus,
  openAndroidReporterNotificationSettings,
  requestAccessibilityPermission,
  requestAndroidNotificationAccess,
  requestAndroidUsageAccess,
  runPlatformSelfTest,
} from "@/lib/api";
import type { NotifyPayload } from "@/lib/notify";
import type { AndroidPermissionStatus, PlatformSelfTestResult } from "@/types";
import {
  buildSelfTestCardViews,
  resolveSelfTestPlatformHintKey,
  type SelfTestPermissionAction,
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
  const androidNotificationPermissionLoading = ref(false);
  const androidPermissionStatus = ref<AndroidPermissionStatus | null>(null);
  const androidReporterNotificationPermissionGranted = ref(false);
  const selfTestResult = ref<PlatformSelfTestResult | null>(null);

  const selfTestCards = computed(() =>
    buildSelfTestCardViews(selfTestResult.value, options.t, androidPermissionStatus.value),
  );
  const selfTestPlatformHintKey = computed(() =>
    resolveSelfTestPlatformHintKey(selfTestResult.value),
  );

  async function refreshAndroidPermissionStatus(refreshOptions?: { silent?: boolean }) {
    if (!isAndroidSelfTest()) {
      androidPermissionStatus.value = null;
      return;
    }

    try {
      const result = await getAndroidPermissionStatus();
      if (result.success && result.data) {
        androidPermissionStatus.value = result.data;
      } else if (!refreshOptions?.silent) {
        optionsNotifyPermissionRefreshFailed(result.error);
      }
    } catch (error) {
      if (!refreshOptions?.silent) {
        options.notify({
          severity: "error",
          summary: options.t("settings.notify.permissionFailed"),
          detail: error instanceof Error ? error.message : options.t("settings.notify.permissionFailedDetail"),
          life: 4000,
        });
      }
    }
  }

  async function refreshAndroidReporterNotificationPermission(refreshOptions?: { silent?: boolean }) {
    if (!isAndroidRuntime()) {
      androidReporterNotificationPermissionGranted.value = false;
      return;
    }

    try {
      androidReporterNotificationPermissionGranted.value = await readAndroidReporterNotificationPermission();
    } catch (error) {
      androidReporterNotificationPermissionGranted.value = false;
      if (!refreshOptions?.silent) {
        options.notify({
          severity: "error",
          summary: options.t("settings.notify.permissionFailed"),
          detail: error instanceof Error ? error.message : options.t("settings.notify.permissionFailedDetail"),
          life: 4000,
        });
      }
    }
  }

  async function handleRequestAndroidReporterNotificationPermission() {
    if (!isAndroidRuntime()) {
      return;
    }

    androidNotificationPermissionLoading.value = true;
    try {
      const alreadyGranted = await readAndroidReporterNotificationPermission();
      if (!alreadyGranted) {
        const result = await openAndroidReporterNotificationSettings();
        if (!result.success) {
          options.notify({
            severity: "error",
            summary: options.t("settings.notify.permissionFailed"),
            detail: options.apiErrorDetail(result.error, options.t("settings.notify.permissionFailedDetail")),
            life: 4000,
          });
          return;
        }
      }

      const granted = alreadyGranted || await readAndroidReporterNotificationPermission();
      androidReporterNotificationPermissionGranted.value = granted;
      options.notify({
        severity: granted ? "success" : "info",
        summary: granted
          ? options.t("settings.notify.permissionGranted")
          : options.t("settings.notify.permissionRequested"),
        detail: granted
          ? options.t("settings.notify.androidReporterNotificationPermissionGrantedDetail")
          : options.t("settings.notify.androidReporterNotificationPermissionRequestedDetail"),
        life: 5000,
      });

      window.setTimeout(() => {
        void refreshAndroidReporterNotificationPermission({ silent: true });
      }, 1200);
    } catch (error) {
      options.notify({
        severity: "error",
        summary: options.t("settings.notify.permissionFailed"),
        detail: error instanceof Error ? error.message : options.t("settings.notify.permissionFailedDetail"),
        life: 4000,
      });
    } finally {
      androidNotificationPermissionLoading.value = false;
    }
  }

  async function handleSelfTest(runOptions?: { silent?: boolean }) {
    selfTestLoading.value = true;
    try {
      const result = await runPlatformSelfTest();

      if (!result.success || !result.data) {
        if (!runOptions?.silent) {
          options.notify({
            severity: "error",
            summary: options.t("settings.notify.selfTestFailed"),
            detail: options.apiErrorDetail(result.error, options.t("settings.notify.selfTestFailedDetail")),
            life: 4000,
          });
        }
        return;
      }

      selfTestResult.value = result.data;
      await refreshAndroidPermissionStatus({ silent: true });
      await refreshAndroidReporterNotificationPermission({ silent: true });
      if (!runOptions?.silent) {
        options.notify({
          severity: result.data.foreground.success && result.data.media.success ? "success" : "warn",
          summary: options.t("settings.notify.selfTestDone"),
          detail: options.t("settings.selfTest.platformDetail", { platform: result.data.platform }),
          life: 3000,
        });
      }
    } catch (error) {
      if (!runOptions?.silent) {
        options.notify({
          severity: "error",
          summary: options.t("settings.notify.selfTestFailed"),
          detail: error instanceof Error ? error.message : options.t("settings.notify.selfTestFailedDetail"),
          life: 4000,
        });
      }
    } finally {
      selfTestLoading.value = false;
    }
  }

  async function handleRequestPermission(action: SelfTestPermissionAction) {
    accessibilityPermissionLoading.value = true;
    try {
      const result = await requestPermissionAction(action);

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

      await schedulePermissionRefresh(action);
    } catch (error) {
      options.notify({
        severity: "error",
        summary: options.t("settings.notify.permissionFailed"),
        detail: error instanceof Error ? error.message : options.t("settings.notify.permissionFailedDetail"),
        life: 4000,
      });
    } finally {
      accessibilityPermissionLoading.value = false;
    }
  }

  async function schedulePermissionRefresh(action: SelfTestPermissionAction) {
    if (action === "accessibility" && selfTestResult.value?.platform !== "android") {
      window.setTimeout(() => {
        void handleSelfTest({ silent: true });
      }, 900);
      return;
    }

    await refreshAndroidPermissionStatus({ silent: true });
    window.setTimeout(() => {
      void refreshAndroidPermissionStatus({ silent: true });
      void handleSelfTest({ silent: true });
    }, 900);
  }

  function requestPermissionAction(action: SelfTestPermissionAction) {
    if (action === "usage") {
      return requestAndroidUsageAccess();
    }
    if (action === "notification") {
      return requestAndroidNotificationAccess();
    }
    return requestAccessibilityPermission();
  }

  function isAndroidSelfTest() {
    return selfTestResult.value?.platform === "android";
  }

  async function readAndroidReporterNotificationPermission() {
    try {
      const granted = await invoke<boolean | null>("plugin:notification|is_permission_granted");
      return granted === true;
    } catch {
      return false;
    }
  }

  function optionsNotifyPermissionRefreshFailed(
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
  ) {
    options.notify({
      severity: "error",
      summary: options.t("settings.notify.permissionFailed"),
      detail: options.apiErrorDetail(error, options.t("settings.notify.permissionFailedDetail")),
      life: 4000,
    });
  }

  return {
    accessibilityPermissionLoading,
    androidNotificationPermissionLoading,
    androidReporterNotificationPermissionGranted,
    handleRequestPermission,
    handleRequestAndroidReporterNotificationPermission,
    handleSelfTest,
    refreshAndroidReporterNotificationPermission,
    selfTestCards,
    selfTestLoading,
    selfTestPlatformHintKey,
    selfTestResult,
    refreshAndroidPermissionStatus,
  };
}

function isAndroidRuntime() {
  return typeof navigator !== "undefined" && /Android/i.test(navigator.userAgent);
}

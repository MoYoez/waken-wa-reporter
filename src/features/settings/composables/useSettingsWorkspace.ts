import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "primevue/usetoast";

import type { SupportedLocale } from "@/i18n";
import {
  requestAccessibilityPermission,
  runPlatformSelfTest,
  validateDiscordPresenceConfig,
} from "@/lib/api";
import {
  resolveApiErrorMessage,
  resolveLocalizedEntry,
  resolveLocalizedText,
} from "@/lib/localizedText";
import { createNotifier } from "@/lib/notify";
import type {
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  PlatformProbeResult,
  PlatformSelfTestResult,
  RealtimeReporterSnapshot,
} from "@/types";

interface SettingsWorkspaceProps {
  modelValue: ClientConfig;
  locale: SupportedLocale;
  capabilities: ClientCapabilities;
  reporterSnapshot: RealtimeReporterSnapshot;
  discordPresenceSnapshot: DiscordPresenceSnapshot;
}

interface SettingsWorkspaceCallbacks {
  onUpdateModelValue: (value: ClientConfig) => void;
  onRestartApp: () => void;
}

interface SelfTestCardView {
  key: "foreground" | "windowTitle" | "media";
  titleKey: string;
  success: boolean;
  primaryText: string;
  secondaryText: string;
  showAccessibilityAction?: boolean;
}

export function useSettingsWorkspace(
  props: SettingsWorkspaceProps,
  callbacks: SettingsWorkspaceCallbacks,
) {
  const { t, locale } = useI18n();
  const toast = useToast();
  const selfTestLoading = ref(false);
  const accessibilityPermissionLoading = ref(false);
  const selfTestResult = ref<PlatformSelfTestResult | null>(null);

  const configReady = computed(
    () => !!props.modelValue.baseUrl.trim() && !!props.modelValue.apiToken.trim(),
  );
  const reporterSupported = computed(() => props.capabilities.realtimeReporter);
  const discordSupported = computed(() => props.capabilities.discordPresence);
  const selfTestSupported = computed(() => props.capabilities.platformSelfTest);
  const autostartSupported = computed(() => props.capabilities.autostart);
  const isNativeNotice = computed(() => !props.capabilities.realtimeReporter);
  const canRequestAccessibilityPermission = computed(() => {
    if (typeof navigator === "undefined") {
      return false;
    }
    return /mac/i.test(navigator.userAgent);
  });
  const { notify } = createNotifier(toast, () => isNativeNotice.value);
  const discordConfigIssues = computed(() =>
    validateDiscordPresenceConfig(props.modelValue, props.capabilities),
  );
  const discordConfigReady = computed(() => discordConfigIssues.value.length === 0);
  const selfTestCards = computed<SelfTestCardView[]>(() => {
    if (!selfTestResult.value) {
      return [];
    }

    return [
      {
        key: "foreground",
        titleKey: "settings.selfTest.foreground",
        success: selfTestResult.value.foreground.success,
        primaryText: primaryProbeText(selfTestResult.value.foreground),
        secondaryText: secondaryProbeText(selfTestResult.value.foreground),
      },
      {
        key: "windowTitle",
        titleKey: "settings.selfTest.windowTitle",
        success: selfTestResult.value.windowTitle.success,
        primaryText: primaryProbeText(selfTestResult.value.windowTitle),
        secondaryText: secondaryProbeText(selfTestResult.value.windowTitle),
        showAccessibilityAction:
          selfTestResult.value.platform === "macos"
          && !selfTestResult.value.windowTitle.success,
      },
      {
        key: "media",
        titleKey: "settings.selfTest.media",
        success: selfTestResult.value.media.success,
        primaryText: primaryProbeText(selfTestResult.value.media),
        secondaryText: secondaryProbeText(selfTestResult.value.media),
      },
    ];
  });
  const selfTestPlatformHintKey = computed(() => {
    if (selfTestResult.value?.platform === "macos") {
      return "settings.selfTest.macosHint";
    }
    if (selfTestResult.value?.platform === "linux") {
      return "settings.selfTest.linuxHint";
    }
    return "";
  });

  function updateField<K extends keyof ClientConfig>(key: K, value: ClientConfig[K]) {
    callbacks.onUpdateModelValue({
      ...props.modelValue,
      [key]: value,
    });
  }

  function formatTime(value?: string | null) {
    if (!value) {
      return t("settings.notify.none");
    }

    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
      return value;
    }

    return date.toLocaleString(locale.value);
  }

  function translateText(key: string, params?: Record<string, unknown>) {
    return params ? t(key, params) : t(key);
  }

  function apiErrorDetail(
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) {
    return resolveApiErrorMessage(error, translateText, fallback);
  }

  function firstGuidance(probe?: PlatformProbeResult | null) {
    const localized = probe?.guidanceEntries
      ?.map((entry) => resolveLocalizedEntry(entry, translateText))
      .find((item) => item.trim());
    if (localized) {
      return localized;
    }

    return probe?.guidance?.find((item) => item.trim()) ?? "";
  }

  function probeSummary(probe: PlatformProbeResult) {
    return resolveLocalizedText(
      translateText,
      probe.summaryKey,
      probe.summaryParams,
      probe.summary,
    );
  }

  function probeDetail(probe: PlatformProbeResult) {
    return resolveLocalizedText(
      translateText,
      probe.detailKey,
      probe.detailParams,
      probe.detail,
    );
  }

  function compactDetail(value?: string | null) {
    const text = (value ?? "").trim();
    if (!text) {
      return t("settings.notify.noneResult");
    }

    const normalized = text.replace(/\s+/g, " ");
    if (normalized.length <= 88) {
      return normalized;
    }

    const firstChunk = normalized.split(/[；;]/)[0]?.trim() || normalized;
    if (firstChunk.length <= 88) {
      return firstChunk;
    }

    return `${firstChunk.slice(0, 84).trimEnd()}...`;
  }

  function primaryProbeText(probe: PlatformProbeResult) {
    return probe.success ? compactDetail(probeDetail(probe)) : probeSummary(probe);
  }

  function secondaryProbeText(probe: PlatformProbeResult) {
    if (probe.success) {
      return "";
    }

    return firstGuidance(probe) || probeDetail(probe);
  }

  async function handleSelfTest() {
    selfTestLoading.value = true;
    const result = await runPlatformSelfTest();
    selfTestLoading.value = false;

    if (!result.success || !result.data) {
      notify({
        severity: "error",
        summary: t("settings.notify.selfTestFailed"),
        detail: apiErrorDetail(result.error, t("settings.notify.selfTestFailedDetail")),
        life: 4000,
      });
      return;
    }

    selfTestResult.value = result.data;
    notify({
      severity: result.data.foreground.success && result.data.media.success ? "success" : "warn",
      summary: t("settings.notify.selfTestDone"),
      detail: t("settings.selfTest.platformDetail", { platform: result.data.platform }),
      life: 3000,
    });
  }

  async function handleRequestAccessibilityPermission() {
    accessibilityPermissionLoading.value = true;
    const result = await requestAccessibilityPermission();
    accessibilityPermissionLoading.value = false;

    if (!result.success) {
      notify({
        severity: "error",
        summary: t("settings.notify.permissionFailed"),
        detail: apiErrorDetail(result.error, t("settings.notify.permissionFailedDetail")),
        life: 4000,
      });
      return;
    }

    notify({
      severity: result.data ? "success" : "info",
      summary: result.data
        ? t("settings.notify.permissionGranted")
        : t("settings.notify.permissionRequested"),
      detail: result.data
        ? t("settings.notify.permissionGrantedDetail")
        : t("settings.notify.permissionRequestedDetail"),
      life: 5000,
    });

    await handleSelfTest();
  }

  function handleRestartApp() {
    callbacks.onRestartApp();
  }

  return {
    accessibilityPermissionLoading,
    autostartSupported,
    canRequestAccessibilityPermission,
    configReady,
    discordConfigIssues,
    discordConfigReady,
    discordSupported,
    formatTime,
    handleRequestAccessibilityPermission,
    handleRestartApp,
    handleSelfTest,
    reporterSupported,
    selfTestCards,
    selfTestLoading,
    selfTestPlatformHintKey,
    selfTestResult,
    selfTestSupported,
    updateField,
  };
}

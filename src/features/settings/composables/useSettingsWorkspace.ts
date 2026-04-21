import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "primevue/usetoast";

import type { SupportedLocale } from "@/i18n";
import {
  validateDiscordPresenceConfig,
} from "@/lib/api";
import { resolveApiErrorMessage } from "@/lib/localizedText";
import { createNotifier } from "@/lib/notify";
import type {
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  RealtimeReporterSnapshot,
} from "@/types";
import { useSettingsWorkspaceSelfTest } from "@/features/settings/composables/useSettingsWorkspaceSelfTest";

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

export function useSettingsWorkspace(
  props: SettingsWorkspaceProps,
  callbacks: SettingsWorkspaceCallbacks,
) {
  const { t, locale } = useI18n();
  const toast = useToast();

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
  const {
    accessibilityPermissionLoading,
    handleRequestAccessibilityPermission,
    handleSelfTest,
    selfTestCards,
    selfTestLoading,
    selfTestPlatformHintKey,
    selfTestResult,
  } = useSettingsWorkspaceSelfTest({
    apiErrorDetail,
    notify,
    t: translateText,
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

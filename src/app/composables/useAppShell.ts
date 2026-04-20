import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "primevue/usetoast";

import { useAppShellBootstrap } from "@/app/composables/useAppShellBootstrap";
import { useAppShellLifecycle } from "@/app/composables/useAppShellLifecycle";
import { useAppShellNavigation } from "@/app/composables/useAppShellNavigation";
import { useAppShellPersistence } from "@/app/composables/useAppShellPersistence";
import { useAppShellRuntime } from "@/app/composables/useAppShellRuntime";
import { useAppShellViewport } from "@/app/composables/useAppShellViewport";
import { useAppShellWatchers } from "@/app/composables/useAppShellWatchers";
import { normalizeLocale, type SupportedLocale } from "@/i18n";
import { resolveApiErrorMessage } from "@/lib/localizedText";
import { createNotifier } from "@/lib/notify";
import { defaultClientConfig } from "@/lib/persistence";
import type {
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  ExistingReporterConfig,
  MobileConnectivityState,
  RealtimeReporterSnapshot,
  RecentPreset,
} from "@/types";

const defaultCapabilities: ClientCapabilities = {
  realtimeReporter: true,
  tray: true,
  platformSelfTest: true,
  discordPresence: true,
  autostart: true,
};

export function useAppShell() {
  const { t, locale } = useI18n();
  const toast = useToast();

  const capabilities = ref<ClientCapabilities>(defaultCapabilities);
  const config = ref<ClientConfig>(defaultClientConfig());
  const persistedConfig = ref<ClientConfig>(defaultClientConfig());
  const onboardingDraftConfig = ref<ClientConfig>(defaultClientConfig());
  const recentPresets = ref<RecentPreset[]>([]);
  const currentLocale = ref<SupportedLocale>(normalizeLocale(locale.value));
  const startupLocale = ref<SupportedLocale>(normalizeLocale(locale.value));
  const hydrated = ref(false);
  const onboardingDismissed = ref(false);
  const reporterConfigPromptHandled = ref(false);
  const reporterBusy = ref(false);
  const discordBusy = ref(false);
  const importingReporterConfig = ref(false);
  const existingReporterConfig = ref<ExistingReporterConfig | null>(null);
  const verifiedGeneratedHashKey = ref("");
  const mobileConnectivity = ref<MobileConnectivityState>({
    checking: false,
    checked: false,
    ok: null,
    summary: t("app.mobileConnectivity.pending"),
    detail: "",
    checkedAt: null,
  });
  const reporterSnapshot = ref<RealtimeReporterSnapshot>({
    running: false,
    logs: [],
    currentActivity: null,
    lastHeartbeatAt: null,
    lastError: null,
    lastPendingApprovalMessage: null,
    lastPendingApprovalUrl: null,
  });
  const discordPresenceSnapshot = ref<DiscordPresenceSnapshot>({
    running: false,
    connected: false,
    lastSyncAt: null,
    lastError: null,
    currentSummary: null,
  });
  const pendingApprovalDialogVisible = ref(false);
  const lastPendingApprovalSeen = ref("");
  const lastMobileConnectivitySignature = ref("");
  const onboardingSetupMode = ref(false);
  const localeSaving = ref(false);
  const restartingApp = ref(false);

  const reporterSupported = computed(() => capabilities.value.realtimeReporter);
  const discordSupported = computed(() => capabilities.value.discordPresence);
  const traySupported = computed(() => capabilities.value.tray);
  const autostartSupported = computed(() => capabilities.value.autostart);
  const isNativeNotice = computed(() => !reporterSupported.value);
  const { notify } = createNotifier(toast, () => isNativeNotice.value);

  const { activeSection, visibleSections, ensureVisibleSection, selectSection } = useAppShellNavigation({
    t: translateText,
    reporterSupported,
  });
  const readiness = computed(() => {
    const required = [
      config.value.baseUrl.trim(),
      config.value.apiToken.trim(),
      config.value.generatedHashKey.trim(),
    ];
    return required.every(Boolean);
  });
  const discordReadiness = computed(
    () => !!config.value.baseUrl.trim() && !!config.value.discordApplicationId.trim(),
  );
  const shouldShowOnboarding = computed(
    () => hydrated.value && !onboardingDismissed.value && !readiness.value,
  );
  const localeRestartRequired = computed(() => currentLocale.value !== startupLocale.value);
  const hasPendingSettingsChanges = computed(() => {
    const current = JSON.stringify(normalizeConfigByCapabilities(config.value));
    const persisted = JSON.stringify(normalizeConfigByCapabilities(persistedConfig.value));
    return current !== persisted;
  });
  const settingsRestarting = computed(() => restartingApp.value || localeSaving.value);

  function translateText(key: string, params?: Record<string, unknown>) {
    return params ? t(key, params) : t(key);
  }

  function apiErrorDetail(
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) {
    return resolveApiErrorMessage(error, translateText, fallback);
  }

  const { inferMobileDeviceType, isPhone, syncDeviceTypeByViewport, syncViewportWidth } = useAppShellViewport({
    reporterSupported,
    config,
    onboardingDraftConfig,
  });

  const {
    applyLocale,
    applySettingsChanges,
    closeOnboarding,
    completeOnboardingSetup,
    handleRestartApp,
    hydrateDeviceNameFromSystem,
    normalizeConfigByCapabilities,
    notifyImport,
    persistAppState,
    rememberVerifiedGeneratedHashKey,
    resolveAutostartConfig,
    revertPendingSettings,
    skipExistingReporterConfig,
    startSetup,
    updateConfig,
    updateOnboardingDraftConfig,
    useExistingReporterConfig,
  } = useAppShellPersistence({
    t: translateText,
    notify,
    config,
    persistedConfig,
    onboardingDraftConfig,
    recentPresets,
    currentLocale,
    hydrated,
    onboardingDismissed,
    onboardingSetupMode,
    reporterConfigPromptHandled,
    importingReporterConfig,
    existingReporterConfig,
    verifiedGeneratedHashKey,
    localeSaving,
    restartingApp,
    reporterSupported,
    autostartSupported,
    localeRestartRequired,
    inferMobileDeviceType,
  });

  const {
    closePendingApprovalDialog,
    handlePendingApproval,
    handleStartDiscordPresence,
    handleStartReporter,
    handleStopDiscordPresence,
    handleStopReporter,
    refreshDiscordPresenceSnapshot,
    refreshReporterSnapshot,
    runMobileConnectivityProbe,
    shouldPollDiscordPresenceSnapshot,
    shouldPollReporterSnapshot,
    stopAllPolling,
    syncDiscordPresencePolling,
    syncReporterPolling,
  } = useAppShellRuntime({
    t: translateText,
    notify,
    apiErrorDetail,
    config,
    activeSection,
    reporterBusy,
    discordBusy,
    mobileConnectivity,
    reporterSnapshot,
    discordPresenceSnapshot,
    pendingApprovalDialogVisible,
    lastMobileConnectivitySignature,
    reporterSupported,
    discordSupported,
    readiness,
    rememberVerifiedGeneratedHashKey,
  });

  const { bootstrapAppShell } = useAppShellBootstrap({
    t: translateText,
    locale,
    notify,
    capabilities,
    config,
    persistedConfig,
    onboardingDraftConfig,
    recentPresets,
    currentLocale,
    startupLocale,
    hydrated,
    onboardingDismissed,
    reporterConfigPromptHandled,
    existingReporterConfig,
    verifiedGeneratedHashKey,
    reporterSnapshot,
    discordPresenceSnapshot,
    reporterSupported,
    readiness,
    discordReadiness,
    applyLocale,
    resolveAutostartConfig,
    normalizeConfigByCapabilities,
    persistAppState,
    hydrateDeviceNameFromSystem,
    ensureVisibleSection,
    syncDeviceTypeByViewport,
    refreshReporterSnapshot,
    syncReporterPolling,
    refreshDiscordPresenceSnapshot,
    syncDiscordPresencePolling,
    handleStartReporter,
    handleStartDiscordPresence,
  });

  function handlePresetSaved(preset: RecentPreset) {
    const deduped = recentPresets.value.filter(
      (item) =>
        item.process_name !== preset.process_name
        || item.process_title !== preset.process_title,
    );
    recentPresets.value = [preset, ...deduped].slice(0, 6);
    void persistAppState();
  }

  useAppShellWatchers({
    locale,
    currentLocale,
    hydrated,
    capabilities,
    config,
    activeSection,
    visibleSections,
    reporterSupported,
    discordSupported,
    readiness,
    reporterSnapshot,
    discordPresenceSnapshot,
    pendingApprovalDialogVisible,
    lastPendingApprovalSeen,
    ensureVisibleSection,
    syncDeviceTypeByViewport,
    syncReporterPolling,
    syncDiscordPresencePolling,
    refreshReporterSnapshot,
    refreshDiscordPresenceSnapshot,
    runMobileConnectivityProbe,
  });

  useAppShellLifecycle({
    syncViewportWidth,
    syncReporterPolling,
    syncDiscordPresencePolling,
    shouldPollReporterSnapshot,
    shouldPollDiscordPresenceSnapshot,
    refreshReporterSnapshot,
    refreshDiscordPresenceSnapshot,
    bootstrapAppShell,
    stopAllPolling,
  });

  return {
    activeSection,
    applyLocale,
    applySettingsChanges,
    capabilities,
    closeOnboarding,
    closePendingApprovalDialog,
    completeOnboardingSetup,
    config,
    currentLocale,
    discordBusy,
    discordPresenceSnapshot,
    discordSupported,
    existingReporterConfig,
    handlePendingApproval,
    handlePresetSaved,
    handleRestartApp,
    handleStartDiscordPresence,
    handleStartReporter,
    handleStopDiscordPresence,
    handleStopReporter,
    hasPendingSettingsChanges,
    importingReporterConfig,
    isNativeNotice,
    isPhone,
    localeRestartRequired,
    mobileConnectivity,
    notifyImport,
    onboardingDraftConfig,
    onboardingSetupMode,
    pendingApprovalDialogVisible,
    readiness,
    recentPresets,
    rememberVerifiedGeneratedHashKey,
    reporterBusy,
    reporterConfigPromptHandled,
    reporterSnapshot,
    reporterSupported,
    revertPendingSettings,
    runMobileConnectivityProbe,
    selectSection,
    settingsRestarting,
    shouldShowOnboarding,
    skipExistingReporterConfig,
    startSetup,
    traySupported,
    updateConfig,
    updateOnboardingDraftConfig,
    useExistingReporterConfig,
    verifiedGeneratedHashKey,
    visibleSections,
  };
}

import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "primevue/usetoast";

import { useAppShellBootstrap } from "@/app/composables/useAppShellBootstrap";
import { useAppShellDerivedState } from "@/app/composables/useAppShellDerivedState";
import { useAppShellLifecycle } from "@/app/composables/useAppShellLifecycle";
import { useAppShellNavigation } from "@/app/composables/useAppShellNavigation";
import { useAppShellPersistence } from "@/app/composables/useAppShellPersistence";
import { useAppShellRecentPresets } from "@/app/composables/useAppShellRecentPresets";
import { useAppShellState } from "@/app/composables/useAppShellState";
import { useAppShellRuntime } from "@/app/composables/useAppShellRuntime";
import { useAppShellViewport } from "@/app/composables/useAppShellViewport";
import { useAppShellWatchers } from "@/app/composables/useAppShellWatchers";
import { resolveApiErrorMessage } from "@/lib/localizedText";
import { createNotifier } from "@/lib/notify";

export function useAppShell() {
  const { t, locale } = useI18n();
  const toast = useToast();

  const {
    capabilities,
    config,
    currentLocale,
    discordBusy,
    discordPresenceSnapshot,
    existingReporterConfig,
    hydrated,
    importingReporterConfig,
    lastMobileConnectivitySignature,
    lastPendingApprovalSeen,
    localeSaving,
    mobileConnectivity,
    onboardingDismissed,
    onboardingDraftConfig,
    onboardingSetupMode,
    pendingApprovalDialogVisible,
    persistedConfig,
    recentPresets,
    reporterBusy,
    reporterConfigPromptHandled,
    reporterSnapshot,
    restartingApp,
    startupLocale,
    verifiedGeneratedHashKey,
  } = useAppShellState(locale.value, t("app.mobileConnectivity.pending"));

  const {
    autostartSupported,
    discordReadiness,
    discordSupported,
    isNativeNotice,
    localeRestartRequired,
    readiness,
    reporterSupported,
    settingsRestarting,
    shouldShowOnboarding,
    traySupported,
  } = useAppShellDerivedState({
    capabilities,
    config,
    currentLocale,
    hydrated,
    localeSaving,
    onboardingDismissed,
    restartingApp,
    startupLocale,
  });
  const { notify } = createNotifier(toast, () => isNativeNotice.value);

  const { activeSection, visibleSections, ensureVisibleSection, selectSection } = useAppShellNavigation({
    t: translateText,
    reporterSupported,
  });
  const hasPendingSettingsChanges = computed(() => {
    const current = JSON.stringify(normalizeConfigByCapabilities(config.value));
    const persisted = JSON.stringify(normalizeConfigByCapabilities(persistedConfig.value));
    return current !== persisted;
  });

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

  const { handlePresetSaved } = useAppShellRecentPresets({
    persistAppState,
    recentPresets,
  });

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

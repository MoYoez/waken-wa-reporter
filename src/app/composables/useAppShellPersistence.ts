import type { ComputedRef, Ref } from "vue";

import {
  isAutostartEnabled,
  restartApp as requestAppRestart,
  setAutostartEnabled,
} from "@/lib/api";
import { readDeviceName } from "@/lib/deviceInfo";
import type { NotifyPayload } from "@/lib/notify";
import { saveAppState } from "@/lib/persistence";
import {
  setI18nLocale,
  type SupportedLocale,
} from "@/i18n";
import type {
  ClientConfig,
  DeviceType,
  ExistingReporterConfig,
  RecentPreset,
} from "@/types";

interface UseAppShellPersistenceOptions {
  t: (key: string, params?: Record<string, unknown>) => string;
  notify: (payload: NotifyPayload) => void;
  config: Ref<ClientConfig>;
  persistedConfig: Ref<ClientConfig>;
  onboardingDraftConfig: Ref<ClientConfig>;
  recentPresets: Ref<RecentPreset[]>;
  currentLocale: Ref<SupportedLocale>;
  hydrated: Ref<boolean>;
  onboardingDismissed: Ref<boolean>;
  onboardingSetupMode: Ref<boolean>;
  reporterConfigPromptHandled: Ref<boolean>;
  importingReporterConfig: Ref<boolean>;
  existingReporterConfig: Ref<ExistingReporterConfig | null>;
  verifiedGeneratedHashKey: Ref<string>;
  localeSaving: Ref<boolean>;
  restartingApp: Ref<boolean>;
  reporterSupported: ComputedRef<boolean>;
  autostartSupported: ComputedRef<boolean>;
  localeRestartRequired: ComputedRef<boolean>;
  inferMobileDeviceType: () => DeviceType;
}

export function useAppShellPersistence(options: UseAppShellPersistenceOptions) {
  function normalizeConfigByCapabilities(raw: ClientConfig): ClientConfig {
    const normalizedDevice = raw.device.trim();
    const launchOnStartup = options.autostartSupported.value ? raw.launchOnStartup : false;

    if (options.reporterSupported.value) {
      return {
        ...raw,
        device: normalizedDevice,
        deviceType: "desktop",
        launchOnStartup,
      };
    }

    return {
      ...raw,
      device: normalizedDevice,
      deviceType: options.inferMobileDeviceType(),
      pushMode: "active",
      reporterEnabled: false,
      discordEnabled: false,
      launchOnStartup: false,
    };
  }

  async function resolveAutostartConfig(raw: ClientConfig) {
    if (!options.autostartSupported.value) {
      return { ...raw, launchOnStartup: false };
    }

    try {
      const enabled = await isAutostartEnabled();
      return { ...raw, launchOnStartup: enabled };
    } catch {
      return raw;
    }
  }

  async function persistAppState(configOverride?: ClientConfig) {
    await saveAppState(
      normalizeConfigByCapabilities(configOverride ?? options.persistedConfig.value),
      options.recentPresets.value,
      options.onboardingDismissed.value,
      options.currentLocale.value,
      options.reporterConfigPromptHandled.value,
      options.verifiedGeneratedHashKey.value,
    );
  }

  async function applyLocale(nextLocale: string, persist = false) {
    const normalized = setI18nLocale(nextLocale);
    options.currentLocale.value = normalized;

    if (persist && options.hydrated.value) {
      options.localeSaving.value = true;
      try {
        await persistAppState();
      } catch (error) {
        options.notify({
          severity: "error",
          summary: options.t("app.notify.settingsSaveFailed"),
          detail: error instanceof Error
            ? error.message
            : options.t("app.notify.settingsSaveFailedDetail"),
          life: 4000,
        });
      } finally {
        options.localeSaving.value = false;
      }
    }
  }

  function notifyImport(message: string) {
    options.notify({
      severity: "success",
      summary: options.t("app.notify.importedConfig"),
      detail: message,
      life: 3000,
    });
  }

  function rememberVerifiedGeneratedHashKey(nextKey: string) {
    const normalized = nextKey.trim();
    if (!normalized || options.verifiedGeneratedHashKey.value === normalized) {
      return;
    }

    options.verifiedGeneratedHashKey.value = normalized;
    void persistAppState();
  }

  function closeOnboarding() {
    options.onboardingSetupMode.value = false;
    options.onboardingDismissed.value = true;
    void persistAppState();
  }

  function startSetup() {
    options.reporterConfigPromptHandled.value = true;
    options.onboardingDraftConfig.value = { ...options.config.value };
    options.onboardingSetupMode.value = true;
  }

  function skipExistingReporterConfig() {
    options.reporterConfigPromptHandled.value = true;
    void persistAppState();
  }

  async function useExistingReporterConfig() {
    if (!options.existingReporterConfig.value?.config) {
      return;
    }

    options.importingReporterConfig.value = true;
    options.onboardingDraftConfig.value = normalizeConfigByCapabilities({
      ...options.existingReporterConfig.value.config,
    });
    options.reporterConfigPromptHandled.value = true;
    options.importingReporterConfig.value = false;
    options.notify({
      severity: "success",
      summary: options.t("app.notify.importedExistingConfig"),
      detail: options.t("app.notify.importedExistingConfigDetail"),
      life: 3500,
    });
    options.onboardingSetupMode.value = true;
  }

  async function handleRestartApp() {
    if (options.restartingApp.value) {
      return;
    }

    options.restartingApp.value = true;
    try {
      if (options.localeSaving.value || options.localeRestartRequired.value) {
        await persistAppState();
      }
      await requestAppRestart();
    } catch (error) {
      options.restartingApp.value = false;
      options.notify({
        severity: "error",
        summary: options.t("app.notify.restartFailed"),
        detail: error instanceof Error
          ? error.message
          : options.t("app.notify.restartFailedDetail"),
        life: 4000,
      });
    }
  }

  async function hydrateDeviceNameFromSystem() {
    if (options.config.value.device.trim()) {
      return;
    }

    try {
      const deviceName = (await readDeviceName()).trim();
      if (!deviceName) {
        return;
      }

      const nextConfig = normalizeConfigByCapabilities({
        ...options.config.value,
        device: deviceName,
      });
      options.config.value = nextConfig;
      options.persistedConfig.value = { ...nextConfig };
      options.onboardingDraftConfig.value = { ...nextConfig };
      await persistAppState(nextConfig);
    } catch {
      // Ignore plugin read failures and keep backend fallback behavior.
    }
  }

  async function applySettingsChanges() {
    try {
      const nextConfig = normalizeConfigByCapabilities({ ...options.config.value });

      if (options.autostartSupported.value) {
        await setAutostartEnabled(nextConfig.launchOnStartup);
      }

      options.config.value = nextConfig;
      await persistAppState(nextConfig);
      options.persistedConfig.value = { ...nextConfig };
      options.notify({
        severity: "success",
        summary: options.t("app.notify.settingsSaved"),
        detail: options.t("app.notify.settingsSavedDetail"),
        life: 2500,
      });
    } catch (error) {
      options.notify({
        severity: "error",
        summary: options.t("app.notify.settingsSaveFailed"),
        detail: error instanceof Error
          ? error.message
          : options.t("app.notify.settingsSaveFailedDetail"),
        life: 4000,
      });
    }
  }

  function revertPendingSettings() {
    options.config.value = normalizeConfigByCapabilities({ ...options.persistedConfig.value });
    options.notify({
      severity: "info",
      summary: options.t("app.notify.reverted"),
      detail: options.t("app.notify.revertedDetail"),
      life: 2500,
    });
  }

  async function completeOnboardingSetup() {
    options.config.value = normalizeConfigByCapabilities({ ...options.onboardingDraftConfig.value });
    options.onboardingDismissed.value = true;
    options.onboardingSetupMode.value = false;
    await persistAppState(options.config.value);
    options.persistedConfig.value = { ...options.config.value };
    options.notify({
      severity: "success",
      summary: options.t("app.notify.onboardingDone"),
      detail: options.t("app.notify.onboardingDoneDetail"),
      life: 2500,
    });
  }

  function updateConfig(nextConfig: ClientConfig) {
    options.config.value = normalizeConfigByCapabilities({ ...nextConfig });
  }

  function updateOnboardingDraftConfig(nextConfig: ClientConfig) {
    options.onboardingDraftConfig.value = normalizeConfigByCapabilities({ ...nextConfig });
  }

  return {
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
  };
}

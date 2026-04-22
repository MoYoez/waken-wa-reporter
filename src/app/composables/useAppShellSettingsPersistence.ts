import {
  isAutostartEnabled,
  restartApp as requestAppRestart,
  setAutostartEnabled,
} from "@/lib/api";
import { readDeviceName } from "@/lib/deviceInfo";
import {
  AppStateSaveError,
  AppStateSaveTransportError,
  saveAppState,
} from "@/lib/persistence";
import { setI18nLocale } from "@/i18n";
import { resolveApiErrorMessage } from "@/lib/localizedText";
import {
  REPORTER_TIMING_FIELDS,
  formatReporterTimingIssue,
  validateReporterTimingConfig,
  type ReporterTimingField,
  type ReporterTimingIssue,
  type ReporterTimingIssueReason,
} from "@/lib/reporterTimingValidation";
import type { ClientConfig } from "@/types";

import type {
  NormalizeConfigByCapabilities,
  UseAppShellPersistenceOptions,
} from "@/app/composables/appShellPersistenceTypes";

const reporterTimingIssueReasons = new Set<ReporterTimingIssueReason>([
  "empty",
  "notInteger",
  "tooLarge",
  "tooSmall",
]);

class SettingsSaveStageError extends Error {
  stageKey: string;
  originalError: unknown;

  constructor(stageKey: string, originalError: unknown) {
    super("");
    this.name = "SettingsSaveStageError";
    this.stageKey = stageKey;
    this.originalError = originalError;
  }
}

export function useAppShellSettingsPersistence(options: UseAppShellPersistenceOptions) {
  const normalizeConfigByCapabilities: NormalizeConfigByCapabilities = (raw: ClientConfig) => {
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
  };

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

  function isReporterTimingField(field: unknown): field is ReporterTimingField {
    return field === "pollIntervalMs" || field === "heartbeatIntervalMs";
  }

  function backendIssueToReporterTimingIssue(
    issue: AppStateSaveError["issues"][number],
  ): ReporterTimingIssue | null {
    if (!isReporterTimingField(issue.field) || !reporterTimingIssueReasons.has(issue.reason as ReporterTimingIssueReason)) {
      return null;
    }

    const meta = REPORTER_TIMING_FIELDS[issue.field];
    const received = typeof issue.received === "string" || typeof issue.received === "number"
      ? issue.received
      : JSON.stringify(issue.received ?? "");

    return {
      field: issue.field,
      path: issue.path || `config.${issue.field}`,
      reason: issue.reason as ReporterTimingIssueReason,
      received,
      min: typeof issue.min === "number" ? issue.min : meta.min,
      suggestedValue: typeof issue.suggestedValue === "number"
        ? issue.suggestedValue
        : meta.defaultValue,
    };
  }

  function prefixStage(stageKey: string, detail: string) {
    return `${options.t(stageKey)}：${detail}`;
  }

  function errorDetail(error: unknown, fallback: string): string {
    if (error instanceof SettingsSaveStageError) {
      return prefixStage(error.stageKey, errorDetail(error.originalError, fallback));
    }

    if (error instanceof AppStateSaveError) {
      const timingIssues = error.issues
        .map(backendIssueToReporterTimingIssue)
        .filter((issue): issue is ReporterTimingIssue => Boolean(issue));

      if (timingIssues.length) {
        return prefixStage(
          "app.notify.settingsBackendValidationStage",
          timingIssues.map((issue) => formatReporterTimingIssue(issue, options.t)).join("\n"),
        );
      }

      return prefixStage(
        "app.notify.settingsBackendValidationStage",
        resolveApiErrorMessage(error.apiError, options.t, error.message || fallback),
      );
    }

    if (error instanceof AppStateSaveTransportError) {
      return prefixStage("app.notify.settingsTauriCommandStage", error.message || fallback);
    }

    if (error instanceof Error && error.message.trim()) {
      return error.message;
    }

    if (typeof error === "string" && error.trim()) {
      return error;
    }

    return fallback;
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
          detail: errorDetail(error, options.t("app.notify.settingsSaveFailedDetail")),
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
        detail: errorDetail(error, options.t("app.notify.restartFailedDetail")),
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
      const timingIssues = options.reporterSupported.value
        ? validateReporterTimingConfig(nextConfig)
        : [];

      if (timingIssues.length) {
        options.notify({
          severity: "warn",
          summary: options.t("app.notify.settingsInvalid"),
          detail: prefixStage(
            "app.notify.settingsFrontendValidationStage",
            timingIssues.map((issue) => formatReporterTimingIssue(issue, options.t)).join("\n"),
          ),
          life: 5000,
        });
        return;
      }

      const autostartChanged = options.autostartSupported.value
        && nextConfig.launchOnStartup !== options.persistedConfig.value.launchOnStartup;
      const localStateConfig = autostartChanged
        ? {
          ...nextConfig,
          launchOnStartup: options.persistedConfig.value.launchOnStartup,
        }
        : nextConfig;

      options.config.value = localStateConfig;
      try {
        await persistAppState(localStateConfig);
      } catch (error) {
        throw new SettingsSaveStageError("app.notify.settingsLocalStateStage", error);
      }
      options.persistedConfig.value = { ...localStateConfig };

      if (autostartChanged) {
        try {
          await setAutostartEnabled(nextConfig.launchOnStartup);
        } catch (error) {
          options.notify({
            severity: "warn",
            summary: options.t("app.notify.settingsPartiallySaved"),
            detail: `${options.t("app.notify.settingsSavedExceptAutostart")} ${errorDetail(
              new SettingsSaveStageError("app.notify.settingsAutostartStage", error),
              options.t("app.notify.settingsSaveFailedDetail"),
            )}`,
            life: 6000,
          });
          return;
        }

        const finalConfig = {
          ...localStateConfig,
          launchOnStartup: nextConfig.launchOnStartup,
        };
        options.config.value = finalConfig;
        try {
          await persistAppState(finalConfig);
        } catch (error) {
          throw new SettingsSaveStageError("app.notify.settingsLocalStateStage", error);
        }
        options.persistedConfig.value = { ...finalConfig };
      }

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
        detail: errorDetail(error, options.t("app.notify.settingsSaveFailedDetail")),
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

  function updateConfig(nextConfig: ClientConfig) {
    options.config.value = normalizeConfigByCapabilities({ ...nextConfig });
  }

  function updateOnboardingDraftConfig(nextConfig: ClientConfig) {
    options.onboardingDraftConfig.value = normalizeConfigByCapabilities({ ...nextConfig });
  }

  return {
    applyLocale,
    applySettingsChanges,
    handleRestartApp,
    hydrateDeviceNameFromSystem,
    normalizeConfigByCapabilities,
    notifyImport,
    persistAppState,
    rememberVerifiedGeneratedHashKey,
    resolveAutostartConfig,
    revertPendingSettings,
    updateConfig,
    updateOnboardingDraftConfig,
  };
}

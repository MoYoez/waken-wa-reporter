import type {
  NormalizeConfigByCapabilities,
  PersistAppState,
  UseAppShellPersistenceOptions,
} from "@/app/composables/appShellPersistenceTypes";

export function useAppShellOnboardingPersistence(
  options: UseAppShellPersistenceOptions,
  dependencies: {
    normalizeConfigByCapabilities: NormalizeConfigByCapabilities;
    persistAppState: PersistAppState;
  },
) {
  function closeOnboarding() {
    options.onboardingSetupMode.value = false;
    options.onboardingDismissed.value = true;
    void dependencies.persistAppState();
  }

  function startSetup() {
    options.reporterConfigPromptHandled.value = true;
    options.onboardingDraftConfig.value = { ...options.config.value };
    options.onboardingSetupMode.value = true;
  }

  function skipExistingReporterConfig() {
    options.reporterConfigPromptHandled.value = true;
    void dependencies.persistAppState();
  }

  async function useExistingReporterConfig() {
    if (!options.existingReporterConfig.value?.config) {
      return;
    }

    options.importingReporterConfig.value = true;
    options.onboardingDraftConfig.value = dependencies.normalizeConfigByCapabilities({
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

  async function completeOnboardingSetup() {
    options.config.value = dependencies.normalizeConfigByCapabilities({
      ...options.onboardingDraftConfig.value,
    });
    options.onboardingDismissed.value = true;
    options.onboardingSetupMode.value = false;
    await dependencies.persistAppState(options.config.value);
    options.persistedConfig.value = { ...options.config.value };
    options.notify({
      severity: "success",
      summary: options.t("app.notify.onboardingDone"),
      detail: options.t("app.notify.onboardingDoneDetail"),
      life: 2500,
    });
  }

  return {
    closeOnboarding,
    completeOnboardingSetup,
    skipExistingReporterConfig,
    startSetup,
    useExistingReporterConfig,
  };
}

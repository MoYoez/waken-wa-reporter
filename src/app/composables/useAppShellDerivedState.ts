import { computed, type Ref } from "vue";

import type { SupportedLocale } from "@/i18n";
import type { ClientCapabilities, ClientConfig } from "@/types";

interface UseAppShellDerivedStateOptions {
  capabilities: Ref<ClientCapabilities>;
  config: Ref<ClientConfig>;
  currentLocale: Ref<SupportedLocale>;
  hydrated: Ref<boolean>;
  onboardingDismissed: Ref<boolean>;
  localeSaving: Ref<boolean>;
  restartingApp: Ref<boolean>;
  startupLocale: Ref<SupportedLocale>;
}

export function useAppShellDerivedState(options: UseAppShellDerivedStateOptions) {
  const reporterSupported = computed(() => options.capabilities.value.realtimeReporter);
  const discordSupported = computed(() => options.capabilities.value.discordPresence);
  const traySupported = computed(() => options.capabilities.value.tray);
  const autostartSupported = computed(() => options.capabilities.value.autostart);
  const isNativeNotice = computed(() => !reporterSupported.value);
  const readiness = computed(() => {
    const required = [
      options.config.value.baseUrl.trim(),
      options.config.value.apiToken.trim(),
      options.config.value.generatedHashKey.trim(),
    ];
    return required.every(Boolean);
  });
  const discordReadiness = computed(
    () => !!options.config.value.baseUrl.trim() && !!options.config.value.discordApplicationId.trim(),
  );
  const shouldShowOnboarding = computed(
    () => options.hydrated.value && !options.onboardingDismissed.value && !readiness.value,
  );
  const localeRestartRequired = computed(
    () => options.currentLocale.value !== options.startupLocale.value,
  );
  const settingsRestarting = computed(
    () => options.restartingApp.value || options.localeSaving.value,
  );

  return {
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
  };
}

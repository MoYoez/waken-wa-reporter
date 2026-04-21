import type { UseAppShellPersistenceOptions } from "@/app/composables/appShellPersistenceTypes";
import { useAppShellOnboardingPersistence } from "@/app/composables/useAppShellOnboardingPersistence";
import { useAppShellSettingsPersistence } from "@/app/composables/useAppShellSettingsPersistence";

export function useAppShellPersistence(options: UseAppShellPersistenceOptions) {
  const settingsPersistence = useAppShellSettingsPersistence(options);
  const onboardingPersistence = useAppShellOnboardingPersistence(options, {
    normalizeConfigByCapabilities: settingsPersistence.normalizeConfigByCapabilities,
    persistAppState: settingsPersistence.persistAppState,
  });

  return {
    ...settingsPersistence,
    ...onboardingPersistence,
  };
}

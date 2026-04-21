import type { ComputedRef, Ref } from "vue";

import type { NotifyPayload } from "@/lib/notify";
import type {
  ClientConfig,
  DeviceType,
  ExistingReporterConfig,
  RecentPreset,
} from "@/types";
import type { SupportedLocale } from "@/i18n";

export interface UseAppShellPersistenceOptions {
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

export type NormalizeConfigByCapabilities = (raw: ClientConfig) => ClientConfig;
export type PersistAppState = (configOverride?: ClientConfig) => Promise<void>;

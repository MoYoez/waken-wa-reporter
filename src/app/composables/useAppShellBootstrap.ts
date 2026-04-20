import type { ComputedRef, Ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { SingleInstanceAttemptPayload } from "@/app/types";
import {
  discoverExistingReporterConfig,
  getClientCapabilities,
} from "@/lib/api";
import type { NotifyPayload } from "@/lib/notify";
import { loadAppState } from "@/lib/persistence";
import type {
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  ExistingReporterConfig,
  RecentPreset,
  RealtimeReporterSnapshot,
} from "@/types";
import type { SupportedLocale } from "@/i18n";

interface UseAppShellBootstrapOptions {
  t: (key: string, params?: Record<string, unknown>) => string;
  locale: Ref<string>;
  notify: (payload: NotifyPayload) => void;
  capabilities: Ref<ClientCapabilities>;
  config: Ref<ClientConfig>;
  persistedConfig: Ref<ClientConfig>;
  onboardingDraftConfig: Ref<ClientConfig>;
  recentPresets: Ref<RecentPreset[]>;
  currentLocale: Ref<SupportedLocale>;
  startupLocale: Ref<SupportedLocale>;
  hydrated: Ref<boolean>;
  onboardingDismissed: Ref<boolean>;
  reporterConfigPromptHandled: Ref<boolean>;
  existingReporterConfig: Ref<ExistingReporterConfig | null>;
  verifiedGeneratedHashKey: Ref<string>;
  reporterSnapshot: Ref<RealtimeReporterSnapshot>;
  discordPresenceSnapshot: Ref<DiscordPresenceSnapshot>;
  reporterSupported: ComputedRef<boolean>;
  readiness: ComputedRef<boolean>;
  discordReadiness: ComputedRef<boolean>;
  applyLocale: (nextLocale: string, persist?: boolean) => Promise<void>;
  resolveAutostartConfig: (raw: ClientConfig) => Promise<ClientConfig>;
  normalizeConfigByCapabilities: (raw: ClientConfig) => ClientConfig;
  persistAppState: (configOverride?: ClientConfig) => Promise<void>;
  hydrateDeviceNameFromSystem: () => Promise<void>;
  ensureVisibleSection: () => void;
  syncDeviceTypeByViewport: () => void;
  refreshReporterSnapshot: () => Promise<void>;
  syncReporterPolling: () => void;
  refreshDiscordPresenceSnapshot: () => Promise<void>;
  syncDiscordPresencePolling: () => void;
  handleStartReporter: () => Promise<void>;
  handleStartDiscordPresence: () => Promise<void>;
}

export function useAppShellBootstrap(options: UseAppShellBootstrapOptions) {
  async function listenForSingleInstanceAttempt() {
    try {
      return await listen<SingleInstanceAttemptPayload>(
        "single-instance-attempted",
        () => {
          options.notify({
            severity: "warn",
            summary: options.t("app.notify.singleInstanceDetected"),
            detail: options.t("app.notify.singleInstanceDetectedDetail"),
            life: 3500,
          });
        },
      );
    } catch {
      // Ignore event-listener setup failures outside the Tauri runtime.
      return undefined;
    }
  }

  async function hydrateCapabilities() {
    const capabilitiesResult = await getClientCapabilities();
    if (capabilitiesResult.success && capabilitiesResult.data) {
      options.capabilities.value = capabilitiesResult.data;
    }
  }

  async function hydratePersistedState() {
    const state = await loadAppState();
    void options.applyLocale(state.locale || options.locale.value);
    options.startupLocale.value = options.currentLocale.value;

    const normalized = await options.resolveAutostartConfig(
      options.normalizeConfigByCapabilities(state.config),
    );

    options.config.value = normalized;
    options.persistedConfig.value = { ...normalized };
    options.onboardingDraftConfig.value = { ...normalized };
    options.recentPresets.value = state.recentPresets;
    options.onboardingDismissed.value = state.onboardingDismissed;
    options.verifiedGeneratedHashKey.value = state.verifiedGeneratedHashKey?.trim() ?? "";
    options.reporterConfigPromptHandled.value = options.reporterSupported.value
      ? (state.reporterConfigPromptHandled ?? false)
      : true;
    options.hydrated.value = true;

    if (normalized.launchOnStartup !== Boolean(state.config.launchOnStartup)) {
      void options.persistAppState(normalized);
    }
  }

  async function hydrateShellState() {
    await hydrateCapabilities();
    await hydratePersistedState();
    await options.hydrateDeviceNameFromSystem();

    options.ensureVisibleSection();
    options.syncDeviceTypeByViewport();
  }

  async function discoverReporterConfigIfNeeded() {
    if (!options.reporterSupported.value || options.reporterConfigPromptHandled.value) {
      return;
    }

    const reporterConfigResult = await discoverExistingReporterConfig();
    if (reporterConfigResult.success && reporterConfigResult.data?.found) {
      options.existingReporterConfig.value = reporterConfigResult.data;
    }
  }

  async function initializeRuntimeServices() {
    if (!options.reporterSupported.value) {
      return;
    }

    await options.refreshReporterSnapshot();
    options.syncReporterPolling();
    await options.refreshDiscordPresenceSnapshot();
    options.syncDiscordPresencePolling();

    if (
      options.config.value.reporterEnabled
      && !options.reporterSnapshot.value.running
      && options.readiness.value
    ) {
      void options.handleStartReporter();
    }

    if (
      options.config.value.discordEnabled
      && !options.discordPresenceSnapshot.value.running
      && options.discordReadiness.value
    ) {
      void options.handleStartDiscordPresence();
    }
  }

  async function bootstrapAppShell(): Promise<UnlistenFn | undefined> {
    const unlistenSingleInstance = await listenForSingleInstanceAttempt();

    await hydrateShellState();
    await discoverReporterConfigIfNeeded();
    await initializeRuntimeServices();

    return unlistenSingleInstance;
  }

  return {
    bootstrapAppShell,
  };
}

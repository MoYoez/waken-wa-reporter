import { ref } from "vue";

import { normalizeLocale, type SupportedLocale } from "@/i18n";
import { defaultClientConfig } from "@/lib/persistence";
import type {
  ClientCapabilities,
  DiscordPresenceSnapshot,
  ExistingReporterConfig,
  MobileConnectivityState,
  RealtimeReporterSnapshot,
  RecentPreset,
} from "@/types";

export function createDefaultCapabilities(): ClientCapabilities {
  return {
    realtimeReporter: true,
    tray: true,
    platformSelfTest: true,
    discordPresence: true,
    autostart: true,
  };
}

export function useAppShellState(locale: string, pendingSummary: string) {
  const normalizedLocale = normalizeLocale(locale);

  const capabilities = ref<ClientCapabilities>(createDefaultCapabilities());
  const config = ref(defaultClientConfig());
  const persistedConfig = ref(defaultClientConfig());
  const onboardingDraftConfig = ref(defaultClientConfig());
  const recentPresets = ref<RecentPreset[]>([]);
  const currentLocale = ref<SupportedLocale>(normalizedLocale);
  const startupLocale = ref<SupportedLocale>(normalizedLocale);
  const hydrated = ref(false);
  const onboardingDismissed = ref(false);
  const reporterConfigPromptHandled = ref(false);
  const reporterBusy = ref(false);
  const discordBusy = ref(false);
  const importingReporterConfig = ref(false);
  const existingReporterConfig = ref<ExistingReporterConfig | null>(null);
  const verifiedGeneratedHashKey = ref("");
  const mobileConnectivity = ref<MobileConnectivityState>(createInitialMobileConnectivity(pendingSummary));
  const reporterSnapshot = ref<RealtimeReporterSnapshot>(createInitialReporterSnapshot());
  const discordPresenceSnapshot = ref<DiscordPresenceSnapshot>(createInitialDiscordPresenceSnapshot());
  const pendingApprovalDialogVisible = ref(false);
  const lastPendingApprovalSeen = ref("");
  const lastMobileConnectivitySignature = ref("");
  const onboardingSetupMode = ref(false);
  const localeSaving = ref(false);
  const restartingApp = ref(false);

  return {
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
  };
}

function createInitialMobileConnectivity(summary: string): MobileConnectivityState {
  return {
    checking: false,
    checked: false,
    ok: null,
    summary,
    detail: "",
    checkedAt: null,
  };
}

function createInitialReporterSnapshot(): RealtimeReporterSnapshot {
  return {
    running: false,
    logs: [],
    currentActivity: null,
    lastHeartbeatAt: null,
    lastError: null,
    lastPendingApprovalMessage: null,
    lastPendingApprovalUrl: null,
  };
}

function createInitialDiscordPresenceSnapshot(): DiscordPresenceSnapshot {
  return {
    running: false,
    connected: false,
    lastSyncAt: null,
    lastError: null,
    currentSummary: null,
  };
}

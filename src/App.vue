<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useI18n } from "vue-i18n";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import Tag from "primevue/tag";
import Toast from "primevue/toast";
import { useToast } from "primevue/usetoast";

import ActivityWorkspace from "./components/ActivityWorkspace.vue";
import ConnectionPanel from "./components/ConnectionPanel.vue";
import InspirationWorkspace from "./components/InspirationWorkspace.vue";
import OverviewWorkspace from "./components/OverviewWorkspace.vue";
import RealtimeWorkspace from "./components/RealtimeWorkspace.vue";
import SettingsWorkspace from "./components/SettingsWorkspace.vue";
import {
  discoverExistingReporterConfig,
  extractPendingApprovalInfo,
  formatPendingApprovalDetail,
  getClientCapabilities,
  getDiscordPresenceSnapshot,
  getRealtimeReporterSnapshot,
  isAutostartEnabled,
  probeConnectivity,
  restartApp as requestAppRestart,
  setAutostartEnabled,
  startDiscordPresenceSync,
  startRealtimeReporter,
  stopDiscordPresenceSync,
  stopRealtimeReporter,
} from "./lib/api";
import { readDeviceName } from "./lib/deviceInfo";
import {
  normalizeLocale,
  setI18nLocale,
  type SupportedLocale,
} from "./i18n";
import { resolveApiErrorMessage } from "./lib/localizedText";
import { createNotifier } from "./lib/notify";
import { defaultClientConfig, loadAppState, saveAppState } from "./lib/persistence";
import type {
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  DeviceType,
  ExistingReporterConfig,
  MobileConnectivityState,
  PendingApprovalInfo,
  RealtimeReporterSnapshot,
  RecentPreset,
} from "./types";

type AppSection = "overview" | "settings" | "activity" | "realtime" | "inspiration";

interface SectionNavItem {
  key: AppSection;
  title: string;
  kicker: string;
  icon: string;
  requiresRealtime?: boolean;
}

interface SingleInstanceAttemptPayload {
  args: string[];
  cwd: string;
}

const defaultCapabilities: ClientCapabilities = {
  realtimeReporter: true,
  tray: true,
  platformSelfTest: true,
  discordPresence: true,
  autostart: true,
};

const { t, locale } = useI18n();

const toast = useToast();

const capabilities = ref<ClientCapabilities>(defaultCapabilities);
const config = ref<ClientConfig>(defaultClientConfig());
const persistedConfig = ref<ClientConfig>(defaultClientConfig());
const onboardingDraftConfig = ref<ClientConfig>(defaultClientConfig());
const recentPresets = ref<RecentPreset[]>([]);
const currentLocale = ref<SupportedLocale>(normalizeLocale(locale.value));
const startupLocale = ref<SupportedLocale>(normalizeLocale(locale.value));
const activeSection = ref<AppSection>("overview");
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
const viewportWidth = ref<number>(1200);
const localeSaving = ref(false);
const restartingApp = ref(false);

let reporterPollingTimer: number | undefined;
let reporterSnapshotRefreshInFlight = false;
let discordPollingTimer: number | undefined;
let discordSnapshotRefreshInFlight = false;
let unlistenSingleInstance: UnlistenFn | undefined;
const REPORTER_SNAPSHOT_POLL_MS = 5000;

const sections = computed<SectionNavItem[]>(() => [
  {
    key: "overview",
    title: t("app.sections.overview.title"),
    kicker: t("app.sections.overview.kicker"),
    icon: "pi pi-home",
  },
  {
    key: "inspiration",
    title: t("app.sections.inspiration.title"),
    kicker: t("app.sections.inspiration.kicker"),
    icon: "pi pi-file-edit",
  },
  {
    key: "activity",
    title: t("app.sections.activity.title"),
    kicker: t("app.sections.activity.kicker"),
    icon: "pi pi-pencil",
  },
  {
    key: "realtime",
    title: t("app.sections.realtime.title"),
    kicker: t("app.sections.realtime.kicker"),
    icon: "pi pi-chart-line",
    requiresRealtime: true,
  },
  {
    key: "settings",
    title: t("app.sections.settings.title"),
    kicker: t("app.sections.settings.kicker"),
    icon: "pi pi-cog",
  },
]);

const reporterSupported = computed(() => capabilities.value.realtimeReporter);
const discordSupported = computed(() => capabilities.value.discordPresence);
const traySupported = computed(() => capabilities.value.tray);
const autostartSupported = computed(() => capabilities.value.autostart);
const isPhone = computed(() => viewportWidth.value < 900);
const isNativeNotice = computed(() => !reporterSupported.value);
const { notify } = createNotifier(toast, () => isNativeNotice.value);

const visibleSections = computed(() =>
  sections.value.filter((section) => !section.requiresRealtime || reporterSupported.value),
);

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

function ensureVisibleSection() {
  if (!visibleSections.value.some((section) => section.key === activeSection.value)) {
    activeSection.value = visibleSections.value[0]?.key ?? "overview";
  }
}

function inferMobileDeviceType(): DeviceType {
  return isPhone.value ? "mobile" : "tablet";
}

function normalizeConfigByCapabilities(raw: ClientConfig): ClientConfig {
  const normalizedDevice = raw.device.trim();
  const launchOnStartup = autostartSupported.value ? raw.launchOnStartup : false;

  if (reporterSupported.value) {
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
    deviceType: inferMobileDeviceType(),
    pushMode: "active",
    reporterEnabled: false,
    discordEnabled: false,
    launchOnStartup: false,
  };
}

async function resolveAutostartConfig(raw: ClientConfig) {
  if (!autostartSupported.value) {
    return { ...raw, launchOnStartup: false };
  }

  try {
    const enabled = await isAutostartEnabled();
    return { ...raw, launchOnStartup: enabled };
  } catch {
    return raw;
  }
}

function syncDeviceTypeByViewport() {
  const nextType = reporterSupported.value ? "desktop" : inferMobileDeviceType();

  if (config.value.deviceType !== nextType) {
    config.value = { ...config.value, deviceType: nextType };
  }

  if (onboardingDraftConfig.value.deviceType !== nextType) {
    onboardingDraftConfig.value = { ...onboardingDraftConfig.value, deviceType: nextType };
  }
}

function onViewportResize() {
  viewportWidth.value = window.innerWidth;
  syncDeviceTypeByViewport();
}

function onVisibilityChange() {
  syncReporterPolling();
  syncDiscordPresencePolling();
  if (shouldPollReporterSnapshot()) {
    void refreshReporterSnapshot();
  }
  if (shouldPollDiscordPresenceSnapshot()) {
    void refreshDiscordPresenceSnapshot();
  }
}

function handlePresetSaved(preset: RecentPreset) {
  const deduped = recentPresets.value.filter(
    (item) =>
      item.process_name !== preset.process_name ||
      item.process_title !== preset.process_title,
  );
  recentPresets.value = [preset, ...deduped].slice(0, 6);
  void persistAppState();
}

async function applyLocale(nextLocale: string, persist = false) {
  const normalized = setI18nLocale(nextLocale);
  currentLocale.value = normalized;

  if (persist && hydrated.value) {
    localeSaving.value = true;
    try {
      await persistAppState();
    } catch (error) {
      notify({
        severity: "error",
        summary: t("app.notify.settingsSaveFailed"),
        detail: error instanceof Error ? error.message : t("app.notify.settingsSaveFailedDetail"),
        life: 4000,
      });
    } finally {
      localeSaving.value = false;
    }
  }
}

function translateText(key: string, params?: Record<string, unknown>) {
  return params ? t(key, params) : t(key);
}

function apiErrorDetail(
  error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
  fallback: string,
) {
  return resolveApiErrorMessage(error, translateText, fallback);
}

function notifyImport(message: string) {
  notify({
    severity: "success",
    summary: t("app.notify.importedConfig"),
    detail: message,
    life: 3000,
  });
}

function rememberVerifiedGeneratedHashKey(nextKey: string) {
  const normalized = nextKey.trim();
  if (!normalized || verifiedGeneratedHashKey.value === normalized) {
    return;
  }

  verifiedGeneratedHashKey.value = normalized;
  void persistAppState();
}

function handlePendingApproval(info: PendingApprovalInfo) {
  reporterSnapshot.value.lastPendingApprovalMessage = info.message;
  reporterSnapshot.value.lastPendingApprovalUrl = info.approvalUrl ?? null;
  pendingApprovalDialogVisible.value = true;
}

function resetMobileConnectivity(summary = t("app.mobileConnectivity.pending"), detail = "") {
  mobileConnectivity.value = {
    checking: false,
    checked: false,
    ok: null,
    summary,
    detail,
    checkedAt: null,
  };
}

async function runMobileConnectivityProbe(force = false) {
  if (reporterSupported.value) {
    resetMobileConnectivity();
    return;
  }

  if (!readiness.value) {
    lastMobileConnectivitySignature.value = "";
    resetMobileConnectivity(
      t("app.mobileConnectivity.waitingConfig"),
      t("app.mobileConnectivity.waitingConfigDetail"),
    );
    return;
  }

  const signature = [
    config.value.baseUrl.trim(),
    config.value.apiToken.trim(),
    config.value.generatedHashKey.trim(),
  ].join("|");

  if (!force && mobileConnectivity.value.checked && lastMobileConnectivitySignature.value === signature) {
    return;
  }

  lastMobileConnectivitySignature.value = signature;
  mobileConnectivity.value = {
    checking: true,
    checked: false,
    ok: null,
    summary: t("app.mobileConnectivity.checking"),
    detail: t("app.mobileConnectivity.checkingDetail"),
    checkedAt: null,
  };

  const result = await probeConnectivity(config.value);
  const checkedAt = new Date().toISOString();
  const pendingApproval = extractPendingApprovalInfo(result);

  if (pendingApproval) {
    mobileConnectivity.value = {
      checking: false,
      checked: true,
      ok: false,
      summary: t("app.mobileConnectivity.pendingApproval"),
      detail: formatPendingApprovalDetail(pendingApproval),
      checkedAt,
    };
    handlePendingApproval(pendingApproval);
    return;
  }

  if (result.success) {
    rememberVerifiedGeneratedHashKey(config.value.generatedHashKey.trim());
    mobileConnectivity.value = {
      checking: false,
      checked: true,
      ok: true,
      summary: t("app.mobileConnectivity.passed"),
      detail: t("app.mobileConnectivity.passedDetail"),
      checkedAt,
    };
    return;
  }

  let summary = t("app.mobileConnectivity.failed");
  let detail = apiErrorDetail(result.error, t("app.mobileConnectivity.failedDetail"));

  if (result.status === 401) {
    summary = t("app.mobileConnectivity.tokenUnavailable");
    detail = t("app.mobileConnectivity.tokenUnavailableDetail");
  } else if (result.status === 403) {
    summary = t("app.mobileConnectivity.deviceUnavailable");
    detail = apiErrorDetail(result.error, t("app.mobileConnectivity.deviceUnavailableDetail"));
  } else if (result.status === 400) {
    summary = t("app.mobileConnectivity.configIncomplete");
    detail = apiErrorDetail(result.error, t("app.mobileConnectivity.configIncompleteDetail"));
  } else if (result.status === 0) {
    summary = t("app.mobileConnectivity.siteUnreachable");
    detail = apiErrorDetail(result.error, t("app.mobileConnectivity.siteUnreachableDetail"));
  }

  mobileConnectivity.value = {
    checking: false,
    checked: true,
    ok: false,
    summary,
    detail,
    checkedAt,
  };
}

function closeOnboarding() {
  onboardingSetupMode.value = false;
  onboardingDismissed.value = true;
  void persistAppState();
}

function startSetup() {
  reporterConfigPromptHandled.value = true;
  onboardingDraftConfig.value = { ...config.value };
  onboardingSetupMode.value = true;
}

function skipExistingReporterConfig() {
  reporterConfigPromptHandled.value = true;
  void persistAppState();
}

async function useExistingReporterConfig() {
  if (!existingReporterConfig.value?.config) return;
  importingReporterConfig.value = true;
  onboardingDraftConfig.value = normalizeConfigByCapabilities({
    ...existingReporterConfig.value.config,
  });
  reporterConfigPromptHandled.value = true;
  importingReporterConfig.value = false;
  notify({
    severity: "success",
    summary: t("app.notify.importedExistingConfig"),
    detail: t("app.notify.importedExistingConfigDetail"),
    life: 3500,
  });
  onboardingSetupMode.value = true;
}

async function persistAppState(configOverride?: ClientConfig) {
  await saveAppState(
    normalizeConfigByCapabilities(configOverride ?? persistedConfig.value),
    recentPresets.value,
    onboardingDismissed.value,
    currentLocale.value,
    reporterConfigPromptHandled.value,
    verifiedGeneratedHashKey.value,
  );
}

async function handleRestartApp() {
  if (restartingApp.value) {
    return;
  }

  restartingApp.value = true;
  try {
    if (localeSaving.value || localeRestartRequired.value) {
      await persistAppState();
    }
    await requestAppRestart();
  } catch (error) {
    restartingApp.value = false;
    notify({
      severity: "error",
      summary: t("app.notify.restartFailed"),
      detail: error instanceof Error ? error.message : t("app.notify.restartFailedDetail"),
      life: 4000,
    });
  }
}

async function hydrateDeviceNameFromSystem() {
  if (config.value.device.trim()) {
    return;
  }

  try {
    const deviceName = (await readDeviceName()).trim();
    if (!deviceName) {
      return;
    }

    const nextConfig = normalizeConfigByCapabilities({
      ...config.value,
      device: deviceName,
    });
    config.value = nextConfig;
    persistedConfig.value = { ...nextConfig };
    onboardingDraftConfig.value = { ...nextConfig };
    await persistAppState(nextConfig);
  } catch {
    // Ignore plugin read failures and keep backend fallback behavior.
  }
}

async function applySettingsChanges() {
  try {
    const nextConfig = normalizeConfigByCapabilities({ ...config.value });

    if (autostartSupported.value) {
      await setAutostartEnabled(nextConfig.launchOnStartup);
    }

    config.value = nextConfig;
    await persistAppState(nextConfig);
    persistedConfig.value = { ...nextConfig };
    notify({
      severity: "success",
      summary: t("app.notify.settingsSaved"),
      detail: t("app.notify.settingsSavedDetail"),
      life: 2500,
    });
  } catch (error) {
    notify({
      severity: "error",
      summary: t("app.notify.settingsSaveFailed"),
      detail: error instanceof Error ? error.message : t("app.notify.settingsSaveFailedDetail"),
      life: 4000,
    });
  }
}

function revertPendingSettings() {
  config.value = normalizeConfigByCapabilities({ ...persistedConfig.value });
  notify({
    severity: "info",
    summary: t("app.notify.reverted"),
    detail: t("app.notify.revertedDetail"),
    life: 2500,
  });
}

async function completeOnboardingSetup() {
  config.value = normalizeConfigByCapabilities({ ...onboardingDraftConfig.value });
  onboardingDismissed.value = true;
  onboardingSetupMode.value = false;
  await persistAppState(config.value);
  persistedConfig.value = { ...config.value };
  notify({
    severity: "success",
    summary: t("app.notify.onboardingDone"),
    detail: t("app.notify.onboardingDoneDetail"),
    life: 2500,
  });
}

async function refreshReporterSnapshot() {
  if (!reporterSupported.value || activeSection.value === "inspiration") {
    return;
  }
  if (reporterSnapshotRefreshInFlight) {
    return;
  }

  reporterSnapshotRefreshInFlight = true;
  try {
    const result = await getRealtimeReporterSnapshot();
    if (!result.success || !result.data) {
      return;
    }
    Object.assign(reporterSnapshot.value, result.data);
  } finally {
    reporterSnapshotRefreshInFlight = false;
  }
}

async function refreshDiscordPresenceSnapshot() {
  if (!discordSupported.value) {
    return;
  }
  if (discordSnapshotRefreshInFlight) {
    return;
  }

  discordSnapshotRefreshInFlight = true;
  try {
    const result = await getDiscordPresenceSnapshot();
    if (!result.success || !result.data) {
      return;
    }
    Object.assign(discordPresenceSnapshot.value, result.data);
  } finally {
    discordSnapshotRefreshInFlight = false;
  }
}

function closePendingApprovalDialog() {
  pendingApprovalDialogVisible.value = false;
}

function shouldPollReporterSnapshot() {
  return reporterSupported.value
    && reporterSnapshot.value.running
    && document.visibilityState === "visible";
}

function shouldPollDiscordPresenceSnapshot() {
  return discordSupported.value
    && discordPresenceSnapshot.value.running
    && document.visibilityState === "visible";
}

function stopReporterPolling() {
  if (reporterPollingTimer !== undefined) {
    window.clearInterval(reporterPollingTimer);
    reporterPollingTimer = undefined;
  }
}

function stopDiscordPresencePolling() {
  if (discordPollingTimer !== undefined) {
    window.clearInterval(discordPollingTimer);
    discordPollingTimer = undefined;
  }
}

function syncReporterPolling() {
  stopReporterPolling();

  if (!shouldPollReporterSnapshot()) {
    return;
  }

  reporterPollingTimer = window.setInterval(() => {
    if (!shouldPollReporterSnapshot()) {
      stopReporterPolling();
      return;
    }
    void refreshReporterSnapshot();
  }, REPORTER_SNAPSHOT_POLL_MS);
}

function syncDiscordPresencePolling() {
  stopDiscordPresencePolling();

  if (!shouldPollDiscordPresenceSnapshot()) {
    return;
  }

  discordPollingTimer = window.setInterval(() => {
    if (!shouldPollDiscordPresenceSnapshot()) {
      stopDiscordPresencePolling();
      return;
    }
    void refreshDiscordPresenceSnapshot();
  }, REPORTER_SNAPSHOT_POLL_MS);
}

async function handleStartReporter() {
  if (!reporterSupported.value || reporterBusy.value) {
    return;
  }

  reporterBusy.value = true;
  const reporterConfig = {
    ...config.value,
    pushMode: "realtime" as const,
  };
  const result = await startRealtimeReporter(reporterConfig);
  reporterBusy.value = false;

  if (!result.success || !result.data) {
    notify({
      severity: "error",
      summary: t("app.notify.reporterStartFailed"),
      detail: apiErrorDetail(result.error, t("app.notify.reporterStartFailedDetail")),
      life: 4000,
    });
    return;
  }

  Object.assign(reporterSnapshot.value, result.data);
  notify({
    severity: "success",
    summary: t("app.notify.reporterStarted"),
    detail: t("app.notify.reporterStartedDetail"),
    life: 3000,
  });
}

async function handleStopReporter() {
  if (!reporterSupported.value || reporterBusy.value) {
    return;
  }

  reporterBusy.value = true;
  const result = await stopRealtimeReporter();
  reporterBusy.value = false;

  if (!result.success || !result.data) {
    notify({
      severity: "error",
      summary: t("app.notify.reporterStopFailed"),
      detail: apiErrorDetail(result.error, t("app.notify.reporterStopFailedDetail")),
      life: 4000,
    });
    return;
  }

  Object.assign(reporterSnapshot.value, result.data);
  notify({
    severity: "success",
    summary: t("app.notify.reporterStopped"),
    detail: t("app.notify.reporterStoppedDetail"),
    life: 3000,
  });
}

async function handleStartDiscordPresence() {
  if (!discordSupported.value || discordBusy.value) {
    return;
  }

  discordBusy.value = true;
  const result = await startDiscordPresenceSync(config.value);
  discordBusy.value = false;

  if (!result.success || !result.data) {
    notify({
      severity: "error",
      summary: t("app.notify.discordStartFailed"),
      detail: apiErrorDetail(result.error, t("app.notify.discordStartFailedDetail")),
      life: 4000,
    });
    return;
  }

  Object.assign(discordPresenceSnapshot.value, result.data);
  notify({
    severity: "success",
    summary: t("app.notify.discordStarted"),
    detail: t("app.notify.discordStartedDetail"),
    life: 3000,
  });
}

async function handleStopDiscordPresence() {
  if (!discordSupported.value || discordBusy.value) {
    return;
  }

  discordBusy.value = true;
  const result = await stopDiscordPresenceSync();
  discordBusy.value = false;

  if (!result.success || !result.data) {
    notify({
      severity: "error",
      summary: t("app.notify.discordStopFailed"),
      detail: apiErrorDetail(result.error, t("app.notify.discordStopFailedDetail")),
      life: 4000,
    });
    return;
  }

  Object.assign(discordPresenceSnapshot.value, result.data);
  notify({
    severity: "success",
    summary: t("app.notify.discordStopped"),
    detail: t("app.notify.discordStoppedDetail"),
    life: 3000,
  });
}

watch(
  () => [
    reporterSnapshot.value.lastPendingApprovalMessage,
    reporterSnapshot.value.lastPendingApprovalUrl,
  ],
  ([message, url]) => {
    const nextKey = `${message ?? ""}|${url ?? ""}`;
    if (!message || nextKey === "|" || nextKey === lastPendingApprovalSeen.value) {
      return;
    }
    lastPendingApprovalSeen.value = nextKey;
    pendingApprovalDialogVisible.value = true;
  },
  { immediate: true },
);

watch(
  () => activeSection.value,
  (section) => {
    if (reporterSupported.value && section !== "inspiration") {
      void refreshReporterSnapshot();
    }
    if (discordSupported.value) {
      void refreshDiscordPresenceSnapshot();
    }
    syncReporterPolling();
    syncDiscordPresencePolling();
  },
);

watch(
  () => visibleSections.value.map((item) => item.key).join(","),
  () => {
    ensureVisibleSection();
  },
);

watch(
  () => capabilities.value,
  () => {
    syncDeviceTypeByViewport();
    syncReporterPolling();
    syncDiscordPresencePolling();
  },
  { deep: true },
);

watch(
  () => reporterSnapshot.value.running,
  () => {
    syncReporterPolling();
  },
  { immediate: true },
);

watch(
  () => discordPresenceSnapshot.value.running,
  () => {
    syncDiscordPresencePolling();
  },
  { immediate: true },
);

watch(
  () => locale.value,
  () => {
    currentLocale.value = normalizeLocale(locale.value);
    if (hydrated.value) {
      void runMobileConnectivityProbe(true);
    }
  },
);

watch(
  () => [
    hydrated.value,
    reporterSupported.value,
    readiness.value,
    config.value.baseUrl.trim(),
    config.value.apiToken.trim(),
    config.value.generatedHashKey.trim(),
  ].join("|"),
  () => {
    if (!hydrated.value) {
      return;
    }
    void runMobileConnectivityProbe();
  },
  { immediate: true },
);

onMounted(async () => {
  viewportWidth.value = window.innerWidth;
  window.addEventListener("resize", onViewportResize);
  document.addEventListener("visibilitychange", onVisibilityChange);

  try {
    unlistenSingleInstance = await listen<SingleInstanceAttemptPayload>(
      "single-instance-attempted",
      () => {
        notify({
          severity: "warn",
          summary: t("app.notify.singleInstanceDetected"),
          detail: t("app.notify.singleInstanceDetectedDetail"),
          life: 3500,
        });
      },
    );
  } catch {
    // Ignore event-listener setup failures outside the Tauri runtime.
  }

  const capabilitiesResult = await getClientCapabilities();
  if (capabilitiesResult.success && capabilitiesResult.data) {
    capabilities.value = capabilitiesResult.data;
  }

  const state = await loadAppState();
  applyLocale(state.locale || locale.value);
  startupLocale.value = currentLocale.value;
  const normalized = await resolveAutostartConfig(
    normalizeConfigByCapabilities(state.config),
  );
  config.value = normalized;
  persistedConfig.value = { ...normalized };
  onboardingDraftConfig.value = { ...normalized };
  recentPresets.value = state.recentPresets;
  onboardingDismissed.value = state.onboardingDismissed;
  verifiedGeneratedHashKey.value = state.verifiedGeneratedHashKey?.trim() ?? "";
  reporterConfigPromptHandled.value = reporterSupported.value
    ? (state.reporterConfigPromptHandled ?? false)
    : true;
  hydrated.value = true;

  if (normalized.launchOnStartup !== Boolean(state.config.launchOnStartup)) {
    void persistAppState(normalized);
  }

  await hydrateDeviceNameFromSystem();

  ensureVisibleSection();
  syncDeviceTypeByViewport();

  if (reporterSupported.value && !reporterConfigPromptHandled.value) {
    const reporterConfigResult = await discoverExistingReporterConfig();
    if (reporterConfigResult.success && reporterConfigResult.data?.found) {
      existingReporterConfig.value = reporterConfigResult.data;
    }
  }

  if (!reporterSupported.value) {
    return;
  }

  await refreshReporterSnapshot();
  syncReporterPolling();
  await refreshDiscordPresenceSnapshot();
  syncDiscordPresencePolling();

  if (config.value.reporterEnabled && !reporterSnapshot.value.running && readiness.value) {
    void handleStartReporter();
  }
  if (config.value.discordEnabled && !discordPresenceSnapshot.value.running && discordReadiness.value) {
    void handleStartDiscordPresence();
  }
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", onViewportResize);
  document.removeEventListener("visibilitychange", onVisibilityChange);
  stopReporterPolling();
  stopDiscordPresencePolling();
  unlistenSingleInstance?.();
  unlistenSingleInstance = undefined;
});
</script>

<template>
  <div class="app-root" :class="{ 'has-pending-save': hasPendingSettingsChanges }">
    <Toast v-if="!isNativeNotice" position="top-right" />
    <Dialog
      :visible="shouldShowOnboarding"
      modal
      dismissable-mask
      :draggable="false"
      :closable="false"
      class="onboarding-dialog"
    >
      <template #container>
        <div class="onboarding-panel">
          <template v-if="!onboardingSetupMode">
            <p class="eyebrow">{{ t("app.onboarding.eyebrow") }}</p>
            <h3>{{ t("app.onboarding.welcomeTitle") }}</h3>
            <p class="onboarding-copy">
              {{ t("app.onboarding.welcomeCopy") }}
            </p>
            <div
              v-if="reporterSupported && existingReporterConfig?.found && !reporterConfigPromptHandled"
              class="onboarding-step onboarding-highlight"
            >
              <strong>{{ t("app.onboarding.foundConfigTitle") }}</strong>
              <span>{{ t("app.onboarding.foundConfigDetail") }}</span>
              <small v-if="existingReporterConfig.path">{{ existingReporterConfig.path }}</small>
              <div class="actions-row">
                <Button
                  :label="t('app.onboarding.useExistingConfig')"
                  icon="pi pi-download"
                  :loading="importingReporterConfig"
                  @click="useExistingReporterConfig"
                />
                <Button
                  :label="t('app.onboarding.skipImport')"
                  severity="secondary"
                  text
                  @click="skipExistingReporterConfig"
                />
              </div>
            </div>
            <div class="onboarding-steps">
              <div class="onboarding-step">
                <strong>{{ t("app.onboarding.step1Title") }}</strong>
                <span>{{ t("app.onboarding.step1Detail") }}</span>
              </div>
              <div class="onboarding-step">
                <strong>{{ t("app.onboarding.step2Title") }}</strong>
                <span>{{ t("app.onboarding.step2Detail") }}</span>
              </div>
              <div class="onboarding-step">
                <strong>{{ t("app.onboarding.step3Title") }}</strong>
                <span>{{ t("app.onboarding.step3Detail") }}</span>
              </div>
            </div>
            <div class="actions-row">
              <Button :label="t('app.onboarding.goToSettings')" icon="pi pi-arrow-right" @click="startSetup" />
              <Button :label="t('app.onboarding.later')" severity="secondary" text @click="closeOnboarding" />
            </div>
          </template>

          <template v-else>
            <p class="eyebrow">{{ t("app.onboarding.eyebrow") }}</p>
            <h3>{{ t("app.onboarding.setupTitle") }}</h3>
            <p class="onboarding-copy">
              {{ t("app.onboarding.setupCopy") }}
            </p>
            <ConnectionPanel
              :model-value="onboardingDraftConfig"
              :capabilities="capabilities"
              :verified-generated-hash-key="verifiedGeneratedHashKey"
              variant="onboarding"
              @update:model-value="onboardingDraftConfig = $event"
              @imported="notifyImport"
            />
            <div class="actions-row">
              <Button
                :label="t('app.onboarding.complete')"
                icon="pi pi-check"
                :disabled="!(
                  onboardingDraftConfig.baseUrl.trim() &&
                  onboardingDraftConfig.apiToken.trim() &&
                  onboardingDraftConfig.generatedHashKey.trim()
                )"
                @click="completeOnboardingSetup"
              />
              <Button
                :label="t('app.onboarding.back')"
                severity="secondary"
                text
                @click="onboardingSetupMode = false"
              />
            </div>
          </template>
        </div>
      </template>
    </Dialog>

    <Dialog
      v-model:visible="pendingApprovalDialogVisible"
      modal
      dismissable-mask
      :draggable="false"
      :header="t('app.pendingApproval.header')"
      style="width: min(560px, calc(100vw - 24px))"
    >
      <div class="onboarding-steps">
        <div class="onboarding-step">
          <strong>{{ reporterSnapshot.lastPendingApprovalMessage || t("app.pendingApproval.defaultMessage") }}</strong>
          <span>{{ t("app.pendingApproval.detail") }}</span>
        </div>
        <div
          v-if="reporterSnapshot.lastPendingApprovalUrl"
          class="onboarding-step onboarding-highlight"
        >
          <strong>{{ t("app.pendingApproval.approvalUrl") }}</strong>
          <span>{{ reporterSnapshot.lastPendingApprovalUrl }}</span>
        </div>
      </div>
      <div class="actions-row">
        <Button :label="t('app.pendingApproval.confirm')" icon="pi pi-check" @click="closePendingApprovalDialog" />
      </div>
    </Dialog>

    <transition name="pending-save-float">
      <section
        v-if="hasPendingSettingsChanges"
        class="pending-save-float"
        aria-live="polite"
      >
        <div class="pending-save-float-copy">
          <p class="eyebrow">{{ t("app.pendingSave.eyebrow") }}</p>
          <strong>{{ t("app.pendingSave.title") }}</strong>
          <span>{{ t("app.pendingSave.detail") }}</span>
        </div>
        <div class="pending-save-float-actions">
          <Button
            :label="t('app.pendingSave.apply')"
            icon="pi pi-check"
            size="small"
            @click="applySettingsChanges"
          />
          <Button
            :label="t('app.pendingSave.revert')"
            icon="pi pi-undo"
            severity="secondary"
            outlined
            size="small"
            @click="revertPendingSettings"
          />
        </div>
      </section>
    </transition>

    <div class="app-shell" :class="{ 'phone-nav': isPhone }">
      <aside v-if="!isPhone" class="app-sidebar">
        <div class="brand-block">
          <p class="eyebrow">Waken-Wa</p>
          <h1>{{ t("app.brand.client") }}</h1>
        </div>

        <nav class="nav-stack">
          <button
            v-for="section in visibleSections"
            :key="section.key"
            class="nav-item"
            :class="{ active: section.key === activeSection }"
            type="button"
            @click="activeSection = section.key"
          >
            <i :class="section.icon" />
            <div>
              <strong>{{ section.title }}</strong>
              <span>{{ section.kicker }}</span>
            </div>
          </button>
        </nav>

        <div class="sidebar-footer">
          <Tag :value="readiness ? t('app.sidebar.readinessReady') : t('app.sidebar.readinessPending')" :severity="readiness ? 'success' : 'warn'" rounded />
          <Tag
            v-if="reporterSupported"
            :value="reporterSnapshot.running ? t('app.sidebar.reporterRunning') : t('app.sidebar.reporterStopped')"
            :severity="reporterSnapshot.running ? 'success' : 'secondary'"
            rounded
          />
          <Tag
            v-if="discordSupported"
            :value="discordPresenceSnapshot.running ? (discordPresenceSnapshot.connected ? t('app.sidebar.discordRunning') : t('app.sidebar.discordWaiting')) : t('app.sidebar.discordStopped')"
            :severity="discordPresenceSnapshot.running ? (discordPresenceSnapshot.connected ? 'success' : 'warn') : 'secondary'"
            rounded
          />
          <small v-if="traySupported">{{ t("app.sidebar.traySupported") }}</small>
          <small v-else>{{ t("app.sidebar.trayUnsupported") }}</small>
        </div>
      </aside>

      <main class="app-main">
        <OverviewWorkspace
          v-if="activeSection === 'overview'"
          :config="config"
          :readiness="readiness"
          :capabilities="capabilities"
          :mobile-connectivity="mobileConnectivity"
          :reporter-snapshot="reporterSnapshot"
          :discord-presence-snapshot="discordPresenceSnapshot"
          :reporter-busy="reporterBusy"
          @start-reporter="handleStartReporter"
          @stop-reporter="handleStopReporter"
          @retry-mobile-connectivity="runMobileConnectivityProbe(true)"
        />

        <SettingsWorkspace
          v-else-if="activeSection === 'settings'"
          :model-value="config"
          :capabilities="capabilities"
          :reporter-snapshot="reporterSnapshot"
          :discord-presence-snapshot="discordPresenceSnapshot"
          :reporter-busy="reporterBusy"
          :discord-busy="discordBusy"
          :verified-generated-hash-key="verifiedGeneratedHashKey"
          :locale="currentLocale"
          :locale-restart-required="localeRestartRequired"
          :restarting="restartingApp || localeSaving"
          @update:model-value="config = normalizeConfigByCapabilities($event)"
          @update:locale="applyLocale($event, true)"
          @restart-app="handleRestartApp"
          @imported="notifyImport"
          @start-reporter="handleStartReporter"
          @stop-reporter="handleStopReporter"
          @start-discord-presence="handleStartDiscordPresence"
          @stop-discord-presence="handleStopDiscordPresence"
        />

        <ActivityWorkspace
          v-else-if="activeSection === 'activity'"
          :config="config"
          :capabilities="capabilities"
          :recent-presets="recentPresets"
          @preset-saved="handlePresetSaved"
          @pending-approval="handlePendingApproval"
          @key-verified="rememberVerifiedGeneratedHashKey"
        />

        <RealtimeWorkspace
          v-else-if="activeSection === 'realtime'"
          :snapshot="reporterSnapshot"
        />

        <InspirationWorkspace
          v-else-if="activeSection === 'inspiration'"
          :config="config"
          :capabilities="capabilities"
          @pending-approval="handlePendingApproval"
          @key-verified="rememberVerifiedGeneratedHashKey"
        />
      </main>
    </div>

    <nav v-if="isPhone" class="mobile-tabbar">
      <button
        v-for="section in visibleSections"
        :key="section.key"
        class="mobile-tab-item"
        :class="{ active: section.key === activeSection }"
        type="button"
        @click="activeSection = section.key"
      >
        <i :class="section.icon" />
        <span>{{ section.title }}</span>
      </button>
    </nav>
  </div>
</template>

import { computed, onBeforeUnmount, ref, watch, type ComputedRef, type Ref } from "vue";
import { addPluginListener, invoke, type PluginListener } from "@tauri-apps/api/core";
import {
  active,
  createChannel,
  Importance,
  Visibility,
  type ActiveNotification,
  type Options,
} from "@tauri-apps/plugin-notification";

import { readDevicePlatform } from "@/lib/deviceInfo";
import type { ClientCapabilities, ClientConfig, RealtimeReporterSnapshot } from "@/types";

interface UseAndroidReporterNotificationOptions {
  t: (key: string, params?: Record<string, unknown>) => string;
  capabilities: Ref<ClientCapabilities>;
  config: Ref<ClientConfig>;
  hydrated: Ref<boolean>;
  readiness: ComputedRef<boolean>;
  reporterBusy: Ref<boolean>;
  reporterSnapshot: Ref<RealtimeReporterSnapshot>;
  reporterSupported: ComputedRef<boolean>;
  handleStartReporter: () => Promise<void>;
  handleStopReporter: () => Promise<void>;
}

const NOTIFICATION_ID = 408201;
const CHANNEL_ID = "waken-wa-reporter-status-v2";
const SMALL_ICON = "ic_launcher";
const LARGE_ICON = "ic_launcher_foreground";
const PLUGIN_NAME = "waken-android-activity";
const REPORTER_ACTION_EVENT = "reporter-action";

interface ReporterActionPayload {
  action?: "pause" | "resume" | string;
}

export function useAndroidReporterNotification(options: UseAndroidReporterNotificationOptions) {
  let notificationReady = false;
  let notificationSetupInFlight: Promise<boolean> | undefined;
  let notificationSyncInFlight = false;
  let notificationSyncQueued = false;
  const isAndroid = ref(false);

  // Check device platform on initialization
  readDevicePlatform().then((platform) => {
    isAndroid.value = platform === "android";
  });

  const shouldUseNotification = computed(
    () =>
      options.hydrated.value
      && options.reporterSupported.value
      && options.config.value.androidReporterNotificationEnabled
      && options.capabilities.value.persistentNotification
      && isAndroid.value,
  );

  const notificationSignature = computed(() =>
    [
      String(shouldUseNotification.value),
      String(options.readiness.value),
      String(options.reporterBusy.value),
      String(options.reporterSnapshot.value.running),
      String(options.config.value.androidReporterNotificationEnabled),
      String(options.capabilities.value.persistentNotification),
      options.config.value.device.trim(),
      options.reporterSnapshot.value.lastHeartbeatAt ?? "",
      options.reporterSnapshot.value.lastError ?? "",
    ].join("|"),
  );

  async function ensureNotificationReady() {
    if (notificationReady) {
      return true;
    }
    if (notificationSetupInFlight) {
      return notificationSetupInFlight;
    }

    notificationSetupInFlight = setupNotification();
    const ready = await notificationSetupInFlight;
    notificationSetupInFlight = undefined;
    notificationReady = ready;
    return ready;
  }

  async function setupNotification() {
    try {
      const permissionGranted = await readNativeNotificationPermissionGranted();
      logAndroidNotification("permission check result", {
        ...buildDebugState(),
        permissionGranted,
      });
      if (!permissionGranted) {
        return false;
      }

      await createChannel({
        id: CHANNEL_ID,
        name: options.t("app.androidNotification.channelName"),
        description: options.t("app.androidNotification.channelDescription"),
        importance: Importance.Default,
        visibility: Visibility.Public,
      });
      logAndroidNotification("notification channel ready", { channelId: CHANNEL_ID });
      return true;
    } catch (error) {
      logAndroidNotification("notification setup failed", {
        ...buildDebugState(),
        error: formatError(error),
      });
      return false;
    }
  }

  async function readNativeNotificationPermissionGranted() {
    try {
      const granted = await invoke<boolean | null>("plugin:notification|is_permission_granted");
      return granted === true;
    } catch (error) {
      logAndroidNotification("permission check failed", {
        ...buildDebugState(),
        error: formatError(error),
      });
      return false;
    }
  }

  async function syncNotification() {
    if (notificationSyncInFlight) {
      notificationSyncQueued = true;
      return;
    }

    notificationSyncInFlight = true;
    try {
      if (!shouldUseNotification.value) {
        if (notificationReady) {
          await removeReporterNotification();
        }
        return;
      }

      if (!(await ensureNotificationReady())) {
        logAndroidNotification("notification setup not ready", buildDebugState());
        return;
      }

      await notifyReporterStatus();
    } finally {
      notificationSyncInFlight = false;
      if (notificationSyncQueued) {
        notificationSyncQueued = false;
        void syncNotification();
      }
    }
  }

  function buildReporterNotification(): Options {
    const running = options.reporterSnapshot.value.running;
    const ready = options.readiness.value;
    const device = options.config.value.device.trim() || options.t("overview.summary.unnamedDevice");
    const stateKey = running
      ? "running"
      : ready
        ? "paused"
        : "setupRequired";
    const bodyKey = running
      ? "runningDetail"
      : ready
        ? "pausedDetail"
        : "setupRequiredDetail";
    const summaryLine = buildReporterSummaryLine(stateKey, device);
    const detailLines = buildReporterDetailLines(summaryLine);

    return {
      id: NOTIFICATION_ID,
      channelId: CHANNEL_ID,
      title: options.t("app.androidNotification.title"),
      body: summaryLine,
      inboxLines: detailLines,
      summary: options.t(`app.androidNotification.${stateKey}`),
      ongoing: true,
      autoCancel: false,
      silent: true,
      visibility: Visibility.Public,
      icon: SMALL_ICON,
      largeIcon: LARGE_ICON,
      extra: {
        kind: "android-reporter-status",
        state: stateKey,
        body: options.t(`app.androidNotification.${bodyKey}`, { device }),
      },
    };
  }

  function buildReporterSummaryLine(stateKey: string, device: string) {
    const currentActivity = options.reporterSnapshot.value.currentActivity;
    const status = options.t(`app.androidNotification.${stateKey}`);
    const processName = currentActivity?.processName?.trim();
    const processTitle = currentActivity?.processTitle?.trim();
    const activityText = processTitle || processName;

    if (activityText) {
      return `${status} - ${activityText}`;
    }
    return `${status} - ${device}`;
  }

  function buildReporterDetailLines(summaryLine: string) {
    const lines = [summaryLine];
    const lastHeartbeat = formatNotificationTime(options.reporterSnapshot.value.lastHeartbeatAt);

    if (lastHeartbeat) {
      lines.push(options.t("app.androidNotification.lastHeartbeat", { time: lastHeartbeat }));
    }
    if (options.reporterSnapshot.value.lastError) {
      lines.push(options.t("app.androidNotification.lastError", {
        error: options.reporterSnapshot.value.lastError,
      }));
    }

    return lines.slice(0, 5);
  }

  async function notifyReporterStatus() {
    const notification = buildReporterNotification();
    const running = options.reporterSnapshot.value.running;
    logAndroidNotification("sending reporter notification via foreground service", {
      ...buildDebugState(),
      id: notification.id,
      channelId: notification.channelId,
      running,
    });

    // Use foreground service for persistent notification
    await invoke("plugin:waken-android-activity|start_foreground_service", {
      title: notification.title,
      body: notification.body,
      summary: notification.summary,
      running,
      pauseLabel: options.t("app.androidNotification.pauseAction"),
      resumeLabel: options.t("app.androidNotification.startAction"),
    });

    if (import.meta.env.DEV) {
      try {
        const activeNotifications = await active();
        logAndroidNotification("active notifications after send", {
          active: activeNotifications.map(formatActiveNotification),
          hasReporterNotification: activeNotifications.some((item) => item.id === NOTIFICATION_ID),
        });
      } catch (error) {
        logAndroidNotification("active notification probe failed", { error: formatError(error) });
      }
    }
  }

  async function removeReporterNotification() {
    try {
      // Stop foreground service
      await invoke("plugin:waken-android-activity|stop_foreground_service");
    } catch {
      // Removing an already-cleared Android notification can fail harmlessly.
    }
  }

  function buildDebugState() {
    return {
      shouldUseNotification: shouldUseNotification.value,
      hydrated: options.hydrated.value,
      reporterSupported: options.reporterSupported.value,
      notificationEnabled: options.config.value.androidReporterNotificationEnabled,
      readiness: options.readiness.value,
      reporterRunning: options.reporterSnapshot.value.running,
      reporterBusy: options.reporterBusy.value,
      androidRuntime: isAndroid.value,
    };
  }

  watch(
    notificationSignature,
    () => {
      void syncNotification();
    },
    { immediate: true },
  );

  let reporterActionListener: PluginListener | undefined;
  let reporterActionInFlight = false;

  watch(
    isAndroid,
    async (value) => {
      if (!value || reporterActionListener) {
        return;
      }
      try {
        reporterActionListener = await addPluginListener<ReporterActionPayload>(
          PLUGIN_NAME,
          REPORTER_ACTION_EVENT,
          (payload) => {
            void handleReporterActionEvent(payload);
          },
        );
        logAndroidNotification("reporter-action listener registered");
      } catch (error) {
        logAndroidNotification("reporter-action listener register failed", {
          error: formatError(error),
        });
      }
    },
    { immediate: true },
  );

  async function handleReporterActionEvent(payload: ReporterActionPayload) {
    const action = payload?.action;
    if (action !== "pause" && action !== "resume") {
      return;
    }
    if (reporterActionInFlight || options.reporterBusy.value) {
      logAndroidNotification("reporter-action ignored (busy)", { action });
      return;
    }
    reporterActionInFlight = true;
    try {
      if (action === "pause") {
        await options.handleStopReporter();
      } else {
        await options.handleStartReporter();
      }
      logAndroidNotification("reporter-action handled", { action });
    } catch (error) {
      logAndroidNotification("reporter-action failed", {
        action,
        error: formatError(error),
      });
    } finally {
      reporterActionInFlight = false;
    }
  }

  onBeforeUnmount(() => {
    void removeReporterNotification();
    if (reporterActionListener) {
      void reporterActionListener.unregister().catch(() => {
        // listener may already be torn down with the webview
      });
      reporterActionListener = undefined;
    }
  });
}

function formatNotificationTime(value?: string | null) {
  if (!value) {
    return "";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  return `${hours}:${minutes}`;
}

function formatActiveNotification(notification: ActiveNotification) {
  return {
    id: notification.id,
    tag: notification.tag,
  };
}

function logAndroidNotification(message: string, payload?: Record<string, unknown>) {
  if (!import.meta.env.DEV) {
    return;
  }
  console.info(`[android-notification] ${message} ${formatLogPayload(payload ?? {})}`);
}

function formatError(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }
  return formatLogPayload(error);
}

function formatLogPayload(payload: unknown) {
  try {
    return JSON.stringify(payload);
  } catch {
    return String(payload);
  }
}

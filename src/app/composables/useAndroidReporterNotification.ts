import { computed, onBeforeUnmount, watch, type ComputedRef, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  active,
  createChannel,
  Importance,
  onAction,
  registerActionTypes,
  removeActive,
  Visibility,
  type ActiveNotification,
  type Options,
} from "@tauri-apps/plugin-notification";
import type { PluginListener } from "@tauri-apps/api/core";

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

interface AndroidNotificationActionPayload extends Partial<Options> {
  action?: string | { id?: string };
  actionId?: string;
  actionIdentifier?: string;
  notification?: Partial<Options> | null;
}

const NOTIFICATION_ID = 408201;
const CHANNEL_ID = "waken-wa-reporter-status-v2";
const SMALL_ICON = "ic_launcher";
const LARGE_ICON = "ic_launcher_foreground";
const ACTION_TYPE_RUNNING = "waken-wa-reporter-running";
const ACTION_TYPE_PAUSED = "waken-wa-reporter-paused";
const ACTION_START = "start-reporter";
const ACTION_PAUSE = "pause-reporter";

export function useAndroidReporterNotification(options: UseAndroidReporterNotificationOptions) {
  let actionListener: PluginListener | undefined;
  let actionLabelsKey = "";
  let notificationReady = false;
  let notificationSetupInFlight: Promise<boolean> | undefined;
  let notificationSyncInFlight = false;
  let notificationSyncQueued = false;

  const shouldUseNotification = computed(
    () =>
      options.hydrated.value
      && options.reporterSupported.value
      && options.config.value.androidReporterNotificationEnabled
      && options.capabilities.value.persistentNotification
      && isAndroidRuntime(),
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
      options.t("app.androidNotification.pauseAction"),
      options.t("app.androidNotification.startAction"),
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

      await registerReporterActions();
      logAndroidNotification("notification actions ready", {
        actionTypeRunning: ACTION_TYPE_RUNNING,
        actionTypePaused: ACTION_TYPE_PAUSED,
      });

      actionListener = await onAction((payload) => {
        void handleNotificationAction(payload as AndroidNotificationActionPayload);
      });
      logAndroidNotification("notification action listener ready");
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

  async function registerReporterActions() {
    const nextLabelsKey = [
      options.t("app.androidNotification.pauseAction"),
      options.t("app.androidNotification.startAction"),
    ].join("|");

    if (nextLabelsKey === actionLabelsKey) {
      return;
    }

    await registerActionTypes([
      {
        id: ACTION_TYPE_RUNNING,
        actions: [
          {
            id: ACTION_PAUSE,
            title: options.t("app.androidNotification.pauseAction"),
          },
        ],
      },
      {
        id: ACTION_TYPE_PAUSED,
        actions: [
          {
            id: ACTION_START,
            title: options.t("app.androidNotification.startAction"),
            foreground: true,
          },
        ],
      },
    ]);
    actionLabelsKey = nextLabelsKey;
  }

  async function handleNotificationAction(payload: AndroidNotificationActionPayload) {
    const actionId = getNotificationActionId(payload);
    const notification = getActionNotification(payload);

    if (!isReporterAction(actionId) && !isReporterNotification(notification)) {
      return;
    }

    if (actionId === ACTION_PAUSE) {
      await options.handleStopReporter();
    } else if (actionId === ACTION_START) {
      if (options.readiness.value) {
        await options.handleStartReporter();
      }
    }

    await syncNotification();
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

      await registerReporterActions();
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
      actionTypeId: running ? ACTION_TYPE_RUNNING : ACTION_TYPE_PAUSED,
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
    logAndroidNotification("sending reporter notification", {
      ...buildDebugState(),
      id: notification.id,
      channelId: notification.channelId,
      actionTypeId: notification.actionTypeId,
    });

    await invoke("plugin:notification|notify", { options: notification });

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

  function isReporterNotification(notification: Partial<Options> | null | undefined) {
    return notification?.id === NOTIFICATION_ID
      || notification?.actionTypeId === ACTION_TYPE_RUNNING
      || notification?.actionTypeId === ACTION_TYPE_PAUSED
      || notification?.extra?.kind === "android-reporter-status";
  }

  function isReporterAction(actionId: string | undefined) {
    return actionId === ACTION_PAUSE || actionId === ACTION_START;
  }

  function getActionNotification(payload: AndroidNotificationActionPayload) {
    return payload.notification ?? payload;
  }

  function getNotificationActionId(payload: AndroidNotificationActionPayload) {
    if (typeof payload.actionId === "string") {
      return payload.actionId;
    }
    if (typeof payload.actionIdentifier === "string") {
      return payload.actionIdentifier;
    }
    if (typeof payload.action === "string") {
      return payload.action;
    }
    if (payload.action && typeof payload.action === "object" && typeof payload.action.id === "string") {
      return payload.action.id;
    }

    const extraAction = payload.extra?.action;
    if (typeof extraAction === "string") {
      return extraAction;
    }
    const extraActionId = payload.extra?.actionId;
    if (typeof extraActionId === "string") {
      return extraActionId;
    }

    return undefined;
  }

  async function removeReporterNotification() {
    try {
      await removeActive([{ id: NOTIFICATION_ID }]);
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
      androidRuntime: isAndroidRuntime(),
    };
  }

  watch(
    notificationSignature,
    () => {
      void syncNotification();
    },
    { immediate: true },
  );

  onBeforeUnmount(() => {
    actionListener?.unregister();
    void removeReporterNotification();
  });
}

function isAndroidRuntime() {
  return typeof navigator !== "undefined" && /Android/i.test(navigator.userAgent);
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

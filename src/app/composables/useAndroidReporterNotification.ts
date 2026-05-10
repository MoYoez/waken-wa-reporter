import { computed, onBeforeUnmount, watch, type ComputedRef, type Ref } from "vue";
import {
  createChannel,
  Importance,
  isPermissionGranted,
  onAction,
  registerActionTypes,
  removeActive,
  sendNotification,
  Visibility,
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
const CHANNEL_ID = "waken-wa-reporter-status";
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
      && options.capabilities.value.qrImport
      && isAndroidRuntime(),
  );

  const notificationSignature = computed(() =>
    [
      String(shouldUseNotification.value),
      String(options.readiness.value),
      String(options.reporterBusy.value),
      String(options.reporterSnapshot.value.running),
      String(options.config.value.androidReporterNotificationEnabled),
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
      if (!(await isPermissionGranted())) {
        return false;
      }

      await createChannel({
        id: CHANNEL_ID,
        name: options.t("app.androidNotification.channelName"),
        description: options.t("app.androidNotification.channelDescription"),
        importance: Importance.Low,
        visibility: Visibility.Public,
      });
      await registerReporterActions();
      actionListener = await onAction((payload) => {
        void handleNotificationAction(payload as AndroidNotificationActionPayload);
      });
      return true;
    } catch {
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
        return;
      }

      await registerReporterActions();
      sendNotification(buildReporterNotification());
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

    return {
      id: NOTIFICATION_ID,
      channelId: CHANNEL_ID,
      title: options.t("app.androidNotification.title"),
      body: options.t(`app.androidNotification.${bodyKey}`, { device }),
      largeBody: options.t(`app.androidNotification.${bodyKey}`, { device }),
      summary: options.t(`app.androidNotification.${stateKey}`),
      actionTypeId: running ? ACTION_TYPE_RUNNING : ACTION_TYPE_PAUSED,
      ongoing: true,
      autoCancel: false,
      silent: true,
      visibility: Visibility.Public,
      iconColor: "#0f8b8d",
      extra: {
        kind: "android-reporter-status",
        state: stateKey,
      },
    };
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

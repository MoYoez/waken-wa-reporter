import type { ComputedRef, Ref } from "vue";

import {
  getDiscordPresenceSnapshot,
  startDiscordPresenceSync,
  stopDiscordPresenceSync,
} from "@/lib/api";
import type { NotifyPayload } from "@/lib/notify";
import type { ClientConfig, DiscordPresenceSnapshot } from "@/types";

interface UseAppShellDiscordRuntimeOptions {
  t: (key: string, params?: Record<string, unknown>) => string;
  notify: (payload: NotifyPayload) => void;
  apiErrorDetail: (
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) => string;
  config: Ref<ClientConfig>;
  discordBusy: Ref<boolean>;
  discordPresenceSnapshot: Ref<DiscordPresenceSnapshot>;
  discordSupported: ComputedRef<boolean>;
}

const DISCORD_SNAPSHOT_POLL_MS = 5000;

export function useAppShellDiscordRuntime(options: UseAppShellDiscordRuntimeOptions) {
  let discordPollingTimer: number | undefined;
  let discordSnapshotRefreshInFlight = false;

  async function refreshDiscordPresenceSnapshot() {
    if (!options.discordSupported.value || discordSnapshotRefreshInFlight) {
      return;
    }

    discordSnapshotRefreshInFlight = true;
    try {
      const result = await getDiscordPresenceSnapshot();
      if (!result.success || !result.data) {
        return;
      }
      Object.assign(options.discordPresenceSnapshot.value, result.data);
    } finally {
      discordSnapshotRefreshInFlight = false;
    }
  }

  function shouldPollDiscordPresenceSnapshot() {
    return options.discordSupported.value
      && options.discordPresenceSnapshot.value.running
      && document.visibilityState === "visible";
  }

  function stopDiscordPresencePolling() {
    if (discordPollingTimer !== undefined) {
      window.clearInterval(discordPollingTimer);
      discordPollingTimer = undefined;
    }
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
    }, DISCORD_SNAPSHOT_POLL_MS);
  }

  async function handleStartDiscordPresence() {
    if (!options.discordSupported.value || options.discordBusy.value) {
      return;
    }

    options.discordBusy.value = true;
    const result = await startDiscordPresenceSync(options.config.value);
    options.discordBusy.value = false;

    if (!result.success || !result.data) {
      options.notify({
        severity: "error",
        summary: options.t("app.notify.discordStartFailed"),
        detail: options.apiErrorDetail(result.error, options.t("app.notify.discordStartFailedDetail")),
        life: 4000,
      });
      return;
    }

    Object.assign(options.discordPresenceSnapshot.value, result.data);
    options.notify({
      severity: "success",
      summary: options.t("app.notify.discordStarted"),
      detail: options.t("app.notify.discordStartedDetail"),
      life: 3000,
    });
  }

  async function handleStopDiscordPresence() {
    if (!options.discordSupported.value || options.discordBusy.value) {
      return;
    }

    options.discordBusy.value = true;
    const result = await stopDiscordPresenceSync();
    options.discordBusy.value = false;

    if (!result.success || !result.data) {
      options.notify({
        severity: "error",
        summary: options.t("app.notify.discordStopFailed"),
        detail: options.apiErrorDetail(result.error, options.t("app.notify.discordStopFailedDetail")),
        life: 4000,
      });
      return;
    }

    Object.assign(options.discordPresenceSnapshot.value, result.data);
    options.notify({
      severity: "success",
      summary: options.t("app.notify.discordStopped"),
      detail: options.t("app.notify.discordStoppedDetail"),
      life: 3000,
    });
  }

  return {
    handleStartDiscordPresence,
    handleStopDiscordPresence,
    refreshDiscordPresenceSnapshot,
    shouldPollDiscordPresenceSnapshot,
    stopDiscordPresencePolling,
    syncDiscordPresencePolling,
  };
}

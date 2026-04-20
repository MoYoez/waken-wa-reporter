import type { ComputedRef, Ref } from "vue";

import {
  getRealtimeReporterSnapshot,
  startRealtimeReporter,
  stopRealtimeReporter,
} from "@/lib/api";
import type { NotifyPayload } from "@/lib/notify";
import type { ClientConfig, RealtimeReporterSnapshot } from "@/types";

interface UseAppShellReporterRuntimeOptions {
  t: (key: string, params?: Record<string, unknown>) => string;
  notify: (payload: NotifyPayload) => void;
  apiErrorDetail: (
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) => string;
  config: Ref<ClientConfig>;
  activeSection: Ref<string>;
  reporterBusy: Ref<boolean>;
  reporterSnapshot: Ref<RealtimeReporterSnapshot>;
  reporterSupported: ComputedRef<boolean>;
}

const REPORTER_SNAPSHOT_POLL_MS = 5000;

export function useAppShellReporterRuntime(options: UseAppShellReporterRuntimeOptions) {
  let reporterPollingTimer: number | undefined;
  let reporterSnapshotRefreshInFlight = false;

  async function refreshReporterSnapshot() {
    if (!options.reporterSupported.value || options.activeSection.value === "inspiration") {
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
      Object.assign(options.reporterSnapshot.value, result.data);
    } finally {
      reporterSnapshotRefreshInFlight = false;
    }
  }

  function shouldPollReporterSnapshot() {
    return options.reporterSupported.value
      && options.reporterSnapshot.value.running
      && document.visibilityState === "visible";
  }

  function stopReporterPolling() {
    if (reporterPollingTimer !== undefined) {
      window.clearInterval(reporterPollingTimer);
      reporterPollingTimer = undefined;
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

  async function handleStartReporter() {
    if (!options.reporterSupported.value || options.reporterBusy.value) {
      return;
    }

    options.reporterBusy.value = true;
    const reporterConfig = {
      ...options.config.value,
      pushMode: "realtime" as const,
    };
    const result = await startRealtimeReporter(reporterConfig);
    options.reporterBusy.value = false;

    if (!result.success || !result.data) {
      options.notify({
        severity: "error",
        summary: options.t("app.notify.reporterStartFailed"),
        detail: options.apiErrorDetail(result.error, options.t("app.notify.reporterStartFailedDetail")),
        life: 4000,
      });
      return;
    }

    Object.assign(options.reporterSnapshot.value, result.data);
    options.notify({
      severity: "success",
      summary: options.t("app.notify.reporterStarted"),
      detail: options.t("app.notify.reporterStartedDetail"),
      life: 3000,
    });
  }

  async function handleStopReporter() {
    if (!options.reporterSupported.value || options.reporterBusy.value) {
      return;
    }

    options.reporterBusy.value = true;
    const result = await stopRealtimeReporter();
    options.reporterBusy.value = false;

    if (!result.success || !result.data) {
      options.notify({
        severity: "error",
        summary: options.t("app.notify.reporterStopFailed"),
        detail: options.apiErrorDetail(result.error, options.t("app.notify.reporterStopFailedDetail")),
        life: 4000,
      });
      return;
    }

    Object.assign(options.reporterSnapshot.value, result.data);
    options.notify({
      severity: "success",
      summary: options.t("app.notify.reporterStopped"),
      detail: options.t("app.notify.reporterStoppedDetail"),
      life: 3000,
    });
  }

  return {
    handleStartReporter,
    handleStopReporter,
    refreshReporterSnapshot,
    shouldPollReporterSnapshot,
    stopReporterPolling,
    syncReporterPolling,
  };
}

import type { ComputedRef, Ref } from "vue";

import type { NotifyPayload } from "@/lib/notify";
import { useAppShellDiscordRuntime } from "@/app/composables/useAppShellDiscordRuntime";
import { useAppShellMobileConnectivity } from "@/app/composables/useAppShellMobileConnectivity";
import { useAppShellReporterRuntime } from "@/app/composables/useAppShellReporterRuntime";
import type {
  ClientConfig,
  DiscordPresenceSnapshot,
  MobileConnectivityState,
  PendingApprovalInfo,
  RealtimeReporterSnapshot,
} from "@/types";

interface UseAppShellRuntimeOptions {
  t: (key: string, params?: Record<string, unknown>) => string;
  notify: (payload: NotifyPayload) => void;
  apiErrorDetail: (
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) => string;
  config: Ref<ClientConfig>;
  activeSection: Ref<string>;
  reporterBusy: Ref<boolean>;
  discordBusy: Ref<boolean>;
  mobileConnectivity: Ref<MobileConnectivityState>;
  reporterSnapshot: Ref<RealtimeReporterSnapshot>;
  discordPresenceSnapshot: Ref<DiscordPresenceSnapshot>;
  pendingApprovalDialogVisible: Ref<boolean>;
  lastMobileConnectivitySignature: Ref<string>;
  reporterSupported: ComputedRef<boolean>;
  discordSupported: ComputedRef<boolean>;
  readiness: ComputedRef<boolean>;
  rememberVerifiedGeneratedHashKey: (value: string) => void;
}

export function useAppShellRuntime(options: UseAppShellRuntimeOptions) {
  function handlePendingApproval(info: PendingApprovalInfo) {
    options.reporterSnapshot.value.lastPendingApprovalMessage = info.message;
    options.reporterSnapshot.value.lastPendingApprovalUrl = info.approvalUrl ?? null;
    options.pendingApprovalDialogVisible.value = true;
  }

  function closePendingApprovalDialog() {
    options.pendingApprovalDialogVisible.value = false;
  }

  const mobileConnectivityRuntime = useAppShellMobileConnectivity({
    t: options.t,
    apiErrorDetail: options.apiErrorDetail,
    config: options.config,
    mobileConnectivity: options.mobileConnectivity,
    lastMobileConnectivitySignature: options.lastMobileConnectivitySignature,
    reporterSupported: options.reporterSupported,
    readiness: options.readiness,
    rememberVerifiedGeneratedHashKey: options.rememberVerifiedGeneratedHashKey,
    onPendingApproval: handlePendingApproval,
  });

  const reporterRuntime = useAppShellReporterRuntime({
    t: options.t,
    notify: options.notify,
    apiErrorDetail: options.apiErrorDetail,
    config: options.config,
    activeSection: options.activeSection,
    reporterBusy: options.reporterBusy,
    reporterSnapshot: options.reporterSnapshot,
    reporterSupported: options.reporterSupported,
  });

  const discordRuntime = useAppShellDiscordRuntime({
    t: options.t,
    notify: options.notify,
    apiErrorDetail: options.apiErrorDetail,
    config: options.config,
    discordBusy: options.discordBusy,
    discordPresenceSnapshot: options.discordPresenceSnapshot,
    discordSupported: options.discordSupported,
  });

  function stopAllPolling() {
    reporterRuntime.stopReporterPolling();
    discordRuntime.stopDiscordPresencePolling();
  }

  return {
    closePendingApprovalDialog,
    handlePendingApproval,
    stopAllPolling,
    ...mobileConnectivityRuntime,
    ...reporterRuntime,
    ...discordRuntime,
  };
}

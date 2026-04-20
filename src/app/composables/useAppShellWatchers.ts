import { watch, type ComputedRef, type Ref } from "vue";

import type { AppSection, SectionNavItem } from "@/app/types";
import { normalizeLocale, type SupportedLocale } from "@/i18n";
import type {
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  RealtimeReporterSnapshot,
} from "@/types";

interface UseAppShellWatchersOptions {
  locale: Ref<string>;
  currentLocale: Ref<SupportedLocale>;
  hydrated: Ref<boolean>;
  capabilities: Ref<ClientCapabilities>;
  config: Ref<ClientConfig>;
  activeSection: Ref<AppSection>;
  visibleSections: ComputedRef<SectionNavItem[]>;
  reporterSupported: ComputedRef<boolean>;
  discordSupported: ComputedRef<boolean>;
  readiness: ComputedRef<boolean>;
  reporterSnapshot: Ref<RealtimeReporterSnapshot>;
  discordPresenceSnapshot: Ref<DiscordPresenceSnapshot>;
  pendingApprovalDialogVisible: Ref<boolean>;
  lastPendingApprovalSeen: Ref<string>;
  ensureVisibleSection: () => void;
  syncDeviceTypeByViewport: () => void;
  syncReporterPolling: () => void;
  syncDiscordPresencePolling: () => void;
  refreshReporterSnapshot: () => Promise<void>;
  refreshDiscordPresenceSnapshot: () => Promise<void>;
  runMobileConnectivityProbe: (force?: boolean) => Promise<void>;
}

export function useAppShellWatchers(options: UseAppShellWatchersOptions) {
  watch(
    () => [
      options.reporterSnapshot.value.lastPendingApprovalMessage,
      options.reporterSnapshot.value.lastPendingApprovalUrl,
    ],
    ([message, url]) => {
      const nextKey = `${message ?? ""}|${url ?? ""}`;
      if (!message || nextKey === "|" || nextKey === options.lastPendingApprovalSeen.value) {
        return;
      }
      options.lastPendingApprovalSeen.value = nextKey;
      options.pendingApprovalDialogVisible.value = true;
    },
    { immediate: true },
  );

  watch(
    () => options.activeSection.value,
    (section) => {
      if (options.reporterSupported.value && section !== "inspiration") {
        void options.refreshReporterSnapshot();
      }
      if (options.discordSupported.value) {
        void options.refreshDiscordPresenceSnapshot();
      }
      options.syncReporterPolling();
      options.syncDiscordPresencePolling();
    },
  );

  watch(
    () => options.visibleSections.value.map((item) => item.key).join(","),
    () => {
      options.ensureVisibleSection();
    },
  );

  watch(
    () => options.capabilities.value,
    () => {
      options.syncDeviceTypeByViewport();
      options.syncReporterPolling();
      options.syncDiscordPresencePolling();
    },
    { deep: true },
  );

  watch(
    () => options.reporterSnapshot.value.running,
    () => {
      options.syncReporterPolling();
    },
    { immediate: true },
  );

  watch(
    () => options.discordPresenceSnapshot.value.running,
    () => {
      options.syncDiscordPresencePolling();
    },
    { immediate: true },
  );

  watch(
    () => options.locale.value,
    () => {
      options.currentLocale.value = normalizeLocale(options.locale.value);
      if (options.hydrated.value) {
        void options.runMobileConnectivityProbe(true);
      }
    },
  );

  watch(
    () => [
      options.hydrated.value,
      options.reporterSupported.value,
      options.readiness.value,
      options.config.value.baseUrl.trim(),
      options.config.value.apiToken.trim(),
      options.config.value.generatedHashKey.trim(),
    ].join("|"),
    () => {
      if (!options.hydrated.value) {
        return;
      }
      void options.runMobileConnectivityProbe();
    },
    { immediate: true },
  );
}

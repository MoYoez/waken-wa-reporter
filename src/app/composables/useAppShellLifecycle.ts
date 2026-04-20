import { onBeforeUnmount, onMounted } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";

interface UseAppShellLifecycleOptions {
  syncViewportWidth: (width: number) => void;
  syncReporterPolling: () => void;
  syncDiscordPresencePolling: () => void;
  shouldPollReporterSnapshot: () => boolean;
  shouldPollDiscordPresenceSnapshot: () => boolean;
  refreshReporterSnapshot: () => Promise<void>;
  refreshDiscordPresenceSnapshot: () => Promise<void>;
  bootstrapAppShell: () => Promise<UnlistenFn | undefined>;
  stopAllPolling: () => void;
}

export function useAppShellLifecycle(options: UseAppShellLifecycleOptions) {
  let unlistenSingleInstance: UnlistenFn | undefined;

  function onViewportResize() {
    options.syncViewportWidth(window.innerWidth);
  }

  function onVisibilityChange() {
    options.syncReporterPolling();
    options.syncDiscordPresencePolling();

    if (options.shouldPollReporterSnapshot()) {
      void options.refreshReporterSnapshot();
    }

    if (options.shouldPollDiscordPresenceSnapshot()) {
      void options.refreshDiscordPresenceSnapshot();
    }
  }

  onMounted(async () => {
    options.syncViewportWidth(window.innerWidth);
    window.addEventListener("resize", onViewportResize);
    document.addEventListener("visibilitychange", onVisibilityChange);
    unlistenSingleInstance = await options.bootstrapAppShell();
  });

  onBeforeUnmount(() => {
    window.removeEventListener("resize", onViewportResize);
    document.removeEventListener("visibilitychange", onVisibilityChange);
    options.stopAllPolling();
    unlistenSingleInstance?.();
    unlistenSingleInstance = undefined;
  });
}

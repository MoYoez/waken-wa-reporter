import { watch, type Ref } from "vue";
import { check } from "@tauri-apps/plugin-updater";

import { restartApp } from "@/lib/api";
import type { NotifyPayload } from "@/lib/notify";
import type { ClientCapabilities } from "@/types";

interface UseDesktopUpdaterOptions {
  t: (key: string, params?: Record<string, unknown>) => string;
  notify: (payload: NotifyPayload) => void;
  capabilities: Ref<ClientCapabilities>;
  hydrated: Ref<boolean>;
}

export function useDesktopUpdater(options: UseDesktopUpdaterOptions) {
  let checked = false;
  let checking = false;

  async function checkForUpdate() {
    if (checked || checking || !options.hydrated.value || !options.capabilities.value.updater) {
      return;
    }

    checked = true;
    checking = true;

    try {
      const update = await check();
      if (!update) {
        return;
      }

      options.notify({
        severity: "info",
        summary: options.t("app.notify.updateAvailable", { version: update.version }),
        detail: options.t("app.notify.updateDownloading"),
        life: 5000,
      });

      await update.downloadAndInstall();

      options.notify({
        severity: "success",
        summary: options.t("app.notify.updateInstalled", { version: update.version }),
        detail: options.t("app.notify.updateRestarting"),
        life: 3500,
      });

      await restartApp();
    } catch (error) {
      options.notify({
        severity: "warn",
        summary: options.t("app.notify.updateFailed"),
        detail: error instanceof Error ? error.message : options.t("app.notify.updateFailedDetail"),
        life: 6000,
      });
    } finally {
      checking = false;
    }
  }

  watch(
    () => [options.hydrated.value, options.capabilities.value.updater] as const,
    () => {
      void checkForUpdate();
    },
    { immediate: true },
  );
}

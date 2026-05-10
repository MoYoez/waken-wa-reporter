import type { ComputedRef, Ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";

import type { AppSection } from "@/app/types";
import { parseImportedIntegrationConfig } from "@/lib/api";
import {
  extractImportPayloadFromDeepLink,
  isImportDeepLink,
} from "@/lib/deepLinkImport";
import type { NotifyPayload } from "@/lib/notify";
import type { ClientConfig, DeviceType, ImportedIntegrationConfig } from "@/types";

interface UseAppShellDeepLinksOptions {
  t: (key: string, params?: Record<string, unknown>) => string;
  notify: (payload: NotifyPayload) => void;
  config: Ref<ClientConfig>;
  onboardingDraftConfig: Ref<ClientConfig>;
  onboardingDismissed: Ref<boolean>;
  onboardingSetupMode: Ref<boolean>;
  reporterConfigPromptHandled: Ref<boolean>;
  reporterSupported: ComputedRef<boolean>;
  normalizeConfigByCapabilities: (raw: ClientConfig) => ClientConfig;
  inferMobileDeviceType: () => DeviceType;
  selectSection: (section: AppSection) => void;
}

export function useAppShellDeepLinks(options: UseAppShellDeepLinksOptions) {
  const handledUrls = new Set<string>();

  async function initializeDeepLinks(): Promise<UnlistenFn | undefined> {
    try {
      const unlisten = await onOpenUrl((urls) => {
        void handleDeepLinkUrls(urls);
      });
      const currentUrls = await getCurrent().catch(() => null);
      if (currentUrls?.length) {
        void handleDeepLinkUrls(currentUrls);
      }
      return unlisten;
    } catch {
      return undefined;
    }
  }

  async function handleDeepLinkUrls(urls: string[]) {
    const importUrls = urls
      .map((url) => url.trim())
      .filter((url) => url && !handledUrls.has(url) && isImportDeepLink(url));

    for (const url of importUrls) {
      handledUrls.add(url);
      await handleImportUrl(url);
    }
  }

  async function handleImportUrl(url: string) {
    const payload = extractImportPayloadFromDeepLink(url);
    if (!payload) {
      options.notify({
        severity: "warn",
        summary: options.t("app.notify.deepLinkImportFailed"),
        detail: options.t("app.notify.deepLinkImportNoPayload"),
        life: 4000,
      });
      return;
    }

    try {
      const parsed = await parseImportedIntegrationConfig(payload);
      applyImportedConfig(parsed);
    } catch (error) {
      options.notify({
        severity: "error",
        summary: options.t("app.notify.deepLinkImportFailed"),
        detail: error instanceof Error ? error.message : options.t("app.notify.deepLinkImportFailedDetail"),
        life: 4500,
      });
    }
  }

  function applyImportedConfig(parsed: ImportedIntegrationConfig) {
    const importingOnboarding = !options.onboardingDismissed.value;
    const currentConfig = importingOnboarding
      ? options.onboardingDraftConfig.value
      : options.config.value;
    const nextConfig = options.normalizeConfigByCapabilities({
      ...currentConfig,
      baseUrl: toBaseUrl(parsed.reportEndpoint) ?? currentConfig.baseUrl,
      apiToken: parsed.token ?? currentConfig.apiToken,
      device: parsed.deviceName?.trim() || currentConfig.device,
      deviceType: options.reporterSupported.value ? "desktop" : options.inferMobileDeviceType(),
    });

    if (importingOnboarding) {
      options.onboardingDraftConfig.value = nextConfig;
      options.reporterConfigPromptHandled.value = true;
      options.onboardingSetupMode.value = true;
    } else {
      options.config.value = nextConfig;
      options.selectSection("settings");
    }

    options.notify({
      severity: "success",
      summary: options.t("app.notify.deepLinkImported"),
      detail: parsed.tokenName
        ? options.t("app.notify.deepLinkImportedToken", { tokenName: parsed.tokenName })
        : options.t("app.notify.deepLinkImportedDetail"),
      life: 3500,
    });
  }

  function toBaseUrl(reportEndpoint?: string | null) {
    if (!reportEndpoint) {
      return undefined;
    }
    return reportEndpoint.replace(/\/api\/activity\/?$/i, "").replace(/\/$/, "");
  }

  return {
    initializeDeepLinks,
  };
}

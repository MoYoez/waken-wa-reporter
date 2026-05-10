import { computed, reactive, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "primevue/usetoast";

import { parseImportedIntegrationConfig, scanImportQrCode, validateConfig } from "@/lib/api";
import { createNotifier } from "@/lib/notify";
import type { ClientCapabilities, ClientConfig, DeviceType } from "@/types";

interface ConnectionPanelProps {
  modelValue: ClientConfig;
  capabilities: ClientCapabilities;
  variant?: "default" | "onboarding";
}

interface ConnectionPanelCallbacks {
  onUpdateModelValue: (value: ClientConfig) => void;
  onImported: (message: string) => void;
}

export function useConnectionPanel(
  props: ConnectionPanelProps,
  callbacks: ConnectionPanelCallbacks,
) {
  const { t } = useI18n();
  const toast = useToast();

  const importPayload = reactive({ text: "" });
  const qrScanInProgress = ref(false);
  const isNativeNotice = computed(() => !props.capabilities.realtimeReporter);
  const { notify } = createNotifier(toast, () => isNativeNotice.value);

  const issues = computed(() => validateConfig(props.modelValue, props.capabilities));
  const reporterSupported = computed(() => props.capabilities.realtimeReporter);
  const qrScanSupported = computed(() => props.capabilities.qrImport);
  const isOnboarding = computed(() => props.variant === "onboarding");
  const currentGeneratedHashKey = computed(() => props.modelValue.generatedHashKey.trim());
  const reporterContentOptions = computed(() => [
    {
      key: "reportForegroundApp" as const,
      label: t("connectionPanel.reportContent.foreground.label"),
      description: t("connectionPanel.reportContent.foreground.description"),
      inputId: "report-foreground-app",
    },
    {
      key: "reportWindowTitle" as const,
      label: t("connectionPanel.reportContent.windowTitle.label"),
      description: t("connectionPanel.reportContent.windowTitle.description"),
      inputId: "report-window-title",
    },
    {
      key: "reportMedia" as const,
      label: t("connectionPanel.reportContent.media.label"),
      description: t("connectionPanel.reportContent.media.description"),
      inputId: "report-media",
    },
    {
      key: "reportPlaySource" as const,
      label: t("connectionPanel.reportContent.playSource.label"),
      description: t("connectionPanel.reportContent.playSource.description"),
      inputId: "report-play-source",
    },
    {
      key: "reportMediaArtwork" as const,
      label: t("connectionPanel.reportContent.mediaArtwork.label"),
      description: t("connectionPanel.reportContent.mediaArtwork.description"),
      inputId: "report-media-artwork",
    },
    {
      key: "reportPlaybackAppIcon" as const,
      label: t("connectionPanel.reportContent.playbackAppIcon.label"),
      description: t("connectionPanel.reportContent.playbackAppIcon.description"),
      inputId: "report-playback-app-icon",
    },
    {
      key: "reportMediaGenre" as const,
      label: t("connectionPanel.reportContent.mediaGenre.label"),
      description: t("connectionPanel.reportContent.mediaGenre.description"),
      inputId: "report-media-genre",
    },
  ]);

  function updateField<K extends keyof ClientConfig>(key: K, value: ClientConfig[K]) {
    callbacks.onUpdateModelValue({
      ...props.modelValue,
      [key]: value,
    });
  }

  function inferMobileDeviceType(): DeviceType {
    if (typeof window === "undefined") {
      return "mobile";
    }
    return window.matchMedia("(max-width: 899px)").matches ? "mobile" : "tablet";
  }

  function toBaseUrl(reportEndpoint?: string) {
    if (!reportEndpoint) {
      return undefined;
    }
    return reportEndpoint.replace(/\/api\/activity\/?$/i, "").replace(/\/$/, "");
  }

  async function importConfig() {
    try {
      const parsed = await parseImportedIntegrationConfig(importPayload.text);
      callbacks.onUpdateModelValue({
        ...props.modelValue,
        baseUrl: toBaseUrl(parsed.reportEndpoint) ?? props.modelValue.baseUrl,
        apiToken: parsed.token ?? props.modelValue.apiToken,
        device: parsed.deviceName?.trim() || props.modelValue.device,
        deviceType: reporterSupported.value ? "desktop" : inferMobileDeviceType(),
      });
      callbacks.onImported(
        parsed.tokenName
          ? t("connectionPanel.notify.importedToken", { tokenName: parsed.tokenName })
          : t("connectionPanel.notify.importedConfig"),
      );
      notify({
        severity: "success",
        summary: t("connectionPanel.notify.importSuccess"),
        detail: toBaseUrl(parsed.reportEndpoint) ?? t("connectionPanel.notify.importSuccessDetail"),
        life: 3000,
      });
      importPayload.text = "";
    } catch (error) {
      notify({
        severity: "error",
        summary: t("connectionPanel.notify.importFailed"),
        detail: error instanceof Error ? error.message : t("connectionPanel.notify.importFailedDetail"),
        life: 4000,
      });
    }
  }

  async function scanImportConfigQr() {
    if (qrScanInProgress.value) {
      return;
    }

    qrScanInProgress.value = true;
    try {
      const text = await scanImportQrCode();
      if (!text) {
        return;
      }
      importPayload.text = text;
      await importConfig();
    } catch (error) {
      notify({
        severity: "error",
        summary: t("connectionPanel.notify.qrScanFailed"),
        detail: error instanceof Error ? error.message : t("connectionPanel.notify.qrScanFailedDetail"),
        life: 4000,
      });
    } finally {
      qrScanInProgress.value = false;
    }
  }

  return {
    currentGeneratedHashKey,
    importConfig,
    importPayload,
    isOnboarding,
    issues,
    qrScanInProgress,
    qrScanSupported,
    reporterContentOptions,
    reporterSupported,
    scanImportConfigQr,
    updateField,
  };
}

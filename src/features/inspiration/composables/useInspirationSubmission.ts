import { ref, type ComputedRef, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import {
  createInspirationEntry,
  extractPendingApprovalInfo,
  formatPendingApprovalDetail,
  uploadInspirationAsset,
} from "@/lib/api";
import type { NotifyPayload } from "@/lib/notify";
import {
  appendParagraphTextToLexical,
} from "@/lib/inspirationRichText";
import type { ClientConfig, PendingApprovalInfo } from "@/types";
import {
  readInspirationFileAsDataUrl,
  validateInspirationImageFile,
} from "@/features/inspiration/composables/inspirationWorkspaceShared";

interface InspirationComposeState {
  title: string;
  content: string;
  contentLexical: string;
}

interface InspirationSubmissionOptions {
  config: ClientConfig;
  notify: (payload: NotifyPayload) => void;
  apiErrorDetail: (
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) => string;
  configIssues: ComputedRef<string[]>;
  compose: InspirationComposeState;
  inlineImageDataUrl: Ref<string>;
  mobileRuntime: ComputedRef<boolean>;
  attachCurrentStatus: Ref<boolean>;
  attachStatusIncludeDeviceInfo: Ref<boolean>;
  statusSnapshotInput: Ref<string>;
  selectedActivityKey: Ref<string>;
  selectedActivityOption: ComputedRef<unknown | null>;
  selectedSnapshotPreview: ComputedRef<string>;
  buildManualSnapshot: (input: string, includeDeviceInfo: boolean) => string;
  ensureBatteryPercentLoaded: () => Promise<void>;
  onPendingApproval: (info: PendingApprovalInfo) => void;
  onKeyVerified: (generatedHashKey: string) => void;
  resetDraftState: () => void;
  refreshEntries: () => void;
}

export function useInspirationSubmission(options: InspirationSubmissionOptions) {
  const { t } = useI18n();

  const uploadPending = ref(false);
  const inlineUploadPending = ref(false);
  const submitting = ref(false);

  async function onCoverFileSelected(event: Event) {
    const target = event.target as HTMLInputElement | null;
    const file = target?.files?.[0];
    if (!file) {
      return;
    }

    uploadPending.value = true;
    try {
      validateInspirationImageFile(file, t);
      const dataUrl = await readInspirationFileAsDataUrl(file, t);
      options.inlineImageDataUrl.value = dataUrl;

      options.notify({
        severity: "success",
        summary: t("inspiration.notify.coverReady"),
        detail: t("inspiration.notify.coverReadyDetail"),
        life: 3000,
      });
    } catch (error) {
      options.notify({
        severity: "error",
        summary: t("inspiration.notify.coverReadFailed"),
        detail: error instanceof Error ? error.message : t("inspiration.notify.coverReadFailedDetail"),
        life: 4000,
      });
    } finally {
      uploadPending.value = false;
      if (target) {
        target.value = "";
      }
    }
  }

  async function onInlineImageSelected(event: Event) {
    const target = event.target as HTMLInputElement | null;
    const file = target?.files?.[0];
    if (!file) {
      return;
    }

    inlineUploadPending.value = true;

    try {
      validateInspirationImageFile(file, t);
      const dataUrl = await readInspirationFileAsDataUrl(file, t);

      const result = await uploadInspirationAsset(options.config, dataUrl);
      const pendingApproval = extractPendingApprovalInfo(result);
      if (pendingApproval) {
        options.notify({
          severity: "warn",
          summary: t("inspiration.notify.pendingApproval"),
          detail: formatPendingApprovalDetail(pendingApproval),
          life: 6000,
        });
        options.onPendingApproval(pendingApproval);
        return;
      }

      if (!result.success || !result.data?.url) {
        options.notify({
          severity: "error",
          summary: t("inspiration.notify.inlineImageUploadFailed"),
          detail: options.apiErrorDetail(
            result.error,
            t("inspiration.notify.inlineImageUploadFailedDetail"),
          ),
          life: 4000,
        });
        return;
      }

      options.compose.contentLexical = appendParagraphTextToLexical(
        options.compose.contentLexical,
        `![](${result.data.url})`,
      );
      options.onKeyVerified(options.config.generatedHashKey.trim());

      options.notify({
        severity: "success",
        summary: t("inspiration.notify.inlineImageInserted"),
        detail: t("inspiration.notify.inlineImageInsertedDetail"),
        life: 3000,
      });
    } finally {
      inlineUploadPending.value = false;
      if (target) {
        target.value = "";
      }
    }
  }

  async function submitEntry() {
    if (options.configIssues.value.length > 0) {
      options.notify({
        severity: "warn",
        summary: t("inspiration.notify.settingsRequired"),
        detail: t("inspiration.notify.settingsRequiredDetail"),
        life: 4000,
      });
      return;
    }

    if (!options.compose.content.trim()) {
      options.notify({
        severity: "warn",
        summary: t("inspiration.notify.contentRequired"),
        detail: t("inspiration.notify.contentRequiredDetail"),
        life: 3000,
      });
      return;
    }

    if (options.attachCurrentStatus.value) {
      if (options.mobileRuntime.value && !options.statusSnapshotInput.value.trim()) {
        options.notify({
          severity: "warn",
          summary: t("inspiration.notify.statusInputRequired"),
          detail: t("inspiration.notify.statusInputRequiredDetail"),
          life: 3000,
        });
        return;
      }

      if (!options.mobileRuntime.value && !options.selectedActivityOption.value) {
        options.notify({
          severity: "warn",
          summary: t("inspiration.notify.activityRequired"),
          detail: t("inspiration.notify.activityRequiredDetail"),
          life: 3000,
        });
        return;
      }
    }

    const attachPayloadEnabled = options.attachCurrentStatus.value
      && (options.mobileRuntime.value
        ? options.statusSnapshotInput.value.trim().length > 0
        : Boolean(options.selectedActivityOption.value));

    if (
      attachPayloadEnabled
      && options.mobileRuntime.value
      && options.attachStatusIncludeDeviceInfo.value
    ) {
      await options.ensureBatteryPercentLoaded();
    }

    submitting.value = true;
    const result = await createInspirationEntry(options.config, {
      title: options.compose.title,
      content: options.compose.content.trim(),
      contentLexical: options.compose.contentLexical || undefined,
      imageDataUrl: options.inlineImageDataUrl.value || undefined,
      generatedHashKey: options.config.generatedHashKey.trim(),
      attachCurrentStatus: attachPayloadEnabled || undefined,
      preComputedStatusSnapshot: attachPayloadEnabled
        ? (options.mobileRuntime.value
            ? options.buildManualSnapshot(
                options.statusSnapshotInput.value,
                options.attachStatusIncludeDeviceInfo.value,
              )
            : options.selectedSnapshotPreview.value)
        : undefined,
      attachStatusDeviceHash: attachPayloadEnabled ? options.config.generatedHashKey.trim() : undefined,
      attachStatusActivityKey: attachPayloadEnabled && !options.mobileRuntime.value
        ? options.selectedActivityKey.value
        : undefined,
      attachStatusIncludeDeviceInfo: attachPayloadEnabled
        ? options.attachStatusIncludeDeviceInfo.value
        : undefined,
    });
    submitting.value = false;

    const pendingApproval = extractPendingApprovalInfo(result);
    if (pendingApproval) {
      options.notify({
        severity: "warn",
        summary: t("inspiration.notify.pendingApproval"),
        detail: formatPendingApprovalDetail(pendingApproval),
        life: 6000,
      });
      options.onPendingApproval(pendingApproval);
      return;
    }

    if (!result.success) {
      options.notify({
        severity: "error",
        summary: t("inspiration.notify.submitFailed", {
          status: result.status || t("inspiration.common.network"),
        }),
        detail: options.apiErrorDetail(result.error, t("inspiration.notify.submitFailedDetail")),
        life: 4500,
      });
      return;
    }

    options.onKeyVerified(options.config.generatedHashKey.trim());

    options.notify({
      severity: "success",
      summary: t("inspiration.notify.submitSuccess"),
      detail: t("inspiration.notify.submitSuccessDetail"),
      life: 3000,
    });

    options.resetDraftState();
    options.refreshEntries();
  }

  return {
    inlineUploadPending,
    onCoverFileSelected,
    onInlineImageSelected,
    submitting,
    submitEntry,
    uploadPending,
  };
}

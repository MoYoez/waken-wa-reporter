import { ref, type ComputedRef, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import {
  createInspirationEntry,
  extractPendingApprovalInfo,
  formatPendingApprovalDetail,
} from "@/lib/api";
import type { NotifyPayload } from "@/lib/notify";
import type { ClientConfig, PendingApprovalInfo } from "@/types";
import {
  buildInspirationEntryPayload,
  shouldAttachStatusPayload,
} from "@/features/inspiration/composables/inspirationSubmissionPayload";
import { resolveSubmissionValidationError } from "@/features/inspiration/composables/inspirationSubmissionValidation";
import { useInspirationSubmissionUploads } from "@/features/inspiration/composables/useInspirationSubmissionUploads";

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
  const submitting = ref(false);
  const {
    inlineUploadPending,
    onCoverFileSelected,
    onInlineImageSelected,
    uploadPending,
  } = useInspirationSubmissionUploads({
    apiErrorDetail: options.apiErrorDetail,
    compose: options.compose,
    config: options.config,
    inlineImageDataUrl: options.inlineImageDataUrl,
    notify: options.notify,
    onKeyVerified: options.onKeyVerified,
    onPendingApproval: options.onPendingApproval,
  });

  async function submitEntry() {
    const validationError = resolveSubmissionValidationError({
      attachCurrentStatus: options.attachCurrentStatus.value,
      configIssues: options.configIssues.value,
      content: options.compose.content,
      mobileRuntime: options.mobileRuntime.value,
      selectedActivityOption: options.selectedActivityOption.value,
      statusSnapshotInput: options.statusSnapshotInput.value,
      t,
    });
    if (validationError) {
      options.notify(validationError);
      return;
    }

    const attachPayloadEnabled = shouldAttachStatusPayload({
      attachCurrentStatus: options.attachCurrentStatus.value,
      attachStatusIncludeDeviceInfo: options.attachStatusIncludeDeviceInfo.value,
      buildManualSnapshot: options.buildManualSnapshot,
      compose: options.compose,
      config: options.config,
      inlineImageDataUrl: options.inlineImageDataUrl.value,
      mobileRuntime: options.mobileRuntime.value,
      selectedActivityKey: options.selectedActivityKey.value,
      selectedActivityOption: options.selectedActivityOption.value,
      selectedSnapshotPreview: options.selectedSnapshotPreview.value,
      statusSnapshotInput: options.statusSnapshotInput.value,
    });

    if (
      attachPayloadEnabled
      && options.mobileRuntime.value
      && options.attachStatusIncludeDeviceInfo.value
    ) {
      await options.ensureBatteryPercentLoaded();
    }

    submitting.value = true;
    const result = await createInspirationEntry(
      options.config,
      buildInspirationEntryPayload({
        attachCurrentStatus: options.attachCurrentStatus.value,
        attachStatusIncludeDeviceInfo: options.attachStatusIncludeDeviceInfo.value,
        buildManualSnapshot: options.buildManualSnapshot,
        compose: options.compose,
        config: options.config,
        inlineImageDataUrl: options.inlineImageDataUrl.value,
        mobileRuntime: options.mobileRuntime.value,
        selectedActivityKey: options.selectedActivityKey.value,
        selectedActivityOption: options.selectedActivityOption.value,
        selectedSnapshotPreview: options.selectedSnapshotPreview.value,
        statusSnapshotInput: options.statusSnapshotInput.value,
      }),
    );
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

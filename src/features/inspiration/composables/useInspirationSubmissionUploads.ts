import { ref, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import {
  extractPendingApprovalInfo,
  formatPendingApprovalDetail,
  uploadInspirationAsset,
} from "@/lib/api";
import type { NotifyPayload } from "@/lib/notify";
import { appendParagraphTextToLexical } from "@/lib/inspirationRichText";
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

interface UseInspirationSubmissionUploadsOptions {
  apiErrorDetail: (
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) => string;
  compose: InspirationComposeState;
  config: ClientConfig;
  inlineImageDataUrl: Ref<string>;
  notify: (payload: NotifyPayload) => void;
  onKeyVerified: (generatedHashKey: string) => void;
  onPendingApproval: (info: PendingApprovalInfo) => void;
}

export function useInspirationSubmissionUploads(
  options: UseInspirationSubmissionUploadsOptions,
) {
  const { t } = useI18n();

  const uploadPending = ref(false);
  const inlineUploadPending = ref(false);

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

  return {
    inlineUploadPending,
    onCoverFileSelected,
    onInlineImageSelected,
    uploadPending,
  };
}

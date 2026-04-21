import type { NotifyPayload } from "@/lib/notify";

type TranslateFn = (key: string, params?: Record<string, unknown>) => string;

interface InspirationSubmissionValidationOptions {
  attachCurrentStatus: boolean;
  configIssues: string[];
  content: string;
  mobileRuntime: boolean;
  selectedActivityOption: unknown | null;
  statusSnapshotInput: string;
  t: TranslateFn;
}

export function resolveSubmissionValidationError(
  options: InspirationSubmissionValidationOptions,
): NotifyPayload | null {
  if (options.configIssues.length > 0) {
    return {
      severity: "warn",
      summary: options.t("inspiration.notify.settingsRequired"),
      detail: options.t("inspiration.notify.settingsRequiredDetail"),
      life: 4000,
    };
  }

  if (!options.content.trim()) {
    return {
      severity: "warn",
      summary: options.t("inspiration.notify.contentRequired"),
      detail: options.t("inspiration.notify.contentRequiredDetail"),
      life: 3000,
    };
  }

  if (!options.attachCurrentStatus) {
    return null;
  }

  if (options.mobileRuntime && !options.statusSnapshotInput.trim()) {
    return {
      severity: "warn",
      summary: options.t("inspiration.notify.statusInputRequired"),
      detail: options.t("inspiration.notify.statusInputRequiredDetail"),
      life: 3000,
    };
  }

  if (!options.mobileRuntime && !options.selectedActivityOption) {
    return {
      severity: "warn",
      summary: options.t("inspiration.notify.activityRequired"),
      detail: options.t("inspiration.notify.activityRequiredDetail"),
      life: 3000,
    };
  }

  return null;
}

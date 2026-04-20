import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "primevue/usetoast";

import { validateConfig } from "@/lib/api";
import { resolveApiErrorMessage } from "@/lib/localizedText";
import { createNotifier } from "@/lib/notify";
import { useInspirationDraftState } from "@/features/inspiration/composables/useInspirationDraftState";
import { useInspirationEntries } from "@/features/inspiration/composables/useInspirationEntries";
import { useInspirationStatus } from "@/features/inspiration/composables/useInspirationStatus";
import { useInspirationSubmission } from "@/features/inspiration/composables/useInspirationSubmission";
import type {
  ClientCapabilities,
  ClientConfig,
  PendingApprovalInfo,
} from "@/types";

interface InspirationWorkspaceProps {
  config: ClientConfig;
  capabilities: ClientCapabilities;
}

interface InspirationWorkspaceCallbacks {
  onPendingApproval: (info: PendingApprovalInfo) => void;
  onKeyVerified: (generatedHashKey: string) => void;
}

export function useInspirationWorkspace(
  props: InspirationWorkspaceProps,
  callbacks: InspirationWorkspaceCallbacks,
) {
  const { t } = useI18n();
  const toast = useToast();
  const isNativeNotice = computed(() => !props.capabilities.realtimeReporter);
  const { notify } = createNotifier(toast, () => isNativeNotice.value);

  const configIssues = computed(() => validateConfig(props.config, props.capabilities));

  function translateText(key: string, params?: Record<string, unknown>) {
    return params ? t(key, params) : t(key);
  }

  function apiErrorDetail(
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) {
    return resolveApiErrorMessage(error, translateText, fallback);
  }

  const draftState = useInspirationDraftState(props.config);
  const statusState = useInspirationStatus({
    config: props.config,
    capabilities: props.capabilities,
    draftStore: draftState.draftStore,
    apiErrorDetail,
  });
  const entriesState = useInspirationEntries({
    config: props.config,
    apiErrorDetail,
  });
  const submissionState = useInspirationSubmission({
    config: props.config,
    notify,
    apiErrorDetail,
    configIssues,
    compose: draftState.compose,
    inlineImageDataUrl: draftState.inlineImageDataUrl,
    mobileRuntime: statusState.mobileRuntime,
    attachCurrentStatus: statusState.attachCurrentStatus,
    attachStatusIncludeDeviceInfo: statusState.attachStatusIncludeDeviceInfo,
    statusSnapshotInput: statusState.statusSnapshotInput,
    selectedActivityKey: statusState.selectedActivityKey,
    selectedActivityOption: statusState.selectedActivityOption,
    selectedSnapshotPreview: statusState.selectedSnapshotPreview,
    buildManualSnapshot: statusState.buildManualSnapshot,
    ensureBatteryPercentLoaded: statusState.ensureBatteryPercentLoaded,
    onPendingApproval: callbacks.onPendingApproval,
    onKeyVerified: callbacks.onKeyVerified,
    resetDraftState: () => {
      draftState.resetDraftStore();
      draftState.applyDraftStateFromStore();
      statusState.applyDraftStateFromStore();
    },
    refreshEntries: entriesState.refreshEntries,
  });

  return {
    activityLoadError: statusState.activityLoadError,
    activityLoading: statusState.activityLoading,
    activityOptions: statusState.activityOptions,
    attachCurrentStatus: statusState.attachCurrentStatus,
    attachStatusIncludeDeviceInfo: statusState.attachStatusIncludeDeviceInfo,
    compose: draftState.compose,
    composePreviewHtml: draftState.composePreviewHtml,
    composeTab: draftState.composeTab,
    configIssues,
    entryCards: entriesState.entryCards,
    entryCountLabel: entriesState.entryCountLabel,
    hasMoreEntries: entriesState.hasMoreEntries,
    inlineImageDataUrl: draftState.inlineImageDataUrl,
    inlineUploadPending: submissionState.inlineUploadPending,
    loadActivityOptions: statusState.loadActivityOptions,
    loadError: entriesState.loadError,
    loading: entriesState.loading,
    loadingMore: entriesState.loadingMore,
    loadMoreEntries: entriesState.loadMoreEntries,
    mobileRuntime: statusState.mobileRuntime,
    onCoverFileSelected: submissionState.onCoverFileSelected,
    onInlineImageSelected: submissionState.onInlineImageSelected,
    openEntry: entriesState.openEntry,
    refreshEntries: entriesState.refreshEntries,
    selectedActivityKey: statusState.selectedActivityKey,
    selectedEntry: entriesState.selectedEntry,
    selectedEntryCreatedAtLabel: entriesState.selectedEntryCreatedAtLabel,
    selectedEntryHtml: entriesState.selectedEntryHtml,
    selectedEntryImageUrl: entriesState.selectedEntryImageUrl,
    selectedEntryVisible: entriesState.selectedEntryVisible,
    selectedSnapshotPreview: statusState.selectedSnapshotPreview,
    statusSnapshotDeviceName: statusState.statusSnapshotDeviceName,
    statusSnapshotInput: statusState.statusSnapshotInput,
    submitting: submissionState.submitting,
    submitEntry: submissionState.submitEntry,
    uploadPending: submissionState.uploadPending,
  };
}

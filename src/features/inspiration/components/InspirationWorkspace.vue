<script setup lang="ts">
import InspirationComposeCard from "@/features/inspiration/components/InspirationComposeCard.vue";
import InspirationEntryDialog from "@/features/inspiration/components/InspirationEntryDialog.vue";
import InspirationEntryListCard from "@/features/inspiration/components/InspirationEntryListCard.vue";
import { useInspirationWorkspace } from "@/features/inspiration/composables/useInspirationWorkspace";
import type { ClientCapabilities, ClientConfig, PendingApprovalInfo } from "@/types";

const props = defineProps<{
  config: ClientConfig;
  capabilities: ClientCapabilities;
}>();

const emit = defineEmits<{
  pendingApproval: [info: PendingApprovalInfo];
  keyVerified: [generatedHashKey: string];
}>();

const {
  activityLoadError,
  activityLoading,
  activityOptions,
  attachCurrentStatus,
  attachStatusIncludeDeviceInfo,
  compose,
  composePreviewHtml,
  composeTab,
  configIssues,
  entryCards,
  entryCountLabel,
  hasMoreEntries,
  inlineImageDataUrl,
  inlineUploadPending,
  loadActivityOptions,
  loadError,
  loading,
  loadingMore,
  loadMoreEntries,
  mobileRuntime,
  onCoverFileSelected,
  onInlineImageSelected,
  openEntry,
  refreshEntries,
  selectedActivityKey,
  selectedEntry,
  selectedEntryCreatedAtLabel,
  selectedEntryHtml,
  selectedEntryImageUrl,
  selectedEntryVisible,
  selectedSnapshotPreview,
  statusSnapshotDeviceName,
  statusSnapshotInput,
  submitting,
  submitEntry,
  uploadPending,
} = useInspirationWorkspace(props, {
  onPendingApproval: (info) => emit("pendingApproval", info),
  onKeyVerified: (generatedHashKey) => emit("keyVerified", generatedHashKey),
});
</script>

<template>
  <div class="workspace-grid">
    <InspirationComposeCard
      v-model:title="compose.title"
      v-model:content="compose.content"
      v-model:lexical-value="compose.contentLexical"
      v-model:compose-tab="composeTab"
      v-model:status-snapshot-input="statusSnapshotInput"
      v-model:status-snapshot-device-name="statusSnapshotDeviceName"
      v-model:selected-activity-key="selectedActivityKey"
      v-model:attach-current-status="attachCurrentStatus"
      v-model:attach-status-include-device-info="attachStatusIncludeDeviceInfo"
      v-model:cover-image-data-url="inlineImageDataUrl"
      :mobile-runtime="mobileRuntime"
      :activity-options="activityOptions"
      :activity-loading="activityLoading"
      :selected-snapshot-preview="selectedSnapshotPreview"
      :loading="loading"
      :submitting="submitting"
      :upload-pending="uploadPending"
      :inline-upload-pending="inlineUploadPending"
      :config-issues="configIssues"
      :load-error="loadError"
      :activity-load-error="activityLoadError"
      :compose-preview-html="composePreviewHtml"
      @refresh-entries="refreshEntries"
      @refresh-activities="loadActivityOptions"
      @cover-file-selected="onCoverFileSelected"
      @inline-image-selected="onInlineImageSelected"
      @submit="submitEntry"
    />

    <InspirationEntryListCard
      :entries="entryCards"
      :entry-count-label="entryCountLabel"
      :loading="loading"
      :load-error="loadError"
      :has-more-entries="hasMoreEntries"
      :loading-more="loadingMore"
      @open="openEntry"
      @load-more="loadMoreEntries"
    />

    <InspirationEntryDialog
      v-model:visible="selectedEntryVisible"
      :entry="selectedEntry"
      :created-at-label="selectedEntryCreatedAtLabel"
      :image-url="selectedEntryImageUrl"
      :content-html="selectedEntryHtml"
    />
  </div>
</template>

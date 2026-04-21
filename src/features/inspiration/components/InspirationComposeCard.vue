<script setup lang="ts">
import { useI18n } from "vue-i18n";
import Button from "primevue/button";
import Card from "primevue/card";

import InspirationComposeAssetsSection from "@/features/inspiration/components/InspirationComposeAssetsSection.vue";
import InspirationComposeBodySection from "@/features/inspiration/components/InspirationComposeBodySection.vue";
import InspirationComposeStatusSection from "@/features/inspiration/components/InspirationComposeStatusSection.vue";
import type {
  ActivitySelectOption,
  InspirationComposeTab,
} from "@/features/inspiration/types";

const title = defineModel<string>("title", { required: true });
const content = defineModel<string>("content", { required: true });
const lexicalValue = defineModel<string>("lexicalValue", { required: true });
const composeTab = defineModel<InspirationComposeTab>("composeTab", { required: true });
const statusSnapshotInput = defineModel<string>("statusSnapshotInput", { required: true });
const statusSnapshotDeviceName = defineModel<string>("statusSnapshotDeviceName", { required: true });
const selectedActivityKey = defineModel<string>("selectedActivityKey", { required: true });
const attachCurrentStatus = defineModel<boolean>("attachCurrentStatus", { required: true });
const attachStatusIncludeDeviceInfo = defineModel<boolean>("attachStatusIncludeDeviceInfo", { required: true });
const coverImageDataUrl = defineModel<string>("coverImageDataUrl", { required: true });

defineProps<{
  mobileRuntime: boolean;
  activityOptions: ActivitySelectOption[];
  activityLoading: boolean;
  selectedSnapshotPreview: string;
  loading: boolean;
  submitting: boolean;
  uploadPending: boolean;
  inlineUploadPending: boolean;
  configIssues: string[];
  loadError: string;
  activityLoadError: string;
  composePreviewHtml: string;
}>();

const emit = defineEmits<{
  refreshEntries: [];
  refreshActivities: [];
  coverFileSelected: [event: Event];
  inlineImageSelected: [event: Event];
  submit: [];
}>();

const { t } = useI18n();
</script>

<template>
  <Card class="glass-card inspiration-compose">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("inspiration.title.eyebrow") }}</p>
          <h3>{{ t("inspiration.title.title") }}</h3>
        </div>
        <Button
          :label="t('inspiration.buttons.refresh')"
          icon="pi pi-refresh"
          severity="secondary"
          text
          :loading="loading"
          @click="emit('refreshEntries')"
        />
      </div>
    </template>

    <template #content>
      <div class="panel-grid">
        <InspirationComposeStatusSection
          v-model:attach-current-status="attachCurrentStatus"
          v-model:attach-status-include-device-info="attachStatusIncludeDeviceInfo"
          v-model:status-snapshot-input="statusSnapshotInput"
          v-model:status-snapshot-device-name="statusSnapshotDeviceName"
          v-model:selected-activity-key="selectedActivityKey"
          :mobile-runtime="mobileRuntime"
          :activity-options="activityOptions"
          :activity-loading="activityLoading"
          :selected-snapshot-preview="selectedSnapshotPreview"
          @refresh-activities="emit('refreshActivities')"
        />

        <InspirationComposeBodySection
          v-model:title="title"
          v-model:content="content"
          v-model:lexical-value="lexicalValue"
          v-model:compose-tab="composeTab"
          :cover-image-data-url="coverImageDataUrl"
          :compose-preview-html="composePreviewHtml"
          :inline-upload-pending="inlineUploadPending"
          @inline-image-selected="emit('inlineImageSelected', $event)"
        />
      </div>

      <InspirationComposeAssetsSection
        :cover-image-data-url="coverImageDataUrl"
        :submitting="submitting"
        :upload-pending="uploadPending"
        :config-issues="configIssues"
        :load-error="loadError"
        :activity-load-error="activityLoadError"
        @cover-file-selected="emit('coverFileSelected', $event)"
        @submit="emit('submit')"
      />
    </template>
  </Card>
</template>

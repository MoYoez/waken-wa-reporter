<script setup lang="ts">
import { onErrorCaptured, ref } from "vue";
import { useI18n } from "vue-i18n";
import Button from "primevue/button";
import Card from "primevue/card";
import Image from "primevue/image";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import Select from "primevue/select";
import Textarea from "primevue/textarea";
import ToggleSwitch from "primevue/toggleswitch";

import LexicalEditor from "@/features/inspiration/components/LexicalEditor.vue";
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
const bodyImageInput = ref<HTMLInputElement | null>(null);
const editorFaulted = ref(false);

onErrorCaptured((error, instance, info) => {
  console.error("[InspirationComposeCard] render error:", error, info, instance);
  editorFaulted.value = true;
  return false;
});
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
        <div class="field-block field-span-2">
          <span class="field-label">{{ t("inspiration.fields.title") }}</span>
          <InputText v-model="title" :placeholder="t('inspiration.placeholders.title')" />
        </div>

        <div class="field-block field-span-2">
          <span class="field-label">
            {{ mobileRuntime ? t("inspiration.fields.statusMobile") : t("inspiration.fields.statusDesktop") }}
          </span>
          <div class="activity-toggle-row">
            <ToggleSwitch v-model="attachCurrentStatus" input-id="attach-current-status" />
            <label for="attach-current-status">
              {{
                mobileRuntime
                  ? t("inspiration.toggles.attachStatusMobile")
                  : t("inspiration.toggles.attachStatusDesktop")
              }}
            </label>
            <ToggleSwitch
              v-model="attachStatusIncludeDeviceInfo"
              input-id="attach-device-info"
              :disabled="!attachCurrentStatus"
            />
            <label for="attach-device-info">{{ t("inspiration.toggles.attachDeviceInfo") }}</label>
          </div>
          <div v-if="mobileRuntime" class="activity-select-row">
            <InputText
              v-model="statusSnapshotInput"
              :disabled="!attachCurrentStatus"
              :placeholder="t('inspiration.placeholders.statusInput')"
            />
          </div>
          <div v-if="mobileRuntime" class="activity-select-row">
            <InputText
              v-model="statusSnapshotDeviceName"
              :disabled="!attachCurrentStatus"
              :placeholder="t('inspiration.placeholders.deviceName')"
            />
          </div>
          <div v-else class="activity-select-row">
            <Select
              v-model="selectedActivityKey"
              :options="activityOptions"
              option-label="label"
              option-value="value"
              show-clear
              filter
              :loading="activityLoading"
              :disabled="!attachCurrentStatus"
              :placeholder="t('inspiration.placeholders.activitySelect')"
            />
            <Button
              icon="pi pi-refresh"
              severity="secondary"
              text
              :loading="activityLoading"
              :aria-label="t('inspiration.buttons.refreshActivities')"
              :title="t('inspiration.buttons.refreshActivities')"
              @click="emit('refreshActivities')"
            />
          </div>
          <small class="field-help">
            {{
              mobileRuntime
                ? t("inspiration.help.statusMobile")
                : t("inspiration.help.statusDesktop")
            }}
          </small>
          <div v-if="attachCurrentStatus && selectedSnapshotPreview" class="snapshot-preview">
            <strong>{{ t("inspiration.help.snapshotPreview") }}</strong>
            <span>{{ selectedSnapshotPreview }}</span>
          </div>
        </div>

        <div class="field-block field-span-2">
          <span class="field-label">{{ t("inspiration.fields.body") }}</span>
          <input
            ref="bodyImageInput"
            type="file"
            accept="image/*"
            class="sr-only"
            @change="emit('inlineImageSelected', $event)"
          />
          <div class="editor-mode-tabs">
            <button
              type="button"
              class="editor-mode-tab"
              :class="{ active: composeTab === 'edit' }"
              @click="composeTab = 'edit'"
            >
              {{ t("inspiration.tabs.edit") }}
            </button>
            <button
              type="button"
              class="editor-mode-tab"
              :class="{ active: composeTab === 'preview' }"
              @click="composeTab = 'preview'"
            >
              {{ t("inspiration.tabs.preview") }}
            </button>
          </div>
          <div class="editor-asset-actions">
            <Button
              :label="t('inspiration.buttons.insertInlineImage')"
              icon="pi pi-image"
              text
              size="small"
              :loading="inlineUploadPending"
              @click="bodyImageInput?.click()"
            />
            <small>{{ t("inspiration.help.inlineImage") }}</small>
          </div>
          <div v-show="composeTab === 'edit'" class="editor-tab-panel">
            <LexicalEditor
              v-if="!editorFaulted"
              v-model="content"
              v-model:lexical-value="lexicalValue"
              :placeholder="t('inspiration.placeholders.editor')"
            />
            <div v-else class="editor-fallback">
              <Message severity="warn" :closable="false">
                {{ t("inspiration.help.editorFallback") }}
              </Message>
              <Textarea
                v-model="content"
                rows="12"
                auto-resize
                :placeholder="t('inspiration.placeholders.fallbackEditor')"
              />
            </div>
          </div>
          <div v-show="composeTab === 'preview'" class="compose-live-preview compose-tab-preview">
            <article class="entry-card compose-preview-card">
              <div class="entry-header">
                <div>
                  <h4>{{ title.trim() || t("inspiration.common.untitledEntry") }}</h4>
                  <small>{{ t("inspiration.help.unpublished") }}</small>
                </div>
              </div>

              <Image
                v-if="coverImageDataUrl.trim()"
                :src="coverImageDataUrl"
                :alt="t('inspiration.imageAlt.draftCover')"
                image-class="entry-detail-image"
              />

              <div
                v-if="composePreviewHtml"
                class="entry-content markdown-content"
                v-html="composePreviewHtml"
              />
              <Message v-else severity="secondary" :closable="false">
                {{ t("inspiration.help.previewEmpty") }}
              </Message>
            </article>
          </div>
        </div>
      </div>

      <div class="inspiration-upload">
        <label class="upload-label">
          <input
            type="file"
            accept="image/png,image/jpeg,image/webp,image/gif"
            @change="emit('coverFileSelected', $event)"
          />
          <span><i class="pi pi-image" /> {{ t("inspiration.buttons.selectCover") }}</span>
        </label>
        <Button
          :label="t('inspiration.buttons.submit')"
          icon="pi pi-send"
          :loading="submitting || uploadPending"
          @click="emit('submit')"
        />
      </div>

      <div class="message-stack">
        <Message v-if="configIssues.length" severity="warn" :closable="false">
          {{ t("inspiration.help.configIssues") }}
        </Message>
        <Message v-if="loadError" severity="error" :closable="false">
          {{ loadError }}
        </Message>
        <Message v-if="activityLoadError" severity="warn" :closable="false">
          {{ activityLoadError }}
        </Message>
        <Message severity="secondary" :closable="false">
          {{ t("inspiration.help.uploadHint") }}
        </Message>
      </div>

      <div v-if="coverImageDataUrl" class="asset-preview">
        <div>
          <p class="field-label">{{ t("inspiration.fields.currentCover") }}</p>
          <strong>{{ t("inspiration.help.currentCoverTitle") }}</strong>
          <small>{{ t("inspiration.help.currentCoverDetail") }}</small>
        </div>
        <Image
          :src="coverImageDataUrl"
          :alt="t('inspiration.imageAlt.coverPreview')"
          image-class="inline-preview-image"
        />
      </div>
    </template>
  </Card>
</template>

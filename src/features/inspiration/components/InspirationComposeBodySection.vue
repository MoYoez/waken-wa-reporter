<script setup lang="ts">
import { onErrorCaptured, ref } from "vue";
import { useI18n } from "vue-i18n";
import Button from "primevue/button";
import Image from "primevue/image";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import Textarea from "primevue/textarea";

import LexicalEditor from "@/features/inspiration/components/LexicalEditor.vue";
import type { InspirationComposeTab } from "@/features/inspiration/types";

const title = defineModel<string>("title", { required: true });
const content = defineModel<string>("content", { required: true });
const lexicalValue = defineModel<string>("lexicalValue", { required: true });
const composeTab = defineModel<InspirationComposeTab>("composeTab", { required: true });

defineProps<{
  coverImageDataUrl: string;
  composePreviewHtml: string;
  inlineUploadPending: boolean;
}>();

const emit = defineEmits<{
  inlineImageSelected: [event: Event];
}>();

const { t } = useI18n();
const bodyImageInput = ref<HTMLInputElement | null>(null);
const editorFaulted = ref(false);

onErrorCaptured((error, instance, info) => {
  console.error("[InspirationComposeBodySection] render error:", error, info, instance);
  editorFaulted.value = true;
  return false;
});
</script>

<template>
  <div class="field-block field-span-2">
    <span class="field-label">{{ t("inspiration.fields.title") }}</span>
    <InputText v-model="title" :placeholder="t('inspiration.placeholders.title')" />
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
</template>

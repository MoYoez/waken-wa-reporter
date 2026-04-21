<script setup lang="ts">
import Button from "primevue/button";
import Image from "primevue/image";
import Message from "primevue/message";
import { useI18n } from "vue-i18n";

defineProps<{
  coverImageDataUrl: string;
  submitting: boolean;
  uploadPending: boolean;
  configIssues: string[];
  loadError: string;
  activityLoadError: string;
}>();

const emit = defineEmits<{
  coverFileSelected: [event: Event];
  submit: [];
}>();

const { t } = useI18n();
</script>

<template>
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

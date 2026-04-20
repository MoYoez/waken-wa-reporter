<script setup lang="ts">
import Dialog from "primevue/dialog";
import Image from "primevue/image";
import { useI18n } from "vue-i18n";

import type { InspirationEntry } from "@/types";

const visible = defineModel<boolean>("visible", { required: true });

defineProps<{
  entry: InspirationEntry | null;
  createdAtLabel: string;
  imageUrl: string;
  contentHtml: string;
}>();

const { t } = useI18n();
</script>

<template>
  <Dialog
    v-model:visible="visible"
    modal
    dismissable-mask
    :draggable="false"
    class="entry-detail-dialog"
  >
    <template #header>
      <div v-if="entry" class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("inspiration.list.detailEyebrow") }}</p>
          <h3>{{ entry.title || t("inspiration.common.untitledEntry") }}</h3>
        </div>
        <small>{{ createdAtLabel }}</small>
      </div>
    </template>

    <div v-if="entry" class="entry-detail-content">
      <Image
        v-if="imageUrl"
        :src="imageUrl"
        :alt="t('inspiration.imageAlt.entryAttachment')"
        image-class="entry-detail-image"
      />
      <div
        v-if="contentHtml"
        class="entry-content markdown-content"
        v-html="contentHtml"
      />
      <blockquote v-if="entry.statusSnapshot" class="status-snapshot">
        {{ entry.statusSnapshot }}
      </blockquote>
    </div>
  </Dialog>
</template>

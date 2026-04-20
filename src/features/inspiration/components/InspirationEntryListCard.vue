<script setup lang="ts">
import Button from "primevue/button";
import Card from "primevue/card";
import Image from "primevue/image";
import Message from "primevue/message";
import Tag from "primevue/tag";
import { useI18n } from "vue-i18n";

import type { InspirationEntryCardView } from "@/features/inspiration/types";
import type { InspirationEntry } from "@/types";

defineProps<{
  entries: InspirationEntryCardView[];
  entryCountLabel: string;
  loading: boolean;
  loadError: string;
  hasMoreEntries: boolean;
  loadingMore: boolean;
}>();

const emit = defineEmits<{
  open: [entry: InspirationEntry];
  loadMore: [];
}>();

const { t } = useI18n();
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("inspiration.list.eyebrow") }}</p>
          <h3>{{ t("inspiration.list.title") }}</h3>
        </div>
        <Tag :value="entryCountLabel" rounded />
      </div>
    </template>
    <template #content>
      <div v-if="entries.length" class="entry-list">
        <article
          v-for="(item, index) in entries"
          :key="`inspiration-${item.entry.id ?? 'na'}-${item.entry.createdAt}-${index}`"
          class="entry-card entry-card-button"
          @click="emit('open', item.entry)"
        >
          <div class="entry-header">
            <div>
              <h4>{{ item.entry.title || t("inspiration.common.untitledEntry") }}</h4>
              <small>{{ item.createdAtLabel }}</small>
            </div>
            <Tag
              v-if="item.entry.statusSnapshot"
              :value="t('inspiration.list.statusSnapshotTag')"
              severity="contrast"
              rounded
            />
          </div>

          <Image
            v-if="item.previewImageUrl"
            :src="item.previewImageUrl"
            :alt="t('inspiration.imageAlt.entryAttachment')"
            image-class="entry-preview-image"
          />

          <p
            v-if="item.previewText"
            class="entry-content entry-preview-text"
          >
            {{ item.previewText }}
          </p>
          <blockquote v-if="item.entry.statusSnapshot" class="status-snapshot">
            {{ item.entry.statusSnapshot }}
          </blockquote>
          <div class="entry-card-footer">
            <span>{{ t("inspiration.list.viewFull") }}</span>
            <i class="pi pi-angle-right" />
          </div>
        </article>
      </div>
      <Message v-else-if="!loading && !loadError" severity="secondary" :closable="false">
        {{ t("inspiration.list.empty") }}
      </Message>
      <div v-if="entries.length && hasMoreEntries" class="entry-list-actions">
        <Button
          :label="t('inspiration.list.loadMore')"
          icon="pi pi-angle-down"
          severity="secondary"
          outlined
          :loading="loadingMore"
          @click="emit('loadMore')"
        />
      </div>
    </template>
  </Card>
</template>

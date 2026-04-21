<script setup lang="ts">
import { useI18n } from "vue-i18n";
import Card from "primevue/card";
import Message from "primevue/message";

import { formatOptionalDateTime } from "@/lib/dateTime";
import type { RecentPreset } from "@/types";

const { t, locale } = useI18n();

defineProps<{
  recentPresets: RecentPreset[];
}>();

defineEmits<{
  selectPreset: [preset: RecentPreset];
}>();

function formatTime(value: string) {
  return formatOptionalDateTime(value, locale.value, value);
}
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("activity.recent.eyebrow") }}</p>
          <h3>{{ t("activity.recent.title") }}</h3>
        </div>
      </div>
    </template>
    <template #content>
      <div v-if="recentPresets.length" class="preset-grid">
        <button
          v-for="preset in recentPresets"
          :key="`${preset.process_name}-${preset.lastUsedAt}`"
          class="preset-card"
          type="button"
          @click="$emit('selectPreset', preset)"
        >
          <strong>{{ preset.process_name }}</strong>
          <span>{{ preset.process_title || t("activity.common.windowTitleFallback") }}</span>
          <small>{{ formatTime(preset.lastUsedAt) }}</small>
        </button>
      </div>
      <Message v-else severity="secondary" :closable="false">
        {{ t("activity.recent.empty") }}
      </Message>
    </template>
  </Card>
</template>

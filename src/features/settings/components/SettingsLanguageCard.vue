<script setup lang="ts">
import Button from "primevue/button";
import Card from "primevue/card";
import Message from "primevue/message";
import Select from "primevue/select";
import { useI18n } from "vue-i18n";

import {
  localeOptions,
  type SupportedLocale,
} from "@/i18n";

defineProps<{
  locale: SupportedLocale;
  localeRestartRequired: boolean;
  restarting: boolean;
}>();

const emit = defineEmits<{
  updateLocale: [value: SupportedLocale];
  restartApp: [];
}>();

const { t } = useI18n();
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("settings.language.eyebrow") }}</p>
          <h3>{{ t("settings.language.title") }}</h3>
        </div>
      </div>
    </template>
    <template #content>
      <div class="panel-grid">
        <label class="field-block field-span-2">
          <span class="field-label">{{ t("settings.language.field") }}</span>
          <Select
            :model-value="locale"
            :options="localeOptions"
            option-label="label"
            option-value="value"
            @update:model-value="(value) => value && emit('updateLocale', value)"
          />
        </label>
      </div>
      <div class="message-stack">
        <Message severity="secondary" :closable="false">
          {{ t("settings.language.description") }}
        </Message>
        <Message
          v-if="localeRestartRequired"
          severity="warn"
          :closable="false"
        >
          <div class="inline-message-action">
            <span>{{ t("settings.language.restartHint") }}</span>
            <Button
              :label="t('settings.language.restartNow')"
              icon="pi pi-refresh"
              size="small"
              :loading="restarting"
              @click="emit('restartApp')"
            />
          </div>
        </Message>
      </div>
    </template>
  </Card>
</template>

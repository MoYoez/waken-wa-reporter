<script setup lang="ts">
import Button from "primevue/button";
import Card from "primevue/card";
import InputNumber from "primevue/inputnumber";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import ToggleSwitch from "primevue/toggleswitch";
import { useI18n } from "vue-i18n";

import type { ActivityFormState } from "@/features/activity/composables/useActivityWorkspace";

const { t } = useI18n();

defineProps<{
  form: ActivityFormState;
  mobileRuntime: boolean;
  hasConfigIssues: boolean;
  submitting: boolean;
}>();

defineEmits<{
  submit: [];
}>();
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("activity.title.eyebrow") }}</p>
          <h3>{{ t("activity.title.title") }}</h3>
        </div>
      </div>
    </template>
    <template #content>
      <div class="activity-form-stack">
        <div class="panel-grid">
          <label class="field-block">
            <span class="field-label">{{ t("activity.fields.name") }}</span>
            <InputText v-model="form.processName" :placeholder="t('activity.placeholders.name')" />
          </label>
        </div>

        <label class="field-block field-span-2">
          <span class="field-label">{{ t("activity.fields.title") }}</span>
          <InputText v-model="form.processTitle" :placeholder="t('activity.placeholders.title')" />
        </label>

        <div v-if="!mobileRuntime" class="panel-grid">
          <div class="reporter-enabled-card field-span-2">
            <div class="reporter-enabled-copy">
              <span class="field-label">{{ t("activity.fields.includeBattery") }}</span>
              <strong>{{ form.includeBattery ? t("activity.common.enabled") : t("activity.common.disabled") }}</strong>
              <span>{{ t("activity.help.includeBattery") }}</span>
            </div>
            <ToggleSwitch
              v-model="form.includeBattery"
              input-id="manual-include-battery"
            />
          </div>
        </div>

        <label class="field-block field-span-2">
          <span class="field-label">{{ t("activity.fields.persistMinutes") }}</span>
          <InputNumber v-model="form.persistMinutes" :min="1" :max="1440" fluid />
          <small class="field-help">
            {{ t("activity.help.persistMinutes") }}
          </small>
        </label>

        <div class="actions-row">
          <Button
            :label="t('activity.buttons.submit')"
            icon="pi pi-plus"
            :loading="submitting"
            @click="$emit('submit')"
          />
        </div>
      </div>

      <div class="message-stack">
        <Message v-if="hasConfigIssues" severity="warn" :closable="false">
          {{ t("activity.help.settingsWarning") }}
        </Message>
      </div>
    </template>
  </Card>
</template>

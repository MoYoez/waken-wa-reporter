<script setup lang="ts">
import Card from "primevue/card";
import Message from "primevue/message";
import ToggleSwitch from "primevue/toggleswitch";
import { useI18n } from "vue-i18n";

defineProps<{
  enabled: boolean;
}>();

const emit = defineEmits<{
  updateEnabled: [value: boolean];
}>();

const { t } = useI18n();
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("settings.startup.eyebrow") }}</p>
          <h3>{{ t("settings.startup.title") }}</h3>
        </div>
      </div>
    </template>
    <template #content>
      <div class="panel-grid">
        <div class="reporter-enabled-card field-span-2">
          <div class="reporter-enabled-copy">
            <span class="field-label">{{ t("settings.startup.toggle") }}</span>
            <strong>{{ enabled ? t("settings.tags.enabled") : t("settings.tags.disabled") }}</strong>
            <span>
              {{ t("settings.startup.description") }}
            </span>
          </div>
          <ToggleSwitch
            :model-value="enabled"
            input-id="settings-launch-on-startup"
            @update:model-value="emit('updateEnabled', Boolean($event))"
          />
        </div>
      </div>
      <div class="message-stack">
        <Message severity="secondary" :closable="false">
          {{ t("settings.startup.hint") }}
        </Message>
      </div>
    </template>
  </Card>
</template>

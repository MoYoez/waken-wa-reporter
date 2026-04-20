<script setup lang="ts">
import Button from "primevue/button";
import Textarea from "primevue/textarea";
import { useI18n } from "vue-i18n";

const text = defineModel<string>("text", { required: true });

defineProps<{
  titleKey: string;
  helpKey: string;
}>();

const emit = defineEmits<{
  import: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="settings-section">
    <div class="settings-section-head">
      <strong>{{ t(titleKey) }}</strong>
      <span>{{ t(helpKey) }}</span>
    </div>
    <div class="panel-grid">
      <label class="field-block field-span-2">
        <span class="field-label">{{ t("connectionPanel.fields.importConfig") }}</span>
        <Textarea
          v-model="text"
          rows="4"
          auto-resize
          :placeholder="t('connectionPanel.placeholders.importConfig')"
        />
      </label>
    </div>
    <div class="actions-row">
      <Button :label="t('connectionPanel.buttons.import')" icon="pi pi-upload" @click="emit('import')" />
    </div>
  </div>
</template>

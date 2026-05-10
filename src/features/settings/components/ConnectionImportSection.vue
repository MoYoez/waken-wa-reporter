<script setup lang="ts">
import Button from "primevue/button";
import Textarea from "primevue/textarea";
import { useI18n } from "vue-i18n";

const text = defineModel<string>("text", { required: true });

defineProps<{
  titleKey: string;
  helpKey: string;
  canScanQr?: boolean;
  scanQrLoading?: boolean;
}>();

const emit = defineEmits<{
  import: [];
  scanQr: [];
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
      <Button
        :label="t('connectionPanel.buttons.import')"
        icon="pi pi-upload"
        :disabled="scanQrLoading"
        @click="emit('import')"
      />
      <Button
        v-if="canScanQr"
        :label="t('connectionPanel.buttons.scanQr')"
        icon="pi pi-camera"
        outlined
        :loading="scanQrLoading"
        @click="emit('scanQr')"
      />
    </div>
  </div>
</template>

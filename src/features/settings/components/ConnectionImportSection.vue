<script setup lang="ts">
import { onBeforeUnmount, toRefs, watch } from "vue";
import Button from "primevue/button";
import Textarea from "primevue/textarea";
import { useI18n } from "vue-i18n";

const text = defineModel<string>("text", { required: true });

const props = defineProps<{
  titleKey: string;
  helpKey: string;
  canScanQr?: boolean;
  scanQrLoading?: boolean;
}>();

const { titleKey, helpKey, canScanQr, scanQrLoading } = toRefs(props);

const emit = defineEmits<{
  import: [];
  scanQr: [];
  cancelScanQr: [];
}>();

const { t } = useI18n();

function setScanMode(active: boolean) {
  if (typeof document === "undefined") {
    return;
  }

  document.documentElement.classList.toggle("qr-scan-active", active);
}

watch(
  scanQrLoading,
  (active) => {
    setScanMode(Boolean(active));
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  setScanMode(false);
});
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
    <Teleport to="body">
      <div v-if="scanQrLoading" class="qr-scan-overlay" role="status" aria-live="polite">
        <div class="qr-scan-frame" aria-hidden="true">
          <span />
        </div>
      <div class="qr-scan-copy">
        <strong>{{ t("connectionPanel.qrScan.title") }}</strong>
        <span>{{ t("connectionPanel.qrScan.detail") }}</span>
      </div>
        <Button
          :label="t('connectionPanel.buttons.cancelScan')"
          icon="pi pi-times"
          severity="secondary"
          outlined
          @click="emit('cancelScanQr')"
        />
      </div>
    </Teleport>
  </div>
</template>

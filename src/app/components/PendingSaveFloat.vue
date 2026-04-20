<script setup lang="ts">
import Button from "primevue/button";
import { useI18n } from "vue-i18n";

const emit = defineEmits<{
  apply: [];
  revert: [];
}>();

const { t } = useI18n();
</script>

<template>
  <section class="pending-save-float" aria-live="polite">
    <div class="pending-save-float-copy">
      <p class="eyebrow">{{ t("app.pendingSave.eyebrow") }}</p>
      <strong>{{ t("app.pendingSave.title") }}</strong>
      <span>{{ t("app.pendingSave.detail") }}</span>
    </div>
    <div class="pending-save-float-actions">
      <Button
        :label="t('app.pendingSave.apply')"
        icon="pi pi-check"
        size="small"
        @click="emit('apply')"
      />
      <Button
        :label="t('app.pendingSave.revert')"
        icon="pi pi-undo"
        severity="secondary"
        outlined
        size="small"
        @click="emit('revert')"
      />
    </div>
  </section>
</template>

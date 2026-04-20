<script setup lang="ts">
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import { useI18n } from "vue-i18n";

defineProps<{
  visible: boolean;
  message?: string | null;
  approvalUrl?: string | null;
}>();

const emit = defineEmits<{
  close: [];
}>();

const { t } = useI18n();
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    dismissable-mask
    :draggable="false"
    :header="t('app.pendingApproval.header')"
    style="width: min(560px, calc(100vw - 24px))"
    @update:visible="(nextVisible) => !nextVisible && emit('close')"
  >
    <div class="onboarding-steps">
      <div class="onboarding-step">
        <strong>{{ message || t("app.pendingApproval.defaultMessage") }}</strong>
        <span>{{ t("app.pendingApproval.detail") }}</span>
      </div>
      <div v-if="approvalUrl" class="onboarding-step onboarding-highlight">
        <strong>{{ t("app.pendingApproval.approvalUrl") }}</strong>
        <span>{{ approvalUrl }}</span>
      </div>
    </div>
    <div class="actions-row">
      <Button :label="t('app.pendingApproval.confirm')" icon="pi pi-check" @click="emit('close')" />
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import Button from "primevue/button";
import Card from "primevue/card";
import Message from "primevue/message";

import type { MobileConnectivityState } from "@/types";

const { t } = useI18n();

const props = defineProps<{
  readiness: boolean;
  mobileConnectivity: MobileConnectivityState;
}>();

defineEmits<{
  retryMobileConnectivity: [];
}>();

const connectivityLabel = computed(() => {
  if (props.mobileConnectivity.checking) {
    return t("overview.mobile.checking");
  }
  if (props.mobileConnectivity.ok === true) {
    return t("overview.mobile.passed");
  }
  if (props.mobileConnectivity.checked) {
    return t("overview.mobile.failed");
  }
  return t("overview.mobile.pending");
});

const connectivitySeverity = computed(() => {
  if (props.mobileConnectivity.ok === true) {
    return "success";
  }
  if (props.mobileConnectivity.checked) {
    return "warn";
  }
  return "secondary";
});
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("overview.mobile.eyebrow") }}</p>
          <h3>{{ t("overview.mobile.title") }}</h3>
        </div>
      </div>
    </template>
    <template #content>
      <div class="overview-summary">
        <div class="overview-item">
          <span>{{ t("overview.mobile.baseConfig") }}</span>
          <strong>{{ readiness ? t("overview.mobile.ready") : t("overview.mobile.incomplete") }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("overview.mobile.realtimeReporter") }}</span>
          <strong>{{ t("overview.mobile.disabled") }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("overview.mobile.manualActivity") }}</span>
          <strong>{{ t("overview.mobile.available") }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("overview.mobile.inspiration") }}</span>
          <strong>{{ t("overview.mobile.available") }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("overview.mobile.connectivity") }}</span>
          <strong>{{ connectivityLabel }}</strong>
        </div>
      </div>
      <div class="actions-row">
        <Button
          :label="t('overview.mobile.retry')"
          icon="pi pi-refresh"
          severity="secondary"
          outlined
          :loading="mobileConnectivity.checking"
          :disabled="!readiness"
          @click="$emit('retryMobileConnectivity')"
        />
      </div>
      <div class="message-stack">
        <Message :severity="connectivitySeverity" :closable="false">
          <strong>{{ mobileConnectivity.summary || t("overview.mobile.mobileMode") }}</strong>
          <br v-if="mobileConnectivity.detail" />
          {{ mobileConnectivity.detail || t("overview.mobile.defaultDetail") }}
        </Message>
        <Message v-if="!readiness" severity="secondary" :closable="false">
          {{ t("overview.mobile.defaultDetail") }}
        </Message>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import Button from "primevue/button";
import Card from "primevue/card";
import Message from "primevue/message";

import type { ClientConfig, RealtimeReporterSnapshot } from "@/types";

const { t } = useI18n();

const props = defineProps<{
  config: ClientConfig;
  readiness: boolean;
  snapshot: RealtimeReporterSnapshot;
  reporterBusy: boolean;
}>();

defineEmits<{
  startReporter: [];
  stopReporter: [];
}>();

const currentProcessName = computed(() => props.snapshot.currentActivity?.processName || t("overview.common.none"));
const currentWindowTitle = computed(() => props.snapshot.currentActivity?.processTitle || t("overview.common.none"));
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("overview.reporter.eyebrow") }}</p>
          <h3>{{ t("overview.reporter.title") }}</h3>
        </div>
      </div>
    </template>
    <template #content>
      <div class="overview-summary">
        <div class="overview-item">
          <span>{{ t("overview.reporter.baseConfig") }}</span>
          <strong>{{ readiness ? t("overview.reporter.ready") : t("overview.reporter.incomplete") }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("overview.reporter.currentProcess") }}</span>
          <strong>{{ currentProcessName }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("overview.reporter.windowTitle") }}</span>
          <strong>{{ currentWindowTitle }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("overview.reporter.pollInterval") }}</span>
          <strong>{{ config.pollIntervalMs }} ms</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("overview.reporter.autoStart") }}</span>
          <strong>{{ config.reporterEnabled ? t("overview.reporter.enabled") : t("overview.reporter.disabled") }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("overview.reporter.heartbeatInterval") }}</span>
          <strong>{{ config.heartbeatIntervalMs }} ms</strong>
        </div>
      </div>

      <div class="actions-row">
        <Button
          :label="t('overview.reporter.start')"
          icon="pi pi-play"
          :loading="reporterBusy"
          :disabled="snapshot.running || !readiness"
          @click="$emit('startReporter')"
        />
        <Button
          :label="t('overview.reporter.stop')"
          icon="pi pi-stop"
          severity="secondary"
          outlined
          :loading="reporterBusy"
          :disabled="!snapshot.running"
          @click="$emit('stopReporter')"
        />
      </div>

      <div class="message-stack">
        <Message v-if="snapshot.lastError" severity="error" :closable="false">
          {{ snapshot.lastError }}
        </Message>
        <Message
          v-else-if="config.reporterEnabled"
          severity="success"
          :closable="false"
        >
          {{ t("overview.reporter.autoStartMessage") }}
        </Message>
        <Message v-else-if="!snapshot.running" severity="secondary" :closable="false">
          {{ t("overview.reporter.idleMessage") }}
        </Message>
      </div>
    </template>
  </Card>
</template>

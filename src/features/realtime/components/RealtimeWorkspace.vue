<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import Card from "primevue/card";
import Dialog from "primevue/dialog";
import Message from "primevue/message";
import Paginator from "primevue/paginator";
import Tag from "primevue/tag";

import { resolveReporterLogDetail, resolveReporterLogTitle } from "@/lib/reporterLogText";
import type { ReporterLogEntry, RealtimeReporterSnapshot } from "@/types";

const { t, locale } = useI18n();

const props = defineProps<{
  snapshot: RealtimeReporterSnapshot;
}>();

const selectedLog = ref<ReporterLogEntry | null>(null);
const first = ref(0);
const rows = 10;

const pagedLogs = computed(() => props.snapshot.logs.slice(first.value, first.value + rows));

const selectedLogPayloadText = computed(() => {
  if (!selectedLog.value?.payload) return "";
  return JSON.stringify(selectedLog.value.payload, null, 2);
});

function formatTime(value?: string | null) {
  if (!value) return t("realtime.summary.none");

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString(locale.value);
}

function levelLabel(level: ReporterLogEntry["level"]) {
  if (level === "success") return t("realtime.levels.success");
  if (level === "error") return t("realtime.levels.error");
  if (level === "warn") return t("realtime.levels.warn");
  return t("realtime.levels.info");
}

function levelSeverity(level: ReporterLogEntry["level"]) {
  if (level === "success") return "success";
  if (level === "error") return "danger";
  if (level === "warn") return "warn";
  return "info";
}

function openLog(log: ReporterLogEntry) {
  selectedLog.value = log;
}

function closeLogDialog() {
  selectedLog.value = null;
}

function translateLogText(key: string, params?: Record<string, unknown> | null) {
  return params ? t(key, params) : t(key);
}

function logTitle(log: ReporterLogEntry) {
  return resolveReporterLogTitle(log, translateLogText);
}

function logDetail(log: ReporterLogEntry) {
  return resolveReporterLogDetail(log, translateLogText);
}

watch(
  () => props.snapshot.logs.length,
  (length) => {
    if (first.value >= length) {
      first.value = Math.max(0, Math.floor(Math.max(length - 1, 0) / rows) * rows);
    }
  },
);
</script>

<template>
  <div class="workspace-grid">
    <Card class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("realtime.title.eyebrow") }}</p>
            <h3>{{ t("realtime.title.title") }}</h3>
          </div>
          <Tag
            :value="snapshot.running ? t('realtime.status.running') : t('realtime.status.notStarted')"
            :severity="snapshot.running ? 'success' : 'warn'"
            rounded
          />
        </div>
      </template>
      <template #content>
        <div class="overview-summary">
          <div class="overview-item">
            <span>{{ t("realtime.summary.currentProcess") }}</span>
            <strong>{{ snapshot.currentActivity?.processName || t("realtime.summary.none") }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("realtime.summary.windowTitle") }}</span>
            <strong>{{ snapshot.currentActivity?.processTitle || t("realtime.summary.none") }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("realtime.summary.lastHeartbeat") }}</span>
            <strong>{{ formatTime(snapshot.lastHeartbeatAt) }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("realtime.summary.logTotal") }}</span>
            <strong>{{ snapshot.logs.length }}</strong>
          </div>
        </div>
      </template>
    </Card>

    <Card class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("realtime.list.eyebrow") }}</p>
            <h3>{{ t("realtime.list.title") }}</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div v-if="snapshot.logs.length" class="log-list">
          <button
            v-for="log in pagedLogs"
            :key="log.id"
            class="log-item log-item-button"
            type="button"
            @click="openLog(log)"
          >
            <div class="log-header">
              <div class="log-title-row">
                <strong>{{ logTitle(log) }}</strong>
                <Tag :value="levelLabel(log.level)" :severity="levelSeverity(log.level)" rounded />
              </div>
              <small>{{ formatTime(log.timestamp) }}</small>
            </div>
            <p class="log-detail">{{ logDetail(log) }}</p>
          </button>
          <Paginator
            :first="first"
            :rows="rows"
            :total-records="snapshot.logs.length"
            template="PrevPageLink CurrentPageReport NextPageLink"
            :current-page-report-template="t('realtime.pagination.report')"
            @page="first = $event.first"
          />
        </div>
        <Message v-else severity="secondary" :closable="false">
          {{ t("realtime.list.empty") }}
        </Message>
      </template>
    </Card>

    <Dialog
      :visible="!!selectedLog"
      modal
      dismissable-mask
      :draggable="false"
      :header="selectedLog ? logTitle(selectedLog) : t('realtime.dialog.defaultHeader')"
      style="width: min(760px, calc(100vw - 24px))"
      @update:visible="(value) => !value && closeLogDialog()"
    >
      <div v-if="selectedLog" class="log-dialog-content">
        <div class="log-dialog-meta">
          <Tag
            :value="levelLabel(selectedLog.level)"
            :severity="levelSeverity(selectedLog.level)"
            rounded
          />
          <small>{{ formatTime(selectedLog.timestamp) }}</small>
        </div>
        <p class="log-detail">{{ logDetail(selectedLog) }}</p>
        <pre v-if="selectedLog.payload" class="payload-preview">{{ selectedLogPayloadText }}</pre>
      </div>
    </Dialog>
  </div>
</template>


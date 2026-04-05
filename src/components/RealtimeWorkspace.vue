<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Card from "primevue/card";
import Dialog from "primevue/dialog";
import Message from "primevue/message";
import Paginator from "primevue/paginator";
import Tag from "primevue/tag";

import type { ReporterLogEntry, RealtimeReporterSnapshot } from "../types";

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
  if (!value) return "暂无";
  return new Date(value).toLocaleString();
}

function levelLabel(level: ReporterLogEntry["level"]) {
  if (level === "success") return "成功";
  if (level === "error") return "错误";
  if (level === "warn") return "警告";
  return "信息";
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
            <p class="eyebrow">实时同步</p>
            <h3>查看后台同步状态和最近记录</h3>
          </div>
          <Tag
            :value="snapshot.running ? '运行中' : '未启动'"
            :severity="snapshot.running ? 'success' : 'warn'"
            rounded
          />
        </div>
      </template>
      <template #content>
        <div class="overview-summary">
          <div class="overview-item">
            <span>当前进程</span>
            <strong>{{ snapshot.currentActivity?.processName || "暂无" }}</strong>
          </div>
          <div class="overview-item">
            <span>窗口标题</span>
            <strong>{{ snapshot.currentActivity?.processTitle || "暂无" }}</strong>
          </div>
          <div class="overview-item">
            <span>最近心跳</span>
            <strong>{{ formatTime(snapshot.lastHeartbeatAt) }}</strong>
          </div>
          <div class="overview-item">
            <span>日志总数</span>
            <strong>{{ snapshot.logs.length }}</strong>
          </div>
        </div>
      </template>
    </Card>

    <Card class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">同步记录</p>
            <h3>按时间查看每一条同步结果</h3>
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
                <strong>{{ log.title }}</strong>
                <Tag :value="levelLabel(log.level)" :severity="levelSeverity(log.level)" rounded />
              </div>
              <small>{{ formatTime(log.timestamp) }}</small>
            </div>
            <p class="log-detail">{{ log.detail }}</p>
          </button>
          <Paginator
            :first="first"
            :rows="rows"
            :total-records="snapshot.logs.length"
            template="PrevPageLink CurrentPageReport NextPageLink"
            current-page-report-template="{first} - {last} / {totalRecords}"
            @page="first = $event.first"
          />
        </div>
        <Message v-else severity="secondary" :closable="false">
          暂无同步记录。你可以先到“设置”里开启后台同步。
        </Message>
      </template>
    </Card>

    <Dialog
      :visible="!!selectedLog"
      modal
      dismissable-mask
      :draggable="false"
      :header="selectedLog?.title || '同步详情'"
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
        <p class="log-detail">{{ selectedLog.detail }}</p>
        <pre v-if="selectedLog.payload" class="payload-preview">{{ selectedLogPayloadText }}</pre>
      </div>
    </Dialog>
  </div>
</template>

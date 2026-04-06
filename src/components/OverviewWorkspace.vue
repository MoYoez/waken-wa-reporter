<script setup lang="ts">
import { computed } from "vue";
import Button from "primevue/button";
import Card from "primevue/card";
import Message from "primevue/message";
import Tag from "primevue/tag";

import type {
  ClientCapabilities,
  ClientConfig,
  MobileConnectivityState,
  RealtimeReporterSnapshot,
} from "../types";

const props = defineProps<{
  config: ClientConfig;
  readiness: boolean;
  capabilities: ClientCapabilities;
  mobileConnectivity: MobileConnectivityState;
  reporterSnapshot: RealtimeReporterSnapshot;
  reporterBusy: boolean;
}>();

defineEmits<{
  startReporter: [];
  stopReporter: [];
  retryMobileConnectivity: [];
}>();

const latestLogs = computed(() => props.reporterSnapshot.logs.slice(0, 4));
const reporterSupported = computed(() => props.capabilities.realtimeReporter);
const effectiveModeLabel = computed(() => {
  if (!reporterSupported.value) {
    return "活动模式";
  }

  return props.reporterSnapshot.running ? "实时模式" : "活动模式";
});

function formatTime(value?: string | null) {
  if (!value) return "暂无";
  return new Date(value).toLocaleString();
}
</script>

<template>
  <div class="workspace-grid">
    <header class="hero-panel">
      <div>
        <p class="eyebrow">概览</p>
        <h2>概览</h2>
        <p class="hero-copy">
          在这里查看当前连接状态、设备信息和同步运行情况。
        </p>
      </div>
      <div class="hero-actions">
        <Tag
          v-if="reporterSupported"
          :value="reporterSnapshot.running ? '后台同步运行中' : '后台同步未开启'"
          :severity="reporterSnapshot.running ? 'success' : 'warn'"
          rounded
        />
        <Tag v-else value="移动端模式" severity="info" rounded />
      </div>
    </header>

    <Card class="glass-card overview-summary-card">
      <template #content>
        <div class="overview-summary">
          <div class="overview-item">
            <span>站点地址</span>
            <strong>{{ config.baseUrl || "未设置" }}</strong>
          </div>
          <div class="overview-item">
            <span>设备名称</span>
            <strong>{{ config.device || "未命名设备" }}</strong>
          </div>
          <div class="overview-item">
            <span>当前模式</span>
            <strong>{{ effectiveModeLabel }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ reporterSupported ? "最近心跳" : "设备类型" }}</span>
            <strong>{{ reporterSupported ? formatTime(reporterSnapshot.lastHeartbeatAt) : config.deviceType }}</strong>
          </div>
        </div>
      </template>
    </Card>

    <Card v-if="reporterSupported" class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">当前状态</p>
            <h3>后台同步</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div class="overview-summary">
          <div class="overview-item">
            <span>基础配置</span>
            <strong>{{ readiness ? "已就绪" : "待完善" }}</strong>
          </div>
          <div class="overview-item">
            <span>当前进程</span>
            <strong>{{ reporterSnapshot.currentActivity?.processName || "暂无" }}</strong>
          </div>
          <div class="overview-item">
            <span>窗口标题</span>
            <strong>{{ reporterSnapshot.currentActivity?.processTitle || "暂无" }}</strong>
          </div>
          <div class="overview-item">
            <span>轮询间隔</span>
            <strong>{{ config.pollIntervalMs }} ms</strong>
          </div>
          <div class="overview-item">
            <span>开机后自动同步</span>
            <strong>{{ config.reporterEnabled ? "已开启" : "未开启" }}</strong>
          </div>
          <div class="overview-item">
            <span>心跳间隔</span>
            <strong>{{ config.heartbeatIntervalMs }} ms</strong>
          </div>
        </div>

        <div class="actions-row">
          <Button
            label="开启后台同步"
            icon="pi pi-play"
            :loading="reporterBusy"
            :disabled="reporterSnapshot.running || !readiness"
            @click="$emit('startReporter')"
          />
          <Button
            label="停止后台同步"
            icon="pi pi-stop"
            severity="secondary"
            outlined
            :loading="reporterBusy"
            :disabled="!reporterSnapshot.running"
            @click="$emit('stopReporter')"
          />
        </div>

        <div class="message-stack">
          <Message v-if="reporterSnapshot.lastError" severity="error" :closable="false">
            {{ reporterSnapshot.lastError }}
          </Message>
          <Message
            v-else-if="config.reporterEnabled"
            severity="success"
            :closable="false"
          >
            已启用启动后自动同步，客户端下次打开时会自动尝试开始后台同步。
          </Message>
          <Message v-else-if="!reporterSnapshot.running" severity="secondary" :closable="false">
            后台同步当前未开启。你可以在“设置”里手动开启，或启用启动后自动同步。
          </Message>
        </div>
      </template>
    </Card>

    <Card v-else class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">当前状态</p>
            <h3>移动端能力</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div class="overview-summary">
          <div class="overview-item">
            <span>基础配置</span>
            <strong>{{ readiness ? "已就绪" : "待完善" }}</strong>
          </div>
          <div class="overview-item">
            <span>后台实时上报</span>
            <strong>已禁用</strong>
          </div>
          <div class="overview-item">
            <span>手动活动提交</span>
            <strong>可用</strong>
          </div>
          <div class="overview-item">
            <span>灵感发布</span>
            <strong>可用</strong>
          </div>
          <div class="overview-item">
            <span>连通性检查</span>
            <strong>
              {{
                mobileConnectivity.checking
                  ? "检测中"
                  : mobileConnectivity.ok === true
                    ? "已通过"
                    : mobileConnectivity.checked
                      ? "异常"
                      : "待检测"
              }}
            </strong>
          </div>
        </div>
        <div class="actions-row">
          <Button
            label="重新测试连接"
            icon="pi pi-refresh"
            severity="secondary"
            outlined
            :loading="mobileConnectivity.checking"
            :disabled="!readiness"
            @click="$emit('retryMobileConnectivity')"
          />
        </div>
        <div class="message-stack">
          <Message
            :severity="
              mobileConnectivity.ok === true
                ? 'success'
                : mobileConnectivity.checked
                  ? 'warn'
                  : 'secondary'
            "
            :closable="false"
          >
            <strong>{{ mobileConnectivity.summary || "移动端模式" }}</strong>
            <br v-if="mobileConnectivity.detail" />
            {{ mobileConnectivity.detail || "当前平台已关闭后台实时同步，适用于移动端前台使用场景。" }}
          </Message>
          <Message v-if="!readiness" severity="secondary" :closable="false">
            当前平台已关闭后台实时同步，适用于移动端前台使用场景。
          </Message>
        </div>
      </template>
    </Card>

    <Card v-if="reporterSupported" class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">最近动态</p>
            <h3>这里展示最近几次后台同步的内容摘要</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div v-if="latestLogs.length" class="log-list">
          <article v-for="log in latestLogs" :key="log.id" class="log-item">
            <div class="log-header">
              <strong>{{ log.title }}</strong>
              <small>{{ formatTime(log.timestamp) }}</small>
            </div>
            <p class="log-detail">{{ log.detail }}</p>
          </article>
        </div>
        <Message v-else severity="secondary" :closable="false">
          开启后台同步后，最近动态会显示在这里。
        </Message>
      </template>
    </Card>
  </div>
</template>

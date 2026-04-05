<script setup lang="ts">
import { computed, ref } from "vue";
import Button from "primevue/button";
import Card from "primevue/card";
import Message from "primevue/message";
import Tag from "primevue/tag";
import { useToast } from "primevue/usetoast";

import ConnectionPanel from "./ConnectionPanel.vue";
import { runPlatformSelfTest } from "../lib/api";
import type { ClientConfig, PlatformSelfTestResult, RealtimeReporterSnapshot } from "../types";

const toast = useToast();
const selfTestLoading = ref(false);
const selfTestResult = ref<PlatformSelfTestResult | null>(null);

const props = defineProps<{
  modelValue: ClientConfig;
  reporterSnapshot: RealtimeReporterSnapshot;
  reporterBusy: boolean;
}>();

const configReady = computed(
  () => !!props.modelValue.baseUrl.trim() && !!props.modelValue.apiToken.trim(),
);

defineEmits<{
  "update:modelValue": [value: ClientConfig];
  imported: [message: string];
  save: [];
  startReporter: [];
  stopReporter: [];
}>();

function formatTime(value?: string | null) {
  if (!value) return "暂无";
  return new Date(value).toLocaleString();
}

async function handleSelfTest() {
  selfTestLoading.value = true;
  const result = await runPlatformSelfTest();
  selfTestLoading.value = false;

  if (!result.success || !result.data) {
    toast.add({
      severity: "error",
      summary: "检查未完成",
      detail: result.error?.message ?? "平台能力检查执行失败。",
      life: 4000,
    });
    return;
  }

  selfTestResult.value = result.data;
  toast.add({
    severity: result.data.foreground.success && result.data.media.success ? "success" : "warn",
    summary: "检查已完成",
    detail: `当前平台：${result.data.platform}`,
    life: 3000,
  });
}
</script>

<template>
  <div class="workspace-grid">
    <header class="hero-panel">
      <div>
        <p class="eyebrow">设置</p>
        <h2>设置</h2>
        <p class="hero-copy">在这里完成连接、设备身份和后台同步设置，让这台客户端稳定接入你的 Waken-Wa。</p>
      </div>
      <div class="hero-actions">
        <Button label="保存设置" icon="pi pi-save" severity="secondary" @click="$emit('save')" />
        <Tag :value="reporterSnapshot.running ? '运行中' : '未启动'" :severity="reporterSnapshot.running ? 'success' : 'warn'" rounded />
      </div>
    </header>

    <ConnectionPanel
      :model-value="modelValue"
      @update:model-value="$emit('update:modelValue', $event)"
      @imported="$emit('imported', $event)"
    />

    <Card class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">后台同步</p>
            <h3>管理后台同步状态，持续更新你当前的活动信息</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div class="overview-summary">
          <div class="overview-item">
            <span>运行状态</span>
            <strong>{{ reporterSnapshot.running ? "运行中" : "未启动" }}</strong>
          </div>
          <div class="overview-item">
            <span>当前进程</span>
            <strong>{{ reporterSnapshot.currentActivity?.processName || "暂无" }}</strong>
          </div>
          <div class="overview-item">
            <span>最近心跳</span>
            <strong>{{ formatTime(reporterSnapshot.lastHeartbeatAt) }}</strong>
          </div>
          <div class="overview-item">
            <span>最近错误</span>
            <strong>{{ reporterSnapshot.lastError || "暂无" }}</strong>
          </div>
        </div>

        <div class="actions-row">
          <Button
            label="开启后台同步"
            icon="pi pi-play"
            :loading="reporterBusy"
            :disabled="reporterSnapshot.running || !configReady"
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
          <Button
            label="检查平台能力"
            icon="pi pi-search"
            severity="secondary"
            text
            :loading="selfTestLoading"
            @click="handleSelfTest"
          />
        </div>

        <div class="message-stack">
          <Message v-if="!configReady" severity="warn" :closable="false">
            开启后台同步前，请先填写好接入配置。
          </Message>
          <Message v-if="reporterSnapshot.lastError" severity="error" :closable="false">
            {{ reporterSnapshot.lastError }}
          </Message>
          <Message v-else severity="secondary" :closable="false">
            如果启用了“启动后自动开启后台同步”，应用下次打开时会自动开始同步。
          </Message>
        </div>
      </template>
    </Card>

    <Card v-if="selfTestResult" class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">平台能力检查</p>
            <h3>检查应用识别、窗口标题和媒体读取能力</h3>
          </div>
          <Tag :value="selfTestResult.platform" severity="contrast" rounded />
        </div>
      </template>
      <template #content>
        <div class="self-test-grid">
          <article class="self-test-card">
            <div class="self-test-head">
              <strong>前台应用</strong>
              <Tag :value="selfTestResult.foreground.success ? '可用' : '异常'" :severity="selfTestResult.foreground.success ? 'success' : 'danger'" rounded />
            </div>
            <p class="self-test-summary">{{ selfTestResult.foreground.summary }}</p>
            <p class="self-test-detail">{{ selfTestResult.foreground.detail }}</p>
            <ul v-if="selfTestResult.foreground.guidance?.length" class="self-test-guidance">
              <li v-for="item in selfTestResult.foreground.guidance" :key="item">{{ item }}</li>
            </ul>
          </article>

          <article class="self-test-card">
            <div class="self-test-head">
              <strong>窗口标题</strong>
              <Tag :value="selfTestResult.windowTitle.success ? '可用' : '异常'" :severity="selfTestResult.windowTitle.success ? 'success' : 'danger'" rounded />
            </div>
            <p class="self-test-summary">{{ selfTestResult.windowTitle.summary }}</p>
            <p class="self-test-detail">{{ selfTestResult.windowTitle.detail }}</p>
            <ul v-if="selfTestResult.windowTitle.guidance?.length" class="self-test-guidance">
              <li v-for="item in selfTestResult.windowTitle.guidance" :key="item">{{ item }}</li>
            </ul>
          </article>

          <article class="self-test-card">
            <div class="self-test-head">
              <strong>媒体采集</strong>
              <Tag :value="selfTestResult.media.success ? '可用' : '异常'" :severity="selfTestResult.media.success ? 'success' : 'danger'" rounded />
            </div>
            <p class="self-test-summary">{{ selfTestResult.media.summary }}</p>
            <p class="self-test-detail">{{ selfTestResult.media.detail }}</p>
            <ul v-if="selfTestResult.media.guidance?.length" class="self-test-guidance">
              <li v-for="item in selfTestResult.media.guidance" :key="item">{{ item }}</li>
            </ul>
          </article>
        </div>

        <div class="message-stack">
          <Message
            v-if="selfTestResult.platform === 'macos'"
            severity="secondary"
            :closable="false"
          >
            macOS 上窗口标题和部分媒体读取能力可能依赖“辅助功能”或“自动化”授权；如果某项异常，请先按上面的提示检查权限。
          </Message>
        </div>
      </template>
    </Card>
  </div>
</template>

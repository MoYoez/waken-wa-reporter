<script setup lang="ts">
import { computed, ref } from "vue";
import Button from "primevue/button";
import Card from "primevue/card";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import Tag from "primevue/tag";
import ToggleSwitch from "primevue/toggleswitch";
import { useToast } from "primevue/usetoast";

import ConnectionPanel from "./ConnectionPanel.vue";
import {
  requestAccessibilityPermission,
  runPlatformSelfTest,
  validateDiscordPresenceConfig,
} from "../lib/api";
import { createNotifier } from "../lib/notify";
import type {
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  PlatformSelfTestResult,
  RealtimeReporterSnapshot,
} from "../types";

const toast = useToast();
const selfTestLoading = ref(false);
const accessibilityPermissionLoading = ref(false);
const selfTestResult = ref<PlatformSelfTestResult | null>(null);

const props = defineProps<{
  modelValue: ClientConfig;
  capabilities: ClientCapabilities;
  reporterSnapshot: RealtimeReporterSnapshot;
  discordPresenceSnapshot: DiscordPresenceSnapshot;
  reporterBusy: boolean;
  discordBusy: boolean;
  verifiedGeneratedHashKey: string;
}>();

const configReady = computed(
  () => !!props.modelValue.baseUrl.trim() && !!props.modelValue.apiToken.trim(),
);
const reporterSupported = computed(() => props.capabilities.realtimeReporter);
const discordSupported = computed(() => props.capabilities.discordPresence);
const selfTestSupported = computed(() => props.capabilities.platformSelfTest);
const isNativeNotice = computed(() => !props.capabilities.realtimeReporter);
const canRequestAccessibilityPermission = computed(() => {
  if (typeof navigator === "undefined") return false;
  return /mac/i.test(navigator.userAgent);
});
const { notify } = createNotifier(toast, () => isNativeNotice.value);
const discordConfigIssues = computed(() =>
  validateDiscordPresenceConfig(props.modelValue, props.capabilities),
);
const discordConfigReady = computed(() => discordConfigIssues.value.length === 0);

const emit = defineEmits<{
  "update:modelValue": [value: ClientConfig];
  imported: [message: string];
  startReporter: [];
  stopReporter: [];
  startDiscordPresence: [];
  stopDiscordPresence: [];
}>();

function updateField<K extends keyof ClientConfig>(key: K, value: ClientConfig[K]) {
  const nextValue = {
    ...props.modelValue,
    [key]: value,
  };
  emit("update:modelValue", nextValue);
}

function formatTime(value?: string | null) {
  if (!value) return "暂无";
  return new Date(value).toLocaleString();
}

function firstGuidance(items?: string[] | null) {
  return items?.find((item) => item.trim()) ?? "";
}

function compactDetail(value?: string | null) {
  const text = (value ?? "").trim();
  if (!text) return "暂无结果。";

  const normalized = text.replace(/\s+/g, " ");
  if (normalized.length <= 88) {
    return normalized;
  }

  const firstChunk = normalized.split("；")[0]?.trim() || normalized;
  if (firstChunk.length <= 88) {
    return `${firstChunk}。`;
  }

  return `${firstChunk.slice(0, 84).trimEnd()}...`;
}

function summarizeProbeDetail(
  platform: string,
  probe: "foreground" | "windowTitle" | "media",
  success: boolean,
  detail?: string | null,
) {
  if (success) {
    return compactDetail(detail);
  }

  const lower = (detail ?? "").toLowerCase();
  if (platform === "linux") {
    if (probe === "foreground" || probe === "windowTitle") {
      if (lower.includes("focused window d-bus") || lower.includes("gdbus")) {
        return "缺少 GNOME 前台窗口支持。";
      }
      if (lower.includes("kdotool")) {
        return "缺少 KDE 前台窗口支持。";
      }
      if (lower.includes("xprop")) {
        return "缺少 xprop。";
      }
      return "未读取到前台窗口信息。";
    }

    if (probe === "media") {
      if (lower.includes("playerctl")) {
        return "缺少 playerctl。";
      }
      return "未读取到媒体信息。";
    }
  }

  return compactDetail(detail);
}

async function handleSelfTest() {
  selfTestLoading.value = true;
  const result = await runPlatformSelfTest();
  selfTestLoading.value = false;

  if (!result.success || !result.data) {
    notify({
      severity: "error",
      summary: "检查未完成",
      detail: result.error?.message ?? "平台能力检查执行失败。",
      life: 4000,
    });
    return;
  }

  selfTestResult.value = result.data;
  notify({
    severity: result.data.foreground.success && result.data.media.success ? "success" : "warn",
    summary: "检查已完成",
    detail: `当前平台：${result.data.platform}`,
    life: 3000,
  });
}

async function handleRequestAccessibilityPermission() {
  accessibilityPermissionLoading.value = true;
  const result = await requestAccessibilityPermission();
  accessibilityPermissionLoading.value = false;

  if (!result.success) {
    notify({
      severity: "error",
      summary: "权限申请未完成",
      detail: result.error?.message ?? "辅助功能权限申请失败。",
      life: 4000,
    });
    return;
  }

  notify({
    severity: result.data ? "success" : "info",
    summary: result.data ? "辅助功能权限已可用" : "已请求辅助功能权限",
    detail: result.data
      ? "系统已允许读取窗口标题。"
      : "系统权限面板应该已经打开，请在“系统设置 -> 隐私与安全性 -> 辅助功能”中允许当前应用。",
    life: 5000,
  });

  await handleSelfTest();
}
</script>

<template>
  <div class="workspace-grid">
    <header class="hero-panel">
      <div>
        <p class="eyebrow">设置</p>
        <h2>设置</h2>
        <p class="hero-copy">管理连接、设备身份和客户端功能。</p>
      </div>
      <div class="hero-actions">
        <Tag
          :value="reporterSupported ? (reporterSnapshot.running ? '运行中' : '未启动') : '移动端模式'"
          :severity="reporterSupported ? (reporterSnapshot.running ? 'success' : 'warn') : 'info'"
          rounded
        />
      </div>
    </header>

    <ConnectionPanel
      :model-value="modelValue"
      :capabilities="capabilities"
      :verified-generated-hash-key="verifiedGeneratedHashKey"
      @update:model-value="$emit('update:modelValue', $event)"
      @imported="$emit('imported', $event)"
    />

    <Card v-if="reporterSupported" class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">后台同步</p>
            <h3>管理同步状态</h3>
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
            v-if="selfTestSupported"
            @click="handleSelfTest"
          />
          <Button
            v-if="canRequestAccessibilityPermission"
            label="授权辅助功能权限"
            icon="pi pi-shield"
            severity="secondary"
            outlined
            :loading="accessibilityPermissionLoading"
            @click="handleRequestAccessibilityPermission"
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

    <Card v-else class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">移动端模式</p>
            <h3>后台同步已关闭</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div class="message-stack">
          <Message severity="secondary" :closable="false">
            当前客户端不提供后台实时同步与系统托盘能力。你仍可手动提交活动并发布灵感内容。
          </Message>
        </div>
      </template>
    </Card>

    <Card v-if="discordSupported" class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">Discord 状态</p>
            <h3>管理 Discord Rich Presence 同步</h3>
          </div>
          <Tag
            :value="discordPresenceSnapshot.running ? (discordPresenceSnapshot.connected ? '运行中' : '等待 Discord') : '未启动'"
            :severity="discordPresenceSnapshot.running ? (discordPresenceSnapshot.connected ? 'success' : 'warn') : 'secondary'"
            rounded
          />
        </div>
      </template>
      <template #content>
        <div class="panel-grid">
          <label class="field-block field-span-2">
            <span class="field-label">Discord Application ID</span>
            <InputText
              :model-value="modelValue.discordApplicationId"
              placeholder="填入 Discord Developer Portal 里的 Application ID"
              @update:model-value="updateField('discordApplicationId', $event ?? '')"
            />
          </label>

          <div class="reporter-enabled-card discord-autostart-card field-span-2">
            <div class="reporter-enabled-copy">
              <span class="field-label">启动后自动开启 Discord 同步</span>
              <strong>{{ modelValue.discordEnabled ? "已开启" : "未开启" }}</strong>
              <span>
                开启后，这台客户端在下次启动时会自动拉取 public feed，并更新 Discord 状态。
              </span>
            </div>
            <ToggleSwitch
              :model-value="modelValue.discordEnabled"
              input-id="settings-discord-enabled"
              @update:model-value="updateField('discordEnabled', Boolean($event))"
            />
          </div>
        </div>

        <div class="overview-summary discord-presence-summary">
          <div class="overview-item">
            <span>运行状态</span>
            <strong>{{ discordPresenceSnapshot.running ? "运行中" : "未启动" }}</strong>
          </div>
          <div class="overview-item">
            <span>Discord 连接</span>
            <strong>{{ discordPresenceSnapshot.connected ? "已连接" : "未连接" }}</strong>
          </div>
          <div class="overview-item">
            <span>当前摘要</span>
            <strong>{{ discordPresenceSnapshot.currentSummary || "暂无" }}</strong>
          </div>
          <div class="overview-item">
            <span>最近同步</span>
            <strong>{{ formatTime(discordPresenceSnapshot.lastSyncAt) }}</strong>
          </div>
        </div>

        <div class="actions-row discord-presence-actions">
          <Button
            label="开启 Discord 同步"
            icon="pi pi-desktop"
            :loading="discordBusy"
            :disabled="discordPresenceSnapshot.running || !discordConfigReady"
            @click="$emit('startDiscordPresence')"
          />
          <Button
            label="停止 Discord 同步"
            icon="pi pi-stop"
            severity="secondary"
            outlined
            :loading="discordBusy"
            :disabled="!discordPresenceSnapshot.running"
            @click="$emit('stopDiscordPresence')"
          />
        </div>

        <div class="message-stack">
          <Message
            v-for="issue in discordConfigIssues"
            :key="issue"
            severity="warn"
            :closable="false"
          >
            {{ issue }}
          </Message>
          <Message v-if="discordPresenceSnapshot.lastError" severity="warn" :closable="false">
            {{ discordPresenceSnapshot.lastError }}
          </Message>
          <Message
            v-else-if="modelValue.discordEnabled"
            severity="secondary"
            :closable="false"
          >
            已启用启动后自动同步，客户端下次打开时会自动尝试连接 Discord。
          </Message>
          <Message v-else severity="secondary" :closable="false">
            Discord 同步只会读取 public feed 中属于当前客户端自己的活动，不会干涉其他机器。
          </Message>
        </div>
      </template>
    </Card>

    <Card v-if="selfTestSupported && selfTestResult" class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">平台能力检查</p>
            <h3>前台应用、窗口标题和媒体读取</h3>
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
            <p class="self-test-detail">
              {{ summarizeProbeDetail(selfTestResult.platform, "foreground", selfTestResult.foreground.success, selfTestResult.foreground.detail) }}
            </p>
            <p v-if="firstGuidance(selfTestResult.foreground.guidance)" class="self-test-summary">
              {{ firstGuidance(selfTestResult.foreground.guidance) }}
            </p>
          </article>

          <article class="self-test-card">
            <div class="self-test-head">
              <strong>窗口标题</strong>
              <Tag :value="selfTestResult.windowTitle.success ? '可用' : '异常'" :severity="selfTestResult.windowTitle.success ? 'success' : 'danger'" rounded />
            </div>
            <p class="self-test-detail">
              {{ summarizeProbeDetail(selfTestResult.platform, "windowTitle", selfTestResult.windowTitle.success, selfTestResult.windowTitle.detail) }}
            </p>
            <p v-if="firstGuidance(selfTestResult.windowTitle.guidance)" class="self-test-summary">
              {{ firstGuidance(selfTestResult.windowTitle.guidance) }}
            </p>
            <div v-if="selfTestResult.platform === 'macos' && !selfTestResult.windowTitle.success" class="actions-row">
              <Button
                label="授权辅助功能权限"
                icon="pi pi-shield"
                severity="secondary"
                outlined
                :loading="accessibilityPermissionLoading"
                @click="handleRequestAccessibilityPermission"
              />
            </div>
          </article>

          <article class="self-test-card">
            <div class="self-test-head">
              <strong>媒体采集</strong>
              <Tag :value="selfTestResult.media.success ? '可用' : '异常'" :severity="selfTestResult.media.success ? 'success' : 'danger'" rounded />
            </div>
            <p class="self-test-detail">
              {{ summarizeProbeDetail(selfTestResult.platform, "media", selfTestResult.media.success, selfTestResult.media.detail) }}
            </p>
            <p v-if="firstGuidance(selfTestResult.media.guidance)" class="self-test-summary">
              {{ firstGuidance(selfTestResult.media.guidance) }}
            </p>
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
          <Message
            v-if="selfTestResult.platform === 'linux'"
            severity="secondary"
            :closable="false"
          >
            Linux：X11 使用 xprop；Wayland 支持 GNOME 的 Focused Window D-Bus 和 KDE 的 kdotool；媒体采集依赖 playerctl。
          </Message>
        </div>
      </template>
    </Card>
  </div>
</template>

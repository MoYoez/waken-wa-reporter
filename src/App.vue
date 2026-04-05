<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, onMounted, ref, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import Tag from "primevue/tag";
import Toast from "primevue/toast";
import { useToast } from "primevue/usetoast";

import ConnectionPanel from "./components/ConnectionPanel.vue";
import OverviewWorkspace from "./components/OverviewWorkspace.vue";
import SettingsWorkspace from "./components/SettingsWorkspace.vue";
import {
  discoverExistingReporterConfig,
  getRealtimeReporterSnapshot,
  startRealtimeReporter,
  stopRealtimeReporter,
} from "./lib/api";
import { defaultClientConfig, loadAppState, saveAppState } from "./lib/persistence";
import type {
  ClientConfig,
  ExistingReporterConfig,
  RealtimeReporterSnapshot,
  RecentPreset,
} from "./types";

const ActivityWorkspace = defineAsyncComponent(() => import("./components/ActivityWorkspace.vue"));
const InspirationWorkspace = defineAsyncComponent(
  () => import("./components/InspirationWorkspace.vue"),
);
const RealtimeWorkspace = defineAsyncComponent(
  () => import("./components/RealtimeWorkspace.vue"),
);

type AppSection = "overview" | "settings" | "activity" | "realtime" | "inspiration";

const toast = useToast();

const config = ref<ClientConfig>(defaultClientConfig());
const recentPresets = ref<RecentPreset[]>([]);
const activeSection = ref<AppSection>("overview");
const hydrated = ref(false);
const onboardingDismissed = ref(false);
const reporterConfigPromptHandled = ref(false);
const reporterBusy = ref(false);
const importingReporterConfig = ref(false);
const existingReporterConfig = ref<ExistingReporterConfig | null>(null);
const reporterSnapshot = ref<RealtimeReporterSnapshot>({
  running: false,
  logs: [],
  currentActivity: null,
  lastHeartbeatAt: null,
  lastError: null,
  lastPendingApprovalMessage: null,
  lastPendingApprovalUrl: null,
});
const pendingApprovalDialogVisible = ref(false);
const lastPendingApprovalSeen = ref("");
const onboardingSetupMode = ref(false);

let reporterPollingTimer: number | undefined;

const sections: Array<{
  key: AppSection;
  title: string;
  kicker: string;
  icon: string;
}> = [
  { key: "overview", title: "概览", kicker: "查看当前连接与同步状态", icon: "pi pi-home" },
  { key: "inspiration", title: "灵感", kicker: "撰写并发布内容", icon: "pi pi-file-edit" },
  { key: "activity", title: "活动同步", kicker: "手动更新当前活动状态", icon: "pi pi-pencil" },
  { key: "realtime", title: "实时同步", kicker: "查看后台同步记录", icon: "pi pi-chart-line" },
  { key: "settings", title: "设置", kicker: "管理连接、设备与同步配置", icon: "pi pi-cog" },
];

const readiness = computed(() => {
  const required = [
    config.value.baseUrl.trim(),
    config.value.apiToken.trim(),
    config.value.generatedHashKey.trim(),
  ];
  return required.every(Boolean);
});

const shouldShowOnboarding = computed(
  () => hydrated.value && !onboardingDismissed.value && !readiness.value,
);

function handlePresetSaved(preset: RecentPreset) {
  const deduped = recentPresets.value.filter(
    (item) =>
      item.process_name !== preset.process_name ||
      item.process_title !== preset.process_title,
  );
  recentPresets.value = [preset, ...deduped].slice(0, 6);
}

function notifyImport(message: string) {
  toast.add({
    severity: "success",
    summary: "已导入接入配置",
    detail: message,
    life: 3000,
  });
}

function closeOnboarding() {
  onboardingSetupMode.value = false;
  onboardingDismissed.value = true;
}

function startSetup() {
  reporterConfigPromptHandled.value = true;
  onboardingSetupMode.value = true;
}

function skipExistingReporterConfig() {
  reporterConfigPromptHandled.value = true;
}

async function useExistingReporterConfig() {
  if (!existingReporterConfig.value?.config) return;
  importingReporterConfig.value = true;
  config.value = { ...existingReporterConfig.value.config };
  reporterConfigPromptHandled.value = true;
  importingReporterConfig.value = false;
  toast.add({
    severity: "success",
    summary: "已导入现有配置",
    detail: "连接信息和后台同步参数已同步到当前客户端。",
    life: 3500,
  });
  onboardingSetupMode.value = true;
}

async function refreshReporterSnapshot() {
  const result = await getRealtimeReporterSnapshot();
  if (!result.success || !result.data) {
    return;
  }
  reporterSnapshot.value = result.data;
}

function closePendingApprovalDialog() {
  pendingApprovalDialogVisible.value = false;
}

async function handleStartReporter() {
  reporterBusy.value = true;
  const result = await startRealtimeReporter(config.value);
  reporterBusy.value = false;

  if (!result.success || !result.data) {
    toast.add({
      severity: "error",
      summary: "启动失败",
      detail: result.error?.message ?? "后台同步启动失败。",
      life: 4000,
    });
    return;
  }

  reporterSnapshot.value = result.data;
  toast.add({
    severity: "success",
    summary: "后台同步已开启",
    detail: "客户端已开始持续同步当前活动状态。",
    life: 3000,
  });
}

async function handleStopReporter() {
  reporterBusy.value = true;
  const result = await stopRealtimeReporter();
  reporterBusy.value = false;

  if (!result.success || !result.data) {
    toast.add({
      severity: "error",
      summary: "停止失败",
      detail: result.error?.message ?? "后台同步停止失败。",
      life: 4000,
    });
    return;
  }

  reporterSnapshot.value = result.data;
  toast.add({
    severity: "success",
    summary: "后台同步已停止",
    detail: "客户端已停止自动同步当前状态。",
    life: 3000,
  });
}

watch(
  [config, recentPresets, onboardingDismissed, reporterConfigPromptHandled],
  async ([nextConfig, nextPresets, nextDismissed, nextReporterPromptHandled]) => {
    if (!hydrated.value) return;
    await saveAppState(nextConfig, nextPresets, nextDismissed, nextReporterPromptHandled);
  },
  { deep: true },
);

watch(
  () => [
    reporterSnapshot.value.lastPendingApprovalMessage,
    reporterSnapshot.value.lastPendingApprovalUrl,
  ],
  ([message, url]) => {
    const nextKey = `${message ?? ""}|${url ?? ""}`;
    if (!message || nextKey === "|" || nextKey === lastPendingApprovalSeen.value) {
      return;
    }
    lastPendingApprovalSeen.value = nextKey;
    pendingApprovalDialogVisible.value = true;
  },
  { immediate: true },
);

onMounted(async () => {
  const state = await loadAppState();
  config.value = state.config;
  recentPresets.value = state.recentPresets;
  onboardingDismissed.value = state.onboardingDismissed;
  reporterConfigPromptHandled.value = state.reporterConfigPromptHandled ?? false;
  hydrated.value = true;

  if (!reporterConfigPromptHandled.value) {
    const reporterConfigResult = await discoverExistingReporterConfig();
    if (reporterConfigResult.success && reporterConfigResult.data?.found) {
      existingReporterConfig.value = reporterConfigResult.data;
    }
  }

  await refreshReporterSnapshot();
  reporterPollingTimer = window.setInterval(() => {
    void refreshReporterSnapshot();
  }, 2000);

  if (config.value.reporterEnabled && !reporterSnapshot.value.running && readiness.value) {
    void handleStartReporter();
  }
});

onBeforeUnmount(() => {
  if (reporterPollingTimer) {
    window.clearInterval(reporterPollingTimer);
  }
});
</script>

<template>
  <Toast position="top-right" />
  <Dialog
    :visible="shouldShowOnboarding"
    modal
    dismissable-mask
    :draggable="false"
    :closable="false"
    class="onboarding-dialog"
  >
    <template #container>
      <div class="onboarding-panel">
        <template v-if="!onboardingSetupMode">
          <p class="eyebrow">首次引导</p>
          <h3>先完成连接设置，再开始使用桌面客户端</h3>
          <p class="onboarding-copy">
            首次使用时，你可以直接导入本机已有配置，或手动完成连接设置。准备完成后，就可以开始同步活动状态，并在后台持续更新。
          </p>
          <div v-if="existingReporterConfig?.found && !reporterConfigPromptHandled" class="onboarding-step onboarding-highlight">
            <strong>发现可用的本机配置</strong>
            <span>已在本机找到 `waken-wa-reporter` 配置文件，可直接导入站点地址、Token、设备标识和同步参数。</span>
            <small v-if="existingReporterConfig.path">{{ existingReporterConfig.path }}</small>
            <div class="actions-row">
              <Button
                label="使用现有配置"
                icon="pi pi-download"
                :loading="importingReporterConfig"
                @click="useExistingReporterConfig"
              />
              <Button
                label="暂不导入"
                severity="secondary"
                text
                @click="skipExistingReporterConfig"
              />
            </div>
          </div>
          <div class="onboarding-steps">
            <div class="onboarding-step">
              <strong>1. 完成连接</strong>
              <span>在引导中完成站点地址、Token 与设备名称设置。</span>
            </div>
            <div class="onboarding-step">
              <strong>2. 检查采集能力</strong>
              <span>完成配置后，再到设置页检查前台应用、窗口标题和媒体读取能力。</span>
            </div>
            <div class="onboarding-step">
              <strong>3. 开始同步</strong>
              <span>配置完成后，你可以手动更新状态，也可以开启后台同步持续更新当前状态。</span>
            </div>
          </div>
          <div class="actions-row">
            <Button label="前往设置" icon="pi pi-arrow-right" @click="startSetup" />
            <Button label="稍后再说" severity="secondary" text @click="closeOnboarding" />
          </div>
        </template>

        <template v-else>
          <p class="eyebrow">首次引导</p>
          <h3>在这里完成连接设置</h3>
          <p class="onboarding-copy">
            填好接入配置后，就可以开始使用桌面客户端；你也可以先导入现有配置，再按需要微调。
          </p>
          <ConnectionPanel
            :model-value="config"
            @update:model-value="config = $event"
            @imported="notifyImport"
          />
          <div class="actions-row">
            <Button
              label="完成并开始使用"
              icon="pi pi-check"
              :disabled="!readiness"
              @click="closeOnboarding"
            />
            <Button
              label="返回上一步"
              severity="secondary"
              text
              @click="onboardingSetupMode = false"
            />
          </div>
        </template>
      </div>
    </template>
  </Dialog>

  <Dialog
    v-model:visible="pendingApprovalDialogVisible"
    modal
    dismissable-mask
    :draggable="false"
    header="设备待审核"
    style="width: min(560px, calc(100vw - 24px))"
  >
    <div class="onboarding-steps">
      <div class="onboarding-step">
        <strong>{{ reporterSnapshot.lastPendingApprovalMessage || "设备待后台审核后可用" }}</strong>
        <span>后台同步已经识别到当前设备尚未通过审核，请前往 Waken-Wa 后台的设备管理完成审核。</span>
      </div>
      <div
        v-if="reporterSnapshot.lastPendingApprovalUrl"
        class="onboarding-step onboarding-highlight"
      >
        <strong>审核地址</strong>
        <span>{{ reporterSnapshot.lastPendingApprovalUrl }}</span>
      </div>
    </div>
    <div class="actions-row">
      <Button label="我知道了" icon="pi pi-check" @click="closePendingApprovalDialog" />
    </div>
  </Dialog>

  <div class="app-shell">
    <aside class="app-sidebar">
      <div class="brand-block">
        <p class="eyebrow">Waken-Wa</p>
        <h1>桌面客户端</h1>
      </div>

      <nav class="nav-stack">
        <button
          v-for="section in sections"
          :key="section.key"
          class="nav-item"
          :class="{ active: section.key === activeSection }"
          type="button"
          @click="activeSection = section.key"
        >
          <i :class="section.icon" />
          <div>
            <strong>{{ section.title }}</strong>
            <span>{{ section.kicker }}</span>
          </div>
        </button>
      </nav>

      <div class="sidebar-footer">
        <Tag :value="readiness ? '连接配置已就绪' : '需要先完成连接设置'" :severity="readiness ? 'success' : 'warn'" rounded />
        <Tag :value="reporterSnapshot.running ? '后台同步运行中' : '后台同步未开启'" :severity="reporterSnapshot.running ? 'success' : 'secondary'" rounded />
        <small>关闭主窗口后会最小化到系统托盘，可在后台继续驻留。</small>
      </div>
    </aside>

    <main class="app-main">
      <OverviewWorkspace
        v-if="activeSection === 'overview'"
        :config="config"
        :readiness="readiness"
        :reporter-snapshot="reporterSnapshot"
      />

      <SettingsWorkspace
        v-else-if="activeSection === 'settings'"
        :model-value="config"
        :reporter-snapshot="reporterSnapshot"
        :reporter-busy="reporterBusy"
        @update:model-value="config = $event"
        @imported="notifyImport"
        @start-reporter="handleStartReporter"
        @stop-reporter="handleStopReporter"
      />

      <ActivityWorkspace
        v-else-if="activeSection === 'activity'"
        :config="config"
        :recent-presets="recentPresets"
        @preset-saved="handlePresetSaved"
      />

      <RealtimeWorkspace
        v-else-if="activeSection === 'realtime'"
        :snapshot="reporterSnapshot"
      />

      <InspirationWorkspace v-else :config="config" />
    </main>
  </div>
</template>

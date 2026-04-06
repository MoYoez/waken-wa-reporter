<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import Tag from "primevue/tag";
import Toast from "primevue/toast";
import { useToast } from "primevue/usetoast";

import ActivityWorkspace from "./components/ActivityWorkspace.vue";
import ConnectionPanel from "./components/ConnectionPanel.vue";
import InspirationWorkspace from "./components/InspirationWorkspace.vue";
import OverviewWorkspace from "./components/OverviewWorkspace.vue";
import RealtimeWorkspace from "./components/RealtimeWorkspace.vue";
import SettingsWorkspace from "./components/SettingsWorkspace.vue";
import {
  discoverExistingReporterConfig,
  getClientCapabilities,
  getRealtimeReporterSnapshot,
  startRealtimeReporter,
  stopRealtimeReporter,
} from "./lib/api";
import { createNotifier } from "./lib/notify";
import { defaultClientConfig, loadAppState, saveAppState } from "./lib/persistence";
import type {
  ClientCapabilities,
  ClientConfig,
  DeviceType,
  ExistingReporterConfig,
  RealtimeReporterSnapshot,
  RecentPreset,
} from "./types";

type AppSection = "overview" | "settings" | "activity" | "realtime" | "inspiration";

interface SectionNavItem {
  key: AppSection;
  title: string;
  kicker: string;
  icon: string;
  requiresRealtime?: boolean;
}

const defaultCapabilities: ClientCapabilities = {
  realtimeReporter: true,
  tray: true,
  platformSelfTest: true,
};

const toast = useToast();

const capabilities = ref<ClientCapabilities>(defaultCapabilities);
const config = ref<ClientConfig>(defaultClientConfig());
const onboardingDraftConfig = ref<ClientConfig>(defaultClientConfig());
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
const viewportWidth = ref<number>(1200);

let reporterPollingTimer: number | undefined;

const sections: SectionNavItem[] = [
  { key: "overview", title: "概览", kicker: "查看当前连接与同步状态", icon: "pi pi-home" },
  { key: "inspiration", title: "灵感", kicker: "撰写并发布内容", icon: "pi pi-file-edit" },
  { key: "activity", title: "活动同步", kicker: "手动更新当前活动状态", icon: "pi pi-pencil" },
  {
    key: "realtime",
    title: "实时同步",
    kicker: "查看后台同步记录",
    icon: "pi pi-chart-line",
    requiresRealtime: true,
  },
  { key: "settings", title: "设置", kicker: "管理连接与设备配置", icon: "pi pi-cog" },
];

const reporterSupported = computed(() => capabilities.value.realtimeReporter);
const traySupported = computed(() => capabilities.value.tray);
const isPhone = computed(() => viewportWidth.value < 900);
const isNativeNotice = computed(() => !reporterSupported.value);
const { notify } = createNotifier(toast, () => isNativeNotice.value);

const visibleSections = computed(() =>
  sections.filter((section) => !section.requiresRealtime || reporterSupported.value),
);

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

function ensureVisibleSection() {
  if (!visibleSections.value.some((section) => section.key === activeSection.value)) {
    activeSection.value = visibleSections.value[0]?.key ?? "overview";
  }
}

function inferMobileDeviceType(): DeviceType {
  return isPhone.value ? "mobile" : "tablet";
}

function normalizeConfigByCapabilities(raw: ClientConfig): ClientConfig {
  const normalizedDevice = raw.device.trim() || defaultClientConfig().device;

  if (reporterSupported.value) {
    return { ...raw, device: normalizedDevice, deviceType: "desktop" };
  }

  return {
    ...raw,
    device: normalizedDevice,
    deviceType: inferMobileDeviceType(),
    reporterEnabled: false,
  };
}

function syncDeviceTypeByViewport() {
  const nextType = reporterSupported.value ? "desktop" : inferMobileDeviceType();

  if (config.value.deviceType !== nextType) {
    config.value = { ...config.value, deviceType: nextType };
  }

  if (onboardingDraftConfig.value.deviceType !== nextType) {
    onboardingDraftConfig.value = { ...onboardingDraftConfig.value, deviceType: nextType };
  }
}

function onViewportResize() {
  viewportWidth.value = window.innerWidth;
  syncDeviceTypeByViewport();
}

function handlePresetSaved(preset: RecentPreset) {
  const deduped = recentPresets.value.filter(
    (item) =>
      item.process_name !== preset.process_name ||
      item.process_title !== preset.process_title,
  );
  recentPresets.value = [preset, ...deduped].slice(0, 6);
  void persistAppState();
}

function notifyImport(message: string) {
  notify({
    severity: "success",
    summary: "已导入接入配置",
    detail: message,
    life: 3000,
  });
}

function closeOnboarding() {
  onboardingSetupMode.value = false;
  onboardingDismissed.value = true;
  void persistAppState();
}

function startSetup() {
  reporterConfigPromptHandled.value = true;
  onboardingDraftConfig.value = { ...config.value };
  onboardingSetupMode.value = true;
}

function skipExistingReporterConfig() {
  reporterConfigPromptHandled.value = true;
  void persistAppState();
}

async function useExistingReporterConfig() {
  if (!existingReporterConfig.value?.config) return;
  importingReporterConfig.value = true;
  onboardingDraftConfig.value = normalizeConfigByCapabilities({
    ...existingReporterConfig.value.config,
  });
  reporterConfigPromptHandled.value = true;
  importingReporterConfig.value = false;
  notify({
    severity: "success",
    summary: "已导入现有配置",
    detail: "连接信息和同步参数已同步到当前客户端。",
    life: 3500,
  });
  onboardingSetupMode.value = true;
}

async function persistAppState(configOverride?: ClientConfig) {
  await saveAppState(
    normalizeConfigByCapabilities(configOverride ?? config.value),
    recentPresets.value,
    onboardingDismissed.value,
    reporterConfigPromptHandled.value,
  );
}

async function handleSaveSettings() {
  config.value = normalizeConfigByCapabilities(config.value);
  await persistAppState();
  notify({
    severity: "success",
    summary: "设置已保存",
    detail: "当前配置已写入本地。",
    life: 2500,
  });
}

async function completeOnboardingSetup() {
  config.value = normalizeConfigByCapabilities({ ...onboardingDraftConfig.value });
  onboardingDismissed.value = true;
  onboardingSetupMode.value = false;
  await persistAppState(config.value);
  notify({
    severity: "success",
    summary: "设置已完成",
    detail: "欢迎开始使用客户端。",
    life: 2500,
  });
}

async function refreshReporterSnapshot() {
  if (!reporterSupported.value || activeSection.value === "inspiration") {
    return;
  }

  const result = await getRealtimeReporterSnapshot();
  if (!result.success || !result.data) {
    return;
  }
  Object.assign(reporterSnapshot.value, result.data);
}

function closePendingApprovalDialog() {
  pendingApprovalDialogVisible.value = false;
}

async function handleStartReporter() {
  if (!reporterSupported.value) {
    return;
  }

  reporterBusy.value = true;
  const result = await startRealtimeReporter(config.value);
  reporterBusy.value = false;

  if (!result.success || !result.data) {
    notify({
      severity: "error",
      summary: "启动失败",
      detail: result.error?.message ?? "后台同步启动失败。",
      life: 4000,
    });
    return;
  }

  Object.assign(reporterSnapshot.value, result.data);
  notify({
    severity: "success",
    summary: "后台同步已开启",
    detail: "客户端已开始持续同步当前活动状态。",
    life: 3000,
  });
}

async function handleStopReporter() {
  if (!reporterSupported.value) {
    return;
  }

  reporterBusy.value = true;
  const result = await stopRealtimeReporter();
  reporterBusy.value = false;

  if (!result.success || !result.data) {
    notify({
      severity: "error",
      summary: "停止失败",
      detail: result.error?.message ?? "后台同步停止失败。",
      life: 4000,
    });
    return;
  }

  Object.assign(reporterSnapshot.value, result.data);
  notify({
    severity: "success",
    summary: "后台同步已停止",
    detail: "客户端已停止自动同步当前状态。",
    life: 3000,
  });
}

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

watch(
  () => activeSection.value,
  (section) => {
    if (reporterSupported.value && section !== "inspiration") {
      void refreshReporterSnapshot();
    }
  },
);

watch(
  () => visibleSections.value.map((item) => item.key).join(","),
  () => {
    ensureVisibleSection();
  },
);

watch(
  () => capabilities.value,
  () => {
    syncDeviceTypeByViewport();
  },
  { deep: true },
);

onMounted(async () => {
  viewportWidth.value = window.innerWidth;
  window.addEventListener("resize", onViewportResize);

  const capabilitiesResult = await getClientCapabilities();
  if (capabilitiesResult.success && capabilitiesResult.data) {
    capabilities.value = capabilitiesResult.data;
  }

  const state = await loadAppState();
  const normalized = normalizeConfigByCapabilities(state.config);
  config.value = normalized;
  onboardingDraftConfig.value = { ...normalized };
  recentPresets.value = state.recentPresets;
  onboardingDismissed.value = state.onboardingDismissed;
  reporterConfigPromptHandled.value = reporterSupported.value
    ? (state.reporterConfigPromptHandled ?? false)
    : true;
  hydrated.value = true;

  ensureVisibleSection();
  syncDeviceTypeByViewport();

  if (reporterSupported.value && !reporterConfigPromptHandled.value) {
    const reporterConfigResult = await discoverExistingReporterConfig();
    if (reporterConfigResult.success && reporterConfigResult.data?.found) {
      existingReporterConfig.value = reporterConfigResult.data;
    }
  }

  if (!reporterSupported.value) {
    return;
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
  window.removeEventListener("resize", onViewportResize);
  if (reporterPollingTimer) {
    window.clearInterval(reporterPollingTimer);
  }
});
</script>

<template>
  <div class="app-root">
    <Toast v-if="!isNativeNotice" position="top-right" />
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
            <h3>先完成连接设置，再开始使用客户端</h3>
            <p class="onboarding-copy">
              首次使用时，你可以导入本机已有配置，或手动完成连接设置。准备完成后，就可以开始使用手动活动同步与灵感发布。
            </p>
            <div
              v-if="reporterSupported && existingReporterConfig?.found && !reporterConfigPromptHandled"
              class="onboarding-step onboarding-highlight"
            >
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
                <strong>2. 活动同步</strong>
                <span>通过“活动同步”页面手动提交当前活动状态。</span>
              </div>
              <div class="onboarding-step">
                <strong>3. 内容发布</strong>
                <span>使用“灵感”页面发布内容并附带状态快照。</span>
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
              填好接入配置后，就可以开始使用客户端；你也可以先导入现有配置，再按需要微调。
            </p>
            <ConnectionPanel
              :model-value="onboardingDraftConfig"
              :capabilities="capabilities"
              @update:model-value="onboardingDraftConfig = $event"
              @imported="notifyImport"
            />
            <div class="actions-row">
              <Button
                label="完成并开始使用"
                icon="pi pi-check"
                :disabled="!(
                  onboardingDraftConfig.baseUrl.trim() &&
                  onboardingDraftConfig.apiToken.trim() &&
                  onboardingDraftConfig.generatedHashKey.trim()
                )"
                @click="completeOnboardingSetup"
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
      v-if="reporterSupported"
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

    <div class="app-shell" :class="{ 'phone-nav': isPhone }">
      <aside v-if="!isPhone" class="app-sidebar">
        <div class="brand-block">
          <p class="eyebrow">Waken-Wa</p>
          <h1>客户端</h1>
        </div>

        <nav class="nav-stack">
          <button
            v-for="section in visibleSections"
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
          <Tag
            v-if="reporterSupported"
            :value="reporterSnapshot.running ? '后台同步运行中' : '后台同步未开启'"
            :severity="reporterSnapshot.running ? 'success' : 'secondary'"
            rounded
          />
          <small v-if="traySupported">关闭主窗口后会最小化到系统托盘，可在后台继续驻留。</small>
          <small v-else>当前平台使用移动端模式，已关闭后台实时同步与托盘能力。</small>
        </div>
      </aside>

      <main class="app-main">
        <OverviewWorkspace
          v-if="activeSection === 'overview'"
          :config="config"
          :readiness="readiness"
          :capabilities="capabilities"
          :reporter-snapshot="reporterSnapshot"
        />

        <SettingsWorkspace
          v-else-if="activeSection === 'settings'"
          :model-value="config"
          :capabilities="capabilities"
          :reporter-snapshot="reporterSnapshot"
          :reporter-busy="reporterBusy"
          @update:model-value="config = normalizeConfigByCapabilities($event)"
          @imported="notifyImport"
          @save="handleSaveSettings"
          @start-reporter="handleStartReporter"
          @stop-reporter="handleStopReporter"
        />

        <ActivityWorkspace
          v-else-if="activeSection === 'activity'"
          :config="config"
          :capabilities="capabilities"
          :recent-presets="recentPresets"
          @preset-saved="handlePresetSaved"
        />

        <RealtimeWorkspace
          v-else-if="activeSection === 'realtime'"
          :snapshot="reporterSnapshot"
        />

        <InspirationWorkspace
          v-else-if="activeSection === 'inspiration'"
          :config="config"
          :capabilities="capabilities"
        />
      </main>
    </div>

    <nav v-if="isPhone" class="mobile-tabbar">
      <button
        v-for="section in visibleSections"
        :key="section.key"
        class="mobile-tab-item"
        :class="{ active: section.key === activeSection }"
        type="button"
        @click="activeSection = section.key"
      >
        <i :class="section.icon" />
        <span>{{ section.title }}</span>
      </button>
    </nav>
  </div>
</template>

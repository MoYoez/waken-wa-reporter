<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import Button from "primevue/button";
import Card from "primevue/card";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import Select from "primevue/select";
import Tag from "primevue/tag";
import ToggleSwitch from "primevue/toggleswitch";
import { useToast } from "primevue/usetoast";

import ConnectionPanel from "./ConnectionPanel.vue";
import {
  localeOptions,
  type SupportedLocale,
} from "../i18n";
import {
  requestAccessibilityPermission,
  runPlatformSelfTest,
  validateDiscordPresenceConfig,
} from "../lib/api";
import {
  resolveApiErrorMessage,
  resolveLocalizedEntry,
  resolveLocalizedText,
} from "../lib/localizedText";
import { createNotifier } from "../lib/notify";
import type {
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  PlatformProbeResult,
  PlatformSelfTestResult,
  RealtimeReporterSnapshot,
} from "../types";

const { t, locale } = useI18n();

const toast = useToast();
const selfTestLoading = ref(false);
const accessibilityPermissionLoading = ref(false);
const selfTestResult = ref<PlatformSelfTestResult | null>(null);

const props = defineProps<{
  modelValue: ClientConfig;
  locale: SupportedLocale;
  capabilities: ClientCapabilities;
  reporterSnapshot: RealtimeReporterSnapshot;
  discordPresenceSnapshot: DiscordPresenceSnapshot;
  reporterBusy: boolean;
  discordBusy: boolean;
  verifiedGeneratedHashKey: string;
  localeRestartRequired: boolean;
  restarting: boolean;
}>();

const configReady = computed(
  () => !!props.modelValue.baseUrl.trim() && !!props.modelValue.apiToken.trim(),
);
const reporterSupported = computed(() => props.capabilities.realtimeReporter);
const discordSupported = computed(() => props.capabilities.discordPresence);
const selfTestSupported = computed(() => props.capabilities.platformSelfTest);
const autostartSupported = computed(() => props.capabilities.autostart);
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
  "update:locale": [value: SupportedLocale];
  imported: [message: string];
  startReporter: [];
  stopReporter: [];
  startDiscordPresence: [];
  stopDiscordPresence: [];
  restartApp: [];
}>();

function updateField<K extends keyof ClientConfig>(key: K, value: ClientConfig[K]) {
  const nextValue = {
    ...props.modelValue,
    [key]: value,
  };
  emit("update:modelValue", nextValue);
}

function formatTime(value?: string | null) {
  if (!value) return t("settings.notify.none");

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString(locale.value);
}

function translateText(key: string, params?: Record<string, unknown>) {
  return params ? t(key, params) : t(key);
}

function apiErrorDetail(
  error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
  fallback: string,
) {
  return resolveApiErrorMessage(error, translateText, fallback);
}

function firstGuidance(probe?: PlatformProbeResult | null) {
  const localized = probe?.guidanceEntries
    ?.map((entry) => resolveLocalizedEntry(entry, translateText))
    .find((item) => item.trim());
  if (localized) {
    return localized;
  }

  return probe?.guidance?.find((item) => item.trim()) ?? "";
}

function probeSummary(probe: PlatformProbeResult) {
  return resolveLocalizedText(
    translateText,
    probe.summaryKey,
    probe.summaryParams,
    probe.summary,
  );
}

function probeDetail(probe: PlatformProbeResult) {
  return resolveLocalizedText(
    translateText,
    probe.detailKey,
    probe.detailParams,
    probe.detail,
  );
}

function compactDetail(value?: string | null) {
  const text = (value ?? "").trim();
  if (!text) return t("settings.notify.noneResult");

  const normalized = text.replace(/\s+/g, " ");
  if (normalized.length <= 88) {
    return normalized;
  }

  const firstChunk = normalized.split(/[；;]/)[0]?.trim() || normalized;
  if (firstChunk.length <= 88) {
    return firstChunk;
  }

  return `${firstChunk.slice(0, 84).trimEnd()}...`;
}

function primaryProbeText(probe: PlatformProbeResult) {
  return probe.success ? compactDetail(probeDetail(probe)) : probeSummary(probe);
}

function secondaryProbeText(probe: PlatformProbeResult) {
  if (probe.success) {
    return "";
  }

  return firstGuidance(probe) || probeDetail(probe);
}

async function handleSelfTest() {
  selfTestLoading.value = true;
  const result = await runPlatformSelfTest();
  selfTestLoading.value = false;

  if (!result.success || !result.data) {
    notify({
      severity: "error",
      summary: t("settings.notify.selfTestFailed"),
      detail: apiErrorDetail(result.error, t("settings.notify.selfTestFailedDetail")),
      life: 4000,
    });
    return;
  }

  selfTestResult.value = result.data;
  notify({
    severity: result.data.foreground.success && result.data.media.success ? "success" : "warn",
    summary: t("settings.notify.selfTestDone"),
    detail: t("settings.selfTest.platformDetail", { platform: result.data.platform }),
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
      summary: t("settings.notify.permissionFailed"),
      detail: apiErrorDetail(result.error, t("settings.notify.permissionFailedDetail")),
      life: 4000,
    });
    return;
  }

  notify({
    severity: result.data ? "success" : "info",
    summary: result.data
      ? t("settings.notify.permissionGranted")
      : t("settings.notify.permissionRequested"),
    detail: result.data
      ? t("settings.notify.permissionGrantedDetail")
      : t("settings.notify.permissionRequestedDetail"),
    life: 5000,
  });

  await handleSelfTest();
}

async function handleRestartApp() {
  emit("restartApp");
}
</script>

<template>
  <div class="workspace-grid">
    <header class="hero-panel">
      <div>
        <p class="eyebrow">{{ t("settings.hero.eyebrow") }}</p>
        <h2>{{ t("settings.hero.title") }}</h2>
        <p class="hero-copy">{{ t("settings.hero.description") }}</p>
      </div>
      <div class="hero-actions">
        <Tag
          :value="reporterSupported ? (reporterSnapshot.running ? t('settings.tags.running') : t('settings.tags.notStarted')) : t('settings.tags.mobileMode')"
          :severity="reporterSupported ? (reporterSnapshot.running ? 'success' : 'warn') : 'info'"
          rounded
        />
      </div>
    </header>

    <Card class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("settings.language.eyebrow") }}</p>
            <h3>{{ t("settings.language.title") }}</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div class="panel-grid">
          <label class="field-block field-span-2">
            <span class="field-label">{{ t("settings.language.field") }}</span>
            <Select
              :model-value="props.locale"
              :options="localeOptions"
              option-label="label"
              option-value="value"
              @update:model-value="(value) => value && emit('update:locale', value)"
            />
          </label>
        </div>
        <div class="message-stack">
          <Message severity="secondary" :closable="false">
            {{ t("settings.language.description") }}
          </Message>
          <Message
            v-if="localeRestartRequired"
            severity="warn"
            :closable="false"
          >
            <div class="inline-message-action">
              <span>{{ t("settings.language.restartHint") }}</span>
              <Button
                :label="t('settings.language.restartNow')"
                icon="pi pi-refresh"
                size="small"
                :loading="restarting"
                @click="handleRestartApp"
              />
            </div>
          </Message>
        </div>
      </template>
    </Card>

    <Card v-if="autostartSupported" class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("settings.startup.eyebrow") }}</p>
            <h3>{{ t("settings.startup.title") }}</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div class="panel-grid">
          <div class="reporter-enabled-card field-span-2">
            <div class="reporter-enabled-copy">
              <span class="field-label">{{ t("settings.startup.toggle") }}</span>
              <strong>{{ modelValue.launchOnStartup ? t("settings.tags.enabled") : t("settings.tags.disabled") }}</strong>
              <span>
                {{ t("settings.startup.description") }}
              </span>
            </div>
            <ToggleSwitch
              :model-value="modelValue.launchOnStartup"
              input-id="settings-launch-on-startup"
              @update:model-value="updateField('launchOnStartup', Boolean($event))"
            />
          </div>
        </div>
        <div class="message-stack">
          <Message severity="secondary" :closable="false">
            {{ t("settings.startup.hint") }}
          </Message>
        </div>
      </template>
    </Card>

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
            <p class="eyebrow">{{ t("settings.reporter.eyebrow") }}</p>
            <h3>{{ t("settings.reporter.title") }}</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div class="overview-summary">
          <div class="overview-item">
            <span>{{ t("settings.reporter.runtimeStatus") }}</span>
            <strong>{{ reporterSnapshot.running ? t("settings.tags.running") : t("settings.tags.notStarted") }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("settings.reporter.currentProcess") }}</span>
            <strong>{{ reporterSnapshot.currentActivity?.processName || t("settings.notify.none") }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("settings.reporter.lastHeartbeat") }}</span>
            <strong>{{ formatTime(reporterSnapshot.lastHeartbeatAt) }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("settings.reporter.lastError") }}</span>
            <strong>{{ reporterSnapshot.lastError || t("settings.notify.none") }}</strong>
          </div>
        </div>

        <div class="actions-row">
          <Button
            :label="t('settings.reporter.start')"
            icon="pi pi-play"
            :loading="reporterBusy"
            :disabled="reporterSnapshot.running || !configReady"
            @click="$emit('startReporter')"
          />
          <Button
            :label="t('settings.reporter.stop')"
            icon="pi pi-stop"
            severity="secondary"
            outlined
            :loading="reporterBusy"
            :disabled="!reporterSnapshot.running"
            @click="$emit('stopReporter')"
          />
          <Button
            v-if="selfTestSupported"
            :label="t('settings.reporter.selfTest')"
            icon="pi pi-search"
            severity="secondary"
            text
            :loading="selfTestLoading"
            @click="handleSelfTest"
          />
          <Button
            v-if="canRequestAccessibilityPermission"
            :label="t('settings.reporter.accessibility')"
            icon="pi pi-shield"
            severity="secondary"
            outlined
            :loading="accessibilityPermissionLoading"
            @click="handleRequestAccessibilityPermission"
          />
        </div>

        <div class="message-stack">
          <Message v-if="!configReady" severity="warn" :closable="false">
            {{ t("settings.reporter.configRequired") }}
          </Message>
          <Message v-if="reporterSnapshot.lastError" severity="error" :closable="false">
            {{ reporterSnapshot.lastError }}
          </Message>
          <Message v-else severity="secondary" :closable="false">
            {{ t("settings.reporter.autoStartHint") }}
          </Message>
        </div>
      </template>
    </Card>

    <Card v-else class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("settings.mobile.eyebrow") }}</p>
            <h3>{{ t("settings.mobile.title") }}</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div class="message-stack">
          <Message severity="secondary" :closable="false">
            {{ t("settings.mobile.description") }}
          </Message>
        </div>
      </template>
    </Card>

    <Card v-if="discordSupported" class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("settings.discord.eyebrow") }}</p>
            <h3>{{ t("settings.discord.title") }}</h3>
          </div>
          <Tag
            :value="discordPresenceSnapshot.running ? (discordPresenceSnapshot.connected ? t('settings.tags.running') : t('settings.tags.waitingDiscord')) : t('settings.tags.notStarted')"
            :severity="discordPresenceSnapshot.running ? (discordPresenceSnapshot.connected ? 'success' : 'warn') : 'secondary'"
            rounded
          />
        </div>
      </template>
      <template #content>
        <div class="panel-grid">
          <label class="field-block field-span-2">
            <span class="field-label">{{ t("settings.discord.appId") }}</span>
            <InputText
              :model-value="modelValue.discordApplicationId"
              :placeholder="t('settings.discord.appIdPlaceholder')"
              @update:model-value="updateField('discordApplicationId', $event ?? '')"
            />
          </label>

          <div class="reporter-enabled-card discord-autostart-card field-span-2">
            <div class="reporter-enabled-copy">
              <span class="field-label">{{ t("settings.discord.autoStart") }}</span>
              <strong>{{ modelValue.discordEnabled ? t("settings.tags.enabled") : t("settings.tags.disabled") }}</strong>
              <span>
                {{ t("settings.discord.autoStartDetail") }}
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
            <span>{{ t("settings.reporter.runtimeStatus") }}</span>
            <strong>{{ discordPresenceSnapshot.running ? t("settings.tags.running") : t("settings.tags.notStarted") }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("settings.discord.connection") }}</span>
            <strong>{{ discordPresenceSnapshot.connected ? t("settings.tags.connected") : t("settings.tags.notConnected") }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("settings.discord.currentSummary") }}</span>
            <strong>{{ discordPresenceSnapshot.currentSummary || t("settings.notify.none") }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("settings.discord.lastSync") }}</span>
            <strong>{{ formatTime(discordPresenceSnapshot.lastSyncAt) }}</strong>
          </div>
        </div>

        <div class="actions-row discord-presence-actions">
          <Button
            :label="t('settings.discord.start')"
            icon="pi pi-desktop"
            :loading="discordBusy"
            :disabled="discordPresenceSnapshot.running || !discordConfigReady"
            @click="$emit('startDiscordPresence')"
          />
          <Button
            :label="t('settings.discord.stop')"
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
            {{ t("settings.discord.autoStartHint") }}
          </Message>
          <Message v-else severity="secondary" :closable="false">
            {{ t("settings.discord.idleHint") }}
          </Message>
        </div>
      </template>
    </Card>

    <Card v-if="selfTestSupported && selfTestResult" class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("settings.selfTest.eyebrow") }}</p>
            <h3>{{ t("settings.selfTest.title") }}</h3>
          </div>
          <Tag :value="selfTestResult.platform" severity="contrast" rounded />
        </div>
      </template>
      <template #content>
        <div class="self-test-grid">
          <article class="self-test-card">
            <div class="self-test-head">
              <strong>{{ t("settings.selfTest.foreground") }}</strong>
              <Tag
                :value="selfTestResult.foreground.success ? t('settings.selfTest.usable') : t('settings.selfTest.abnormal')"
                :severity="selfTestResult.foreground.success ? 'success' : 'danger'"
                rounded
              />
            </div>
            <p class="self-test-detail">
              {{ primaryProbeText(selfTestResult.foreground) }}
            </p>
            <p v-if="secondaryProbeText(selfTestResult.foreground)" class="self-test-summary">
              {{ secondaryProbeText(selfTestResult.foreground) }}
            </p>
          </article>

          <article class="self-test-card">
            <div class="self-test-head">
              <strong>{{ t("settings.selfTest.windowTitle") }}</strong>
              <Tag
                :value="selfTestResult.windowTitle.success ? t('settings.selfTest.usable') : t('settings.selfTest.abnormal')"
                :severity="selfTestResult.windowTitle.success ? 'success' : 'danger'"
                rounded
              />
            </div>
            <p class="self-test-detail">
              {{ primaryProbeText(selfTestResult.windowTitle) }}
            </p>
            <p v-if="secondaryProbeText(selfTestResult.windowTitle)" class="self-test-summary">
              {{ secondaryProbeText(selfTestResult.windowTitle) }}
            </p>
            <div
              v-if="selfTestResult.platform === 'macos' && !selfTestResult.windowTitle.success"
              class="actions-row"
            >
              <Button
                :label="t('settings.reporter.accessibility')"
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
              <strong>{{ t("settings.selfTest.media") }}</strong>
              <Tag
                :value="selfTestResult.media.success ? t('settings.selfTest.usable') : t('settings.selfTest.abnormal')"
                :severity="selfTestResult.media.success ? 'success' : 'danger'"
                rounded
              />
            </div>
            <p class="self-test-detail">
              {{ primaryProbeText(selfTestResult.media) }}
            </p>
            <p v-if="secondaryProbeText(selfTestResult.media)" class="self-test-summary">
              {{ secondaryProbeText(selfTestResult.media) }}
            </p>
          </article>
        </div>

        <div class="message-stack">
          <Message
            v-if="selfTestResult.platform === 'macos'"
            severity="secondary"
            :closable="false"
          >
            {{ t("settings.selfTest.macosHint") }}
          </Message>
          <Message
            v-if="selfTestResult.platform === 'linux'"
            severity="secondary"
            :closable="false"
          >
            {{ t("settings.selfTest.linuxHint") }}
          </Message>
        </div>
      </template>
    </Card>
  </div>
</template>

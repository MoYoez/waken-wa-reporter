<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import Button from "primevue/button";
import Card from "primevue/card";
import Message from "primevue/message";
import Tag from "primevue/tag";

import { resolveReporterLogDetail, resolveReporterLogTitle } from "../lib/reporterLogText";
import type {
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  MobileConnectivityState,
  RealtimeReporterSnapshot,
} from "../types";

const { t, locale } = useI18n();

const props = defineProps<{
  config: ClientConfig;
  readiness: boolean;
  capabilities: ClientCapabilities;
  mobileConnectivity: MobileConnectivityState;
  reporterSnapshot: RealtimeReporterSnapshot;
  discordPresenceSnapshot: DiscordPresenceSnapshot;
  reporterBusy: boolean;
}>();

defineEmits<{
  startReporter: [];
  stopReporter: [];
  retryMobileConnectivity: [];
}>();

const latestLogs = computed(() => props.reporterSnapshot.logs.slice(0, 4));
const reporterSupported = computed(() => props.capabilities.realtimeReporter);
const discordSupported = computed(() => props.capabilities.discordPresence);
const effectiveModeLabel = computed(() => {
  if (!reporterSupported.value) {
    return t("overview.modes.activity");
  }

  return props.reporterSnapshot.running
    ? t("overview.modes.realtime")
    : t("overview.modes.activity");
});

function formatTime(value?: string | null) {
  if (!value) return t("overview.common.none");

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString(locale.value);
}

function translateLogText(key: string, params?: Record<string, unknown> | null) {
  return params ? t(key, params) : t(key);
}

function logTitle(log: RealtimeReporterSnapshot["logs"][number]) {
  return resolveReporterLogTitle(log, translateLogText);
}

function logDetail(log: RealtimeReporterSnapshot["logs"][number]) {
  return resolveReporterLogDetail(log, translateLogText);
}
</script>

<template>
  <div class="workspace-grid">
    <header class="hero-panel">
      <div>
        <p class="eyebrow">{{ t("overview.hero.eyebrow") }}</p>
        <h2>{{ t("overview.hero.title") }}</h2>
        <p class="hero-copy">{{ t("overview.hero.description") }}</p>
      </div>
      <div class="hero-actions">
        <Tag
          v-if="reporterSupported"
          :value="reporterSnapshot.running ? t('overview.common.running') : t('overview.common.stopped')"
          :severity="reporterSnapshot.running ? 'success' : 'warn'"
          rounded
        />
        <Tag v-else :value="t('overview.hero.mobileMode')" severity="info" rounded />
      </div>
    </header>

    <Card class="glass-card overview-summary-card">
      <template #content>
        <div class="overview-summary">
          <div class="overview-item">
            <span>{{ t("overview.summary.siteUrl") }}</span>
            <strong>{{ config.baseUrl || t("overview.summary.notSet") }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("overview.summary.deviceName") }}</span>
            <strong>{{ config.device || t("overview.summary.unnamedDevice") }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("overview.summary.currentMode") }}</span>
            <strong>{{ effectiveModeLabel }}</strong>
          </div>
          <div class="overview-item">
            <span>
              {{ reporterSupported ? t("overview.summary.lastHeartbeat") : t("overview.summary.deviceType") }}
            </span>
            <strong>
              {{ reporterSupported ? formatTime(reporterSnapshot.lastHeartbeatAt) : config.deviceType }}
            </strong>
          </div>
        </div>
      </template>
    </Card>

    <Card v-if="discordSupported" class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("overview.discord.eyebrow") }}</p>
            <h3>{{ t("overview.discord.title") }}</h3>
          </div>
          <Tag
            :value="discordPresenceSnapshot.running ? (discordPresenceSnapshot.connected ? t('overview.discord.running') : t('overview.discord.waiting')) : t('overview.discord.notStarted')"
            :severity="discordPresenceSnapshot.running ? (discordPresenceSnapshot.connected ? 'success' : 'warn') : 'secondary'"
            rounded
          />
        </div>
      </template>
      <template #content>
        <div class="overview-summary">
          <div class="overview-item">
            <span>{{ t("overview.discord.syncStatus") }}</span>
            <strong>
              {{ discordPresenceSnapshot.running ? t("overview.discord.started") : t("overview.discord.notStarted") }}
            </strong>
          </div>
          <div class="overview-item">
            <span>{{ t("overview.discord.connection") }}</span>
            <strong>
              {{ discordPresenceSnapshot.connected ? t("overview.discord.connected") : t("overview.discord.disconnected") }}
            </strong>
          </div>
          <div class="overview-item">
            <span>{{ t("overview.discord.currentSummary") }}</span>
            <strong>{{ discordPresenceSnapshot.currentSummary || t("overview.common.none") }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("overview.discord.lastSync") }}</span>
            <strong>{{ formatTime(discordPresenceSnapshot.lastSyncAt) }}</strong>
          </div>
        </div>

        <div class="message-stack">
          <Message v-if="discordPresenceSnapshot.lastError" severity="warn" :closable="false">
            {{ discordPresenceSnapshot.lastError }}
          </Message>
          <Message
            v-else-if="discordPresenceSnapshot.running"
            :severity="discordPresenceSnapshot.connected ? 'success' : 'secondary'"
            :closable="false"
          >
            {{
              discordPresenceSnapshot.connected
                ? t("overview.discord.activeMessage")
                : t("overview.discord.waitingMessage")
            }}
          </Message>
          <Message v-else severity="secondary" :closable="false">
            {{ t("overview.discord.idleMessage") }}
          </Message>
        </div>
      </template>
    </Card>

    <Card v-if="reporterSupported" class="glass-card">
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
            <strong>{{ reporterSnapshot.currentActivity?.processName || t("overview.common.none") }}</strong>
          </div>
          <div class="overview-item">
            <span>{{ t("overview.reporter.windowTitle") }}</span>
            <strong>{{ reporterSnapshot.currentActivity?.processTitle || t("overview.common.none") }}</strong>
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
            :disabled="reporterSnapshot.running || !readiness"
            @click="$emit('startReporter')"
          />
          <Button
            :label="t('overview.reporter.stop')"
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
            {{ t("overview.reporter.autoStartMessage") }}
          </Message>
          <Message v-else-if="!reporterSnapshot.running" severity="secondary" :closable="false">
            {{ t("overview.reporter.idleMessage") }}
          </Message>
        </div>
      </template>
    </Card>

    <Card v-else class="glass-card">
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
            <strong>
              {{
                mobileConnectivity.checking
                  ? t("overview.mobile.checking")
                  : mobileConnectivity.ok === true
                    ? t("overview.mobile.passed")
                    : mobileConnectivity.checked
                      ? t("overview.mobile.failed")
                      : t("overview.mobile.pending")
              }}
            </strong>
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

    <Card v-if="reporterSupported" class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("overview.logs.eyebrow") }}</p>
            <h3>{{ t("overview.logs.title") }}</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div v-if="latestLogs.length" class="log-list">
          <article v-for="log in latestLogs" :key="log.id" class="log-item">
            <div class="log-header">
              <strong>{{ logTitle(log) }}</strong>
              <small>{{ formatTime(log.timestamp) }}</small>
            </div>
            <p class="log-detail">{{ logDetail(log) }}</p>
          </article>
        </div>
        <Message v-else severity="secondary" :closable="false">
          {{ t("overview.logs.empty") }}
        </Message>
      </template>
    </Card>
  </div>
</template>

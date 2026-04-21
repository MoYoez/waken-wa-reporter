<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import Tag from "primevue/tag";

import OverviewDiscordCard from "@/features/overview/components/OverviewDiscordCard.vue";
import OverviewLogsCard from "@/features/overview/components/OverviewLogsCard.vue";
import OverviewMobileCard from "@/features/overview/components/OverviewMobileCard.vue";
import OverviewReporterCard from "@/features/overview/components/OverviewReporterCard.vue";
import OverviewSummaryCard from "@/features/overview/components/OverviewSummaryCard.vue";
import type {
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  MobileConnectivityState,
  RealtimeReporterSnapshot,
} from "@/types";

const { t } = useI18n();

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

    <OverviewSummaryCard
      :config="config"
      :reporter-supported="reporterSupported"
      :effective-mode-label="effectiveModeLabel"
      :last-heartbeat-at="reporterSnapshot.lastHeartbeatAt"
    />

    <OverviewReporterCard
      v-if="reporterSupported"
      :config="config"
      :readiness="readiness"
      :snapshot="reporterSnapshot"
      :reporter-busy="reporterBusy"
      @start-reporter="$emit('startReporter')"
      @stop-reporter="$emit('stopReporter')"
    />

    <OverviewMobileCard
      v-else
      :readiness="readiness"
      :mobile-connectivity="mobileConnectivity"
      @retry-mobile-connectivity="$emit('retryMobileConnectivity')"
    />

    <OverviewLogsCard
      v-if="reporterSupported"
      :logs="latestLogs"
    />

    <OverviewDiscordCard
      v-if="discordSupported"
      :snapshot="discordPresenceSnapshot"
    />
  </div>
</template>

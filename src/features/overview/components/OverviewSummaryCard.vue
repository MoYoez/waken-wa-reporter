<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import Card from "primevue/card";

import { formatOptionalDateTime } from "@/lib/dateTime";
import type { ClientConfig } from "@/types";

const { t, locale } = useI18n();

const props = defineProps<{
  config: ClientConfig;
  reporterSupported: boolean;
  effectiveModeLabel: string;
  lastHeartbeatAt?: string | null;
}>();

const secondaryValue = computed(() =>
  props.reporterSupported
    ? formatOptionalDateTime(
      props.lastHeartbeatAt,
      locale.value,
      t("overview.common.none"),
    )
    : props.config.deviceType,
);
</script>

<template>
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
          <strong>{{ secondaryValue }}</strong>
        </div>
      </div>
    </template>
  </Card>
</template>

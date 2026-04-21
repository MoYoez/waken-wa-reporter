<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import Card from "primevue/card";
import Message from "primevue/message";

import { formatOptionalDateTime } from "@/lib/dateTime";
import { resolveReporterLogDetail, resolveReporterLogTitle } from "@/lib/reporterLogText";
import type { ReporterLogEntry } from "@/types";

const { t, locale } = useI18n();

const props = defineProps<{
  logs: ReporterLogEntry[];
}>();

const translatedLogs = computed(() => props.logs.map((log) => ({
  ...log,
  resolvedTitle: resolveReporterLogTitle(log, translateText),
  resolvedDetail: resolveReporterLogDetail(log, translateText),
  formattedTimestamp: formatOptionalDateTime(log.timestamp, locale.value, log.timestamp),
})));

function translateText(key: string, params?: Record<string, unknown> | null) {
  return params ? t(key, params) : t(key);
}
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("overview.logs.eyebrow") }}</p>
          <h3>{{ t("overview.logs.title") }}</h3>
        </div>
      </div>
    </template>
    <template #content>
      <div v-if="translatedLogs.length" class="log-list">
        <article v-for="log in translatedLogs" :key="log.id" class="log-item">
          <div class="log-header">
            <strong>{{ log.resolvedTitle }}</strong>
            <small>{{ log.formattedTimestamp }}</small>
          </div>
          <p class="log-detail">{{ log.resolvedDetail }}</p>
        </article>
      </div>
      <Message v-else severity="secondary" :closable="false">
        {{ t("overview.logs.empty") }}
      </Message>
    </template>
  </Card>
</template>

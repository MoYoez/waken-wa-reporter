<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import Card from "primevue/card";
import Message from "primevue/message";
import Tag from "primevue/tag";

import { formatOptionalDateTime } from "@/lib/dateTime";
import type { DiscordPresenceSnapshot } from "@/types";

const { t, locale } = useI18n();

const props = defineProps<{
  snapshot: DiscordPresenceSnapshot;
}>();

const statusLabel = computed(() => (
  props.snapshot.running
    ? (props.snapshot.connected ? t("overview.discord.running") : t("overview.discord.waiting"))
    : t("overview.discord.notStarted")
));

const statusSeverity = computed(() => (
  props.snapshot.running
    ? (props.snapshot.connected ? "success" : "warn")
    : "secondary"
));

const lastSyncLabel = computed(() => formatOptionalDateTime(
  props.snapshot.lastSyncAt,
  locale.value,
  t("overview.common.none"),
));
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("overview.discord.eyebrow") }}</p>
          <h3>{{ t("overview.discord.title") }}</h3>
        </div>
        <Tag :value="statusLabel" :severity="statusSeverity" rounded />
      </div>
    </template>
    <template #content>
      <div class="overview-summary">
        <div class="overview-item">
          <span>{{ t("overview.discord.syncStatus") }}</span>
          <strong>
            {{ snapshot.running ? t("overview.discord.started") : t("overview.discord.notStarted") }}
          </strong>
        </div>
        <div class="overview-item">
          <span>{{ t("overview.discord.connection") }}</span>
          <strong>
            {{ snapshot.connected ? t("overview.discord.connected") : t("overview.discord.disconnected") }}
          </strong>
        </div>
        <div class="overview-item">
          <span>{{ t("overview.discord.currentSummary") }}</span>
          <strong>{{ snapshot.currentSummary || t("overview.common.none") }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("overview.discord.lastSync") }}</span>
          <strong>{{ lastSyncLabel }}</strong>
        </div>
      </div>

      <div class="message-stack">
        <Message v-if="snapshot.lastError" severity="warn" :closable="false">
          {{ snapshot.lastError }}
        </Message>
        <Message
          v-else-if="snapshot.running"
          :severity="snapshot.connected ? 'success' : 'secondary'"
          :closable="false"
        >
          {{
            snapshot.connected
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
</template>

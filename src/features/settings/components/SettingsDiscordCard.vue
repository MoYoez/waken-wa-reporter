<script setup lang="ts">
import Button from "primevue/button";
import Card from "primevue/card";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import Tag from "primevue/tag";
import ToggleSwitch from "primevue/toggleswitch";
import { useI18n } from "vue-i18n";

import type { DiscordPresenceSnapshot } from "@/types";

defineProps<{
  applicationId: string;
  enabled: boolean;
  snapshot: DiscordPresenceSnapshot;
  discordBusy: boolean;
  issues: string[];
  canStart: boolean;
}>();

const emit = defineEmits<{
  updateApplicationId: [value: string];
  updateEnabled: [value: boolean];
  start: [];
  stop: [];
}>();

const { t, locale } = useI18n();

function formatTime(value?: string | null) {
  if (!value) {
    return t("settings.notify.none");
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString(locale.value);
}
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("settings.discord.eyebrow") }}</p>
          <h3>{{ t("settings.discord.title") }}</h3>
        </div>
        <Tag
          :value="snapshot.running ? (snapshot.connected ? t('settings.tags.running') : t('settings.tags.waitingDiscord')) : t('settings.tags.notStarted')"
          :severity="snapshot.running ? (snapshot.connected ? 'success' : 'warn') : 'secondary'"
          rounded
        />
      </div>
    </template>
    <template #content>
      <div class="panel-grid">
        <label class="field-block field-span-2">
          <span class="field-label">{{ t("settings.discord.appId") }}</span>
          <InputText
            :model-value="applicationId"
            :placeholder="t('settings.discord.appIdPlaceholder')"
            @update:model-value="emit('updateApplicationId', $event ?? '')"
          />
        </label>

        <div class="reporter-enabled-card discord-autostart-card field-span-2">
          <div class="reporter-enabled-copy">
            <span class="field-label">{{ t("settings.discord.autoStart") }}</span>
            <strong>{{ enabled ? t("settings.tags.enabled") : t("settings.tags.disabled") }}</strong>
            <span>
              {{ t("settings.discord.autoStartDetail") }}
            </span>
          </div>
          <ToggleSwitch
            :model-value="enabled"
            input-id="settings-discord-enabled"
            @update:model-value="emit('updateEnabled', Boolean($event))"
          />
        </div>
      </div>

      <div class="overview-summary discord-presence-summary">
        <div class="overview-item">
          <span>{{ t("settings.reporter.runtimeStatus") }}</span>
          <strong>{{ snapshot.running ? t("settings.tags.running") : t("settings.tags.notStarted") }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("settings.discord.connection") }}</span>
          <strong>{{ snapshot.connected ? t("settings.tags.connected") : t("settings.tags.notConnected") }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("settings.discord.currentSummary") }}</span>
          <strong>{{ snapshot.currentSummary || t("settings.notify.none") }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("settings.discord.lastSync") }}</span>
          <strong>{{ formatTime(snapshot.lastSyncAt) }}</strong>
        </div>
      </div>

      <div class="actions-row discord-presence-actions">
        <Button
          :label="t('settings.discord.start')"
          icon="pi pi-desktop"
          :loading="discordBusy"
          :disabled="snapshot.running || !canStart"
          @click="emit('start')"
        />
        <Button
          :label="t('settings.discord.stop')"
          icon="pi pi-stop"
          severity="secondary"
          outlined
          :loading="discordBusy"
          :disabled="!snapshot.running"
          @click="emit('stop')"
        />
      </div>

      <div class="message-stack">
        <Message
          v-for="issue in issues"
          :key="issue"
          severity="warn"
          :closable="false"
        >
          {{ issue }}
        </Message>
        <Message v-if="snapshot.lastError" severity="warn" :closable="false">
          {{ snapshot.lastError }}
        </Message>
        <Message
          v-else-if="enabled"
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
</template>

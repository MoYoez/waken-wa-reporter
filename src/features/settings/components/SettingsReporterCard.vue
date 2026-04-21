<script setup lang="ts">
import Button from "primevue/button";
import Card from "primevue/card";
import Message from "primevue/message";
import Tag from "primevue/tag";
import { useI18n } from "vue-i18n";

import type { SelfTestCardView } from "@/features/settings/composables/settingsWorkspaceProbeText";
import type { RealtimeReporterSnapshot } from "@/types";

defineProps<{
  configReady: boolean;
  snapshot: RealtimeReporterSnapshot;
  reporterBusy: boolean;
  selfTestSupported: boolean;
  canRequestAccessibilityPermission: boolean;
  selfTestLoading: boolean;
  accessibilityPermissionLoading: boolean;
  selfTestPlatform: string;
  selfTestCards: SelfTestCardView[];
  selfTestHintKey: string;
}>();

const emit = defineEmits<{
  start: [];
  stop: [];
  selfTest: [];
  requestAccessibilityPermission: [];
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
          <p class="eyebrow">{{ t("settings.reporter.eyebrow") }}</p>
          <h3>{{ t("settings.reporter.title") }}</h3>
        </div>
      </div>
    </template>
    <template #content>
      <div class="overview-summary">
        <div class="overview-item">
          <span>{{ t("settings.reporter.runtimeStatus") }}</span>
          <strong>{{ snapshot.running ? t("settings.tags.running") : t("settings.tags.notStarted") }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("settings.reporter.currentProcess") }}</span>
          <strong>{{ snapshot.currentActivity?.processName || t("settings.notify.none") }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("settings.reporter.lastHeartbeat") }}</span>
          <strong>{{ formatTime(snapshot.lastHeartbeatAt) }}</strong>
        </div>
        <div class="overview-item">
          <span>{{ t("settings.reporter.lastError") }}</span>
          <strong>{{ snapshot.lastError || t("settings.notify.none") }}</strong>
        </div>
      </div>

      <div class="actions-row">
        <Button
          :label="t('settings.reporter.start')"
          icon="pi pi-play"
          :loading="reporterBusy"
          :disabled="snapshot.running || !configReady"
          @click="emit('start')"
        />
        <Button
          :label="t('settings.reporter.stop')"
          icon="pi pi-stop"
          severity="secondary"
          outlined
          :loading="reporterBusy"
          :disabled="!snapshot.running"
          @click="emit('stop')"
        />
        <Button
          v-if="selfTestSupported"
          :label="t('settings.reporter.selfTest')"
          icon="pi pi-search"
          severity="secondary"
          text
          :loading="selfTestLoading"
          @click="emit('selfTest')"
        />
        <Button
          v-if="canRequestAccessibilityPermission"
          :label="t('settings.reporter.accessibility')"
          icon="pi pi-shield"
          severity="secondary"
          outlined
          :loading="accessibilityPermissionLoading"
          @click="emit('requestAccessibilityPermission')"
        />
      </div>

      <div class="message-stack">
        <Message v-if="!configReady" severity="warn" :closable="false">
          {{ t("settings.reporter.configRequired") }}
        </Message>
        <Message v-if="snapshot.lastError" severity="error" :closable="false">
          {{ snapshot.lastError }}
        </Message>
        <Message v-else severity="secondary" :closable="false">
          {{ t("settings.reporter.autoStartHint") }}
        </Message>
      </div>

      <section
        v-if="selfTestCards.length"
        class="reporter-self-test-section"
      >
        <div class="panel-heading reporter-self-test-heading">
          <div>
            <p class="eyebrow">{{ t("settings.selfTest.eyebrow") }}</p>
            <h3>{{ t("settings.selfTest.title") }}</h3>
          </div>
          <Tag
            v-if="selfTestPlatform"
            :value="selfTestPlatform"
            severity="contrast"
            rounded
          />
        </div>

        <div class="self-test-grid">
          <article
            v-for="card in selfTestCards"
            :key="card.key"
            class="self-test-card"
          >
            <div class="self-test-head">
              <strong>{{ t(card.titleKey) }}</strong>
              <Tag
                :value="card.success ? t('settings.selfTest.usable') : t('settings.selfTest.abnormal')"
                :severity="card.success ? 'success' : 'danger'"
                rounded
              />
            </div>
            <p class="self-test-detail">
              {{ card.primaryText }}
            </p>
            <p
              v-if="card.secondaryText"
              class="self-test-summary"
            >
              {{ card.secondaryText }}
            </p>
            <div
              v-if="card.showAccessibilityAction"
              class="actions-row"
            >
              <Button
                :label="t('settings.reporter.accessibility')"
                icon="pi pi-shield"
                severity="secondary"
                outlined
                :loading="accessibilityPermissionLoading"
                @click="emit('requestAccessibilityPermission')"
              />
            </div>
          </article>
        </div>

        <div class="message-stack">
          <Message
            v-if="selfTestHintKey"
            severity="secondary"
            :closable="false"
          >
            {{ t(selfTestHintKey) }}
          </Message>
        </div>
      </section>
    </template>
  </Card>
</template>

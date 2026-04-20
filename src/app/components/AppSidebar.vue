<script setup lang="ts">
import Tag from "primevue/tag";
import { useI18n } from "vue-i18n";

import type { AppSection, SectionNavItem } from "@/app/types";

defineProps<{
  visibleSections: SectionNavItem[];
  activeSection: AppSection;
  readiness: boolean;
  reporterSupported: boolean;
  reporterRunning: boolean;
  discordSupported: boolean;
  discordRunning: boolean;
  discordConnected: boolean;
  traySupported: boolean;
}>();

const emit = defineEmits<{
  select: [section: AppSection];
}>();

const { t } = useI18n();
</script>

<template>
  <aside class="app-sidebar">
    <div class="brand-block">
      <p class="eyebrow">Waken-Wa</p>
      <h1>{{ t("app.brand.client") }}</h1>
    </div>

    <nav class="nav-stack">
      <button
        v-for="section in visibleSections"
        :key="section.key"
        class="nav-item"
        :class="{ active: section.key === activeSection }"
        type="button"
        @click="emit('select', section.key)"
      >
        <i :class="section.icon" />
        <div>
          <strong>{{ section.title }}</strong>
          <span>{{ section.kicker }}</span>
        </div>
      </button>
    </nav>

    <div class="sidebar-footer">
      <Tag
        :value="readiness ? t('app.sidebar.readinessReady') : t('app.sidebar.readinessPending')"
        :severity="readiness ? 'success' : 'warn'"
        rounded
      />
      <Tag
        v-if="reporterSupported"
        :value="reporterRunning ? t('app.sidebar.reporterRunning') : t('app.sidebar.reporterStopped')"
        :severity="reporterRunning ? 'success' : 'secondary'"
        rounded
      />
      <Tag
        v-if="discordSupported"
        :value="discordRunning ? (discordConnected ? t('app.sidebar.discordRunning') : t('app.sidebar.discordWaiting')) : t('app.sidebar.discordStopped')"
        :severity="discordRunning ? (discordConnected ? 'success' : 'warn') : 'secondary'"
        rounded
      />
      <small v-if="traySupported">{{ t("app.sidebar.traySupported") }}</small>
      <small v-else>{{ t("app.sidebar.trayUnsupported") }}</small>
    </div>
  </aside>
</template>

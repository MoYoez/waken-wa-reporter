<script setup lang="ts">
import Tag from "primevue/tag";
import { useI18n } from "vue-i18n";

defineProps<{
  reporterSupported: boolean;
  reporterRunning: boolean;
}>();

const { t } = useI18n();
</script>

<template>
  <header class="hero-panel">
    <div>
      <p class="eyebrow">{{ t("settings.hero.eyebrow") }}</p>
      <h2>{{ t("settings.hero.title") }}</h2>
      <p class="hero-copy">{{ t("settings.hero.description") }}</p>
    </div>
    <div class="hero-actions">
      <Tag
        :value="reporterSupported ? (reporterRunning ? t('settings.tags.running') : t('settings.tags.notStarted')) : t('settings.tags.mobileMode')"
        :severity="reporterSupported ? (reporterRunning ? 'success' : 'warn') : 'info'"
        rounded
      />
    </div>
  </header>
</template>

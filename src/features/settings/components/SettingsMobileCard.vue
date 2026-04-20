<script setup lang="ts">
import Card from "primevue/card";
import Message from "primevue/message";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
</script>

<template>
  <Card class="glass-card">
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
</template>

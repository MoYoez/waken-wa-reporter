<script setup lang="ts">
import Button from "primevue/button";
import Card from "primevue/card";
import Message from "primevue/message";
import Tag from "primevue/tag";
import { useI18n } from "vue-i18n";

interface SelfTestCardView {
  key: "foreground" | "windowTitle" | "media";
  titleKey: string;
  success: boolean;
  primaryText: string;
  secondaryText: string;
  showAccessibilityAction?: boolean;
}

defineProps<{
  platform: string;
  cards: SelfTestCardView[];
  hintKey: string;
  accessibilityPermissionLoading: boolean;
}>();

const emit = defineEmits<{
  requestAccessibilityPermission: [];
}>();

const { t } = useI18n();
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("settings.selfTest.eyebrow") }}</p>
          <h3>{{ t("settings.selfTest.title") }}</h3>
        </div>
        <Tag :value="platform" severity="contrast" rounded />
      </div>
    </template>
    <template #content>
      <div class="self-test-grid">
        <article
          v-for="card in cards"
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
          <p v-if="card.secondaryText" class="self-test-summary">
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
          v-if="hintKey"
          severity="secondary"
          :closable="false"
        >
          {{ t(hintKey) }}
        </Message>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import { useI18n } from "vue-i18n";

import ConnectionPanel from "@/features/settings/components/ConnectionPanel.vue";
import type {
  ClientCapabilities,
  ClientConfig,
  ExistingReporterConfig,
} from "@/types";

const props = defineProps<{
  visible: boolean;
  setupMode: boolean;
  reporterSupported: boolean;
  existingReporterConfig: ExistingReporterConfig | null;
  reporterConfigPromptHandled: boolean;
  importingReporterConfig: boolean;
  modelValue: ClientConfig;
  capabilities: ClientCapabilities;
  verifiedGeneratedHashKey: string;
}>();

const emit = defineEmits<{
  close: [];
  startSetup: [];
  skipExistingConfig: [];
  useExistingConfig: [];
  complete: [];
  back: [];
  "update:modelValue": [value: ClientConfig];
  imported: [message: string];
}>();

const { t } = useI18n();

const canComplete = computed(() => (
  props.modelValue.baseUrl.trim()
  && props.modelValue.apiToken.trim()
  && props.modelValue.generatedHashKey.trim()
));
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    dismissable-mask
    :draggable="false"
    :closable="false"
    class="onboarding-dialog"
    @update:visible="(nextVisible) => !nextVisible && emit('close')"
  >
    <template #container>
      <div class="onboarding-panel">
        <template v-if="!setupMode">
          <p class="eyebrow">{{ t("app.onboarding.eyebrow") }}</p>
          <h3>{{ t("app.onboarding.welcomeTitle") }}</h3>
          <p class="onboarding-copy">
            {{ t("app.onboarding.welcomeCopy") }}
          </p>
          <div
            v-if="reporterSupported && existingReporterConfig?.found && !reporterConfigPromptHandled"
            class="onboarding-step onboarding-highlight"
          >
            <strong>{{ t("app.onboarding.foundConfigTitle") }}</strong>
            <span>{{ t("app.onboarding.foundConfigDetail") }}</span>
            <small v-if="existingReporterConfig.path">{{ existingReporterConfig.path }}</small>
            <div class="actions-row">
              <Button
                :label="t('app.onboarding.useExistingConfig')"
                icon="pi pi-download"
                :loading="importingReporterConfig"
                @click="emit('useExistingConfig')"
              />
              <Button
                :label="t('app.onboarding.skipImport')"
                severity="secondary"
                text
                @click="emit('skipExistingConfig')"
              />
            </div>
          </div>
          <div class="onboarding-steps">
            <div class="onboarding-step">
              <strong>{{ t("app.onboarding.step1Title") }}</strong>
              <span>{{ t("app.onboarding.step1Detail") }}</span>
            </div>
            <div class="onboarding-step">
              <strong>{{ t("app.onboarding.step2Title") }}</strong>
              <span>{{ t("app.onboarding.step2Detail") }}</span>
            </div>
            <div class="onboarding-step">
              <strong>{{ t("app.onboarding.step3Title") }}</strong>
              <span>{{ t("app.onboarding.step3Detail") }}</span>
            </div>
          </div>
          <div class="actions-row">
            <Button
              :label="t('app.onboarding.goToSettings')"
              icon="pi pi-arrow-right"
              @click="emit('startSetup')"
            />
            <Button
              :label="t('app.onboarding.later')"
              severity="secondary"
              text
              @click="emit('close')"
            />
          </div>
        </template>

        <template v-else>
          <p class="eyebrow">{{ t("app.onboarding.eyebrow") }}</p>
          <h3>{{ t("app.onboarding.setupTitle") }}</h3>
          <p class="onboarding-copy">
            {{ t("app.onboarding.setupCopy") }}
          </p>
          <ConnectionPanel
            :model-value="modelValue"
            :capabilities="capabilities"
            :verified-generated-hash-key="verifiedGeneratedHashKey"
            variant="onboarding"
            @update:model-value="(value) => emit('update:modelValue', value)"
            @imported="(message) => emit('imported', message)"
          />
          <div class="actions-row">
            <Button
              :label="t('app.onboarding.complete')"
              icon="pi pi-check"
              :disabled="!canComplete"
              @click="emit('complete')"
            />
            <Button
              :label="t('app.onboarding.back')"
              severity="secondary"
              text
              @click="emit('back')"
            />
          </div>
        </template>
      </div>
    </template>
  </Dialog>
</template>

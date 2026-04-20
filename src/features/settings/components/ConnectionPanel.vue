<script setup lang="ts">
import Card from "primevue/card";
import Message from "primevue/message";
import Tag from "primevue/tag";
import { useI18n } from "vue-i18n";

import ConnectionCoreFields from "@/features/settings/components/ConnectionCoreFields.vue";
import ConnectionImportSection from "@/features/settings/components/ConnectionImportSection.vue";
import ConnectionReporterSection from "@/features/settings/components/ConnectionReporterSection.vue";
import { useConnectionPanel } from "@/features/settings/composables/useConnectionPanel";
import type { ClientCapabilities, ClientConfig } from "@/types";

const props = defineProps<{
  modelValue: ClientConfig;
  capabilities: ClientCapabilities;
  variant?: "default" | "onboarding";
  verifiedGeneratedHashKey?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: ClientConfig];
  imported: [message: string];
}>();

const { t } = useI18n();

const {
  currentGeneratedHashKey,
  importConfig,
  importPayload,
  isOnboarding,
  issues,
  reporterContentOptions,
  reporterSupported,
  updateField,
} = useConnectionPanel(props, {
  onUpdateModelValue: (value) => emit("update:modelValue", value),
  onImported: (message) => emit("imported", message),
});
</script>

<template>
  <Card class="glass-card connection-panel">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">{{ t("connectionPanel.title.eyebrow") }}</p>
          <h3>{{ t("connectionPanel.title.title") }}</h3>
        </div>
        <Tag
          :severity="issues.length ? 'warn' : 'success'"
          :value="issues.length ? t('connectionPanel.status.pending') : t('connectionPanel.status.ready')"
          rounded
        />
      </div>
    </template>

    <template #content>
      <template v-if="isOnboarding">
        <ConnectionImportSection
          v-model:text="importPayload.text"
          title-key="connectionPanel.sections.onboardingImport"
          help-key="connectionPanel.help.onboardingImport"
          @import="importConfig"
        />

        <div class="settings-section">
          <details class="settings-disclosure">
            <summary class="settings-disclosure-summary">
              <div>
                <strong>{{ t("connectionPanel.sections.onboardingAdvanced") }}</strong>
                <span>{{ t("connectionPanel.help.onboardingAdvanced") }}</span>
              </div>
              <i class="pi pi-angle-down" aria-hidden="true" />
            </summary>
            <div class="settings-disclosure-body">
              <ConnectionCoreFields
                :model-value="modelValue"
                :current-generated-hash-key="currentGeneratedHashKey"
                :reporter-supported="reporterSupported"
                :show-device-name="false"
                proxy-input-id="onboarding-use-system-proxy"
                @update-field="updateField"
              />

              <template v-if="reporterSupported">
                <ConnectionReporterSection
                  :model-value="modelValue"
                  variant="onboarding"
                  title-key="connectionPanel.sections.onboardingReporter"
                  help-key="connectionPanel.help.reporterOnboarding"
                  head-class="settings-section-head settings-disclosure-subhead"
                  :reporter-content-options="reporterContentOptions"
                  @update-field="updateField"
                />
              </template>
              <div v-else class="settings-section-head settings-disclosure-subhead">
                <strong>{{ t("connectionPanel.sections.onboardingMobile") }}</strong>
                <span>{{ t("connectionPanel.help.mobileMode") }}</span>
              </div>
            </div>
          </details>
        </div>

        <div class="settings-section">
          <div class="settings-section-head">
            <strong>{{ t("connectionPanel.sections.onboardingDeviceName") }}</strong>
            <span>{{ t("connectionPanel.help.deviceName") }}</span>
          </div>
          <ConnectionCoreFields
            :model-value="modelValue"
            :current-generated-hash-key="currentGeneratedHashKey"
            :reporter-supported="false"
            :show-connection-fields="false"
            :show-generated-hash-key="false"
            :show-use-system-proxy="false"
            device-field-class="field-block field-span-2"
            @update-field="updateField"
          />
        </div>
      </template>

      <template v-else>
        <div class="settings-section">
          <div class="settings-section-head">
            <strong>{{ t("connectionPanel.sections.connection") }}</strong>
            <span>{{ t("connectionPanel.help.connection") }}</span>
          </div>
          <ConnectionCoreFields
            :model-value="modelValue"
            :current-generated-hash-key="currentGeneratedHashKey"
            :reporter-supported="reporterSupported"
            @update-field="updateField"
          />
        </div>

        <div v-if="reporterSupported" class="settings-section">
          <div class="settings-section-head">
            <strong>{{ t("connectionPanel.sections.reporter") }}</strong>
            <span>{{ t("connectionPanel.help.reporter") }}</span>
          </div>
          <ConnectionReporterSection
            :model-value="modelValue"
            variant="default"
            title-key="connectionPanel.sections.reporter"
            help-key="connectionPanel.help.reporter"
            :show-heading="false"
            :reporter-content-options="reporterContentOptions"
            head-class="settings-section-head"
            @update-field="updateField"
          />
        </div>
        <div v-else class="settings-section">
          <div class="settings-section-head">
            <strong>{{ t("connectionPanel.sections.mobile") }}</strong>
            <span>{{ t("connectionPanel.help.mobileMode") }}</span>
          </div>
        </div>

        <ConnectionImportSection
          v-model:text="importPayload.text"
          title-key="connectionPanel.sections.quickImport"
          help-key="connectionPanel.help.import"
          @import="importConfig"
        />
      </template>

      <div class="message-stack">
        <Message v-if="issues.length === 0" severity="success" :closable="false">
          {{ t("connectionPanel.messages.ready") }}
        </Message>
        <Message v-for="issue in issues" :key="issue" severity="warn" :closable="false">
          {{ issue }}
        </Message>
      </div>
    </template>
  </Card>
</template>

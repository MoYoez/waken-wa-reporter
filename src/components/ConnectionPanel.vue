<script setup lang="ts">
import { computed, reactive } from "vue";
import { useI18n } from "vue-i18n";
import Button from "primevue/button";
import Card from "primevue/card";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import Password from "primevue/password";
import Tag from "primevue/tag";
import Textarea from "primevue/textarea";
import ToggleSwitch from "primevue/toggleswitch";
import { useToast } from "primevue/usetoast";

import { parseImportedIntegrationConfig, validateConfig } from "../lib/api";
import { createNotifier } from "../lib/notify";
import type { ClientCapabilities, ClientConfig, DeviceType } from "../types";

const { t } = useI18n();

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

const toast = useToast();
const importPayload = reactive({ text: "" });
const isNativeNotice = computed(() => !props.capabilities.realtimeReporter);
const { notify } = createNotifier(toast, () => isNativeNotice.value);

const issues = computed(() => validateConfig(props.modelValue, props.capabilities));
const reporterSupported = computed(() => props.capabilities.realtimeReporter);
const isOnboarding = computed(() => props.variant === "onboarding");
const currentGeneratedHashKey = computed(() => props.modelValue.generatedHashKey.trim());
const reporterContentOptions = computed(() => [
  {
    key: "reportForegroundApp" as const,
    label: t("connectionPanel.reportContent.foreground.label"),
    description: t("connectionPanel.reportContent.foreground.description"),
    inputId: "report-foreground-app",
  },
  {
    key: "reportWindowTitle" as const,
    label: t("connectionPanel.reportContent.windowTitle.label"),
    description: t("connectionPanel.reportContent.windowTitle.description"),
    inputId: "report-window-title",
  },
  {
    key: "reportMedia" as const,
    label: t("connectionPanel.reportContent.media.label"),
    description: t("connectionPanel.reportContent.media.description"),
    inputId: "report-media",
  },
  {
    key: "reportPlaySource" as const,
    label: t("connectionPanel.reportContent.playSource.label"),
    description: t("connectionPanel.reportContent.playSource.description"),
    inputId: "report-play-source",
  },
]);

function updateField<K extends keyof ClientConfig>(key: K, value: ClientConfig[K]) {
  emit("update:modelValue", {
    ...props.modelValue,
    [key]: value,
  });
}

function inferMobileDeviceType(): DeviceType {
  if (typeof window === "undefined") return "mobile";
  return window.matchMedia("(max-width: 899px)").matches ? "mobile" : "tablet";
}

function toBaseUrl(reportEndpoint?: string) {
  if (!reportEndpoint) return undefined;
  return reportEndpoint.replace(/\/api\/activity\/?$/i, "").replace(/\/$/, "");
}

function importConfig() {
  parseImportedIntegrationConfig(importPayload.text)
    .then((parsed) => {
      emit("update:modelValue", {
        ...props.modelValue,
        baseUrl: toBaseUrl(parsed.reportEndpoint) ?? props.modelValue.baseUrl,
        apiToken: parsed.token ?? props.modelValue.apiToken,
        device: parsed.deviceName?.trim() || props.modelValue.device,
        deviceType: reporterSupported.value ? "desktop" : inferMobileDeviceType(),
      });
      emit(
        "imported",
        parsed.tokenName
          ? t("connectionPanel.notify.importedToken", { tokenName: parsed.tokenName })
          : t("connectionPanel.notify.importedConfig"),
      );
      notify({
        severity: "success",
        summary: t("connectionPanel.notify.importSuccess"),
        detail: toBaseUrl(parsed.reportEndpoint) ?? t("connectionPanel.notify.importSuccessDetail"),
        life: 3000,
      });
      importPayload.text = "";
    })
    .catch((error) => {
      notify({
        severity: "error",
        summary: t("connectionPanel.notify.importFailed"),
        detail: error instanceof Error ? error.message : t("connectionPanel.notify.importFailedDetail"),
        life: 4000,
      });
    });
}
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
        <div class="settings-section">
          <div class="settings-section-head">
            <strong>{{ t("connectionPanel.sections.onboardingImport") }}</strong>
            <span>{{ t("connectionPanel.help.onboardingImport") }}</span>
          </div>
          <div class="panel-grid">
            <label class="field-block field-span-2">
              <span class="field-label">{{ t("connectionPanel.fields.importConfig") }}</span>
              <Textarea
                v-model="importPayload.text"
                rows="4"
                auto-resize
                :placeholder="t('connectionPanel.placeholders.importConfig')"
              />
            </label>
          </div>
          <div class="actions-row">
            <Button :label="t('connectionPanel.buttons.import')" icon="pi pi-upload" @click="importConfig" />
          </div>
        </div>

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
              <div class="panel-grid">
                <label class="field-block field-span-2">
                  <span class="field-label">{{ t("connectionPanel.fields.baseUrl") }}</span>
                  <InputText
                    :model-value="modelValue.baseUrl"
                    :placeholder="t('connectionPanel.placeholders.baseUrl')"
                    @update:model-value="updateField('baseUrl', $event ?? '')"
                  />
                </label>

                <label class="field-block field-span-2">
                  <span class="field-label">{{ t("connectionPanel.fields.apiToken") }}</span>
                  <Password
                    :model-value="modelValue.apiToken"
                    :placeholder="t('connectionPanel.placeholders.apiToken')"
                    fluid
                    toggle-mask
                    :feedback="false"
                    @update:model-value="updateField('apiToken', $event)"
                  />
                </label>

                <label class="field-block field-span-2">
                  <span class="field-label">{{ t("connectionPanel.fields.generatedHashKey") }}</span>
                  <InputText
                    :model-value="currentGeneratedHashKey"
                    readonly
                    :placeholder="t('connectionPanel.placeholders.generatedHashKey')"
                  />
                  <small class="field-help">{{ t("connectionPanel.help.generatedHashKey") }}</small>
                </label>

                <div v-if="reporterSupported" class="reporter-enabled-card field-span-2">
                  <div class="reporter-enabled-copy">
                    <span class="field-label">{{ t("connectionPanel.fields.useSystemProxy") }}</span>
                    <strong>{{ modelValue.useSystemProxy ? t("connectionPanel.toggles.enabled") : t("connectionPanel.toggles.disabled") }}</strong>
                    <span>
                      {{ t("connectionPanel.help.useSystemProxy") }}
                    </span>
                  </div>
                  <ToggleSwitch
                    :model-value="modelValue.useSystemProxy"
                    input-id="onboarding-use-system-proxy"
                    @update:model-value="updateField('useSystemProxy', Boolean($event))"
                  />
                </div>
              </div>

              <template v-if="reporterSupported">
                <div class="settings-section-head settings-disclosure-subhead">
                  <strong>{{ t("connectionPanel.sections.onboardingReporter") }}</strong>
                  <span>{{ t("connectionPanel.help.reporterOnboarding") }}</span>
                </div>
                <div class="panel-grid">
                  <label class="field-block">
                    <span class="field-label">{{ t("connectionPanel.fields.pollInterval") }}</span>
                    <InputText
                      :model-value="String(modelValue.pollIntervalMs)"
                      :placeholder="t('connectionPanel.placeholders.pollInterval')"
                      @update:model-value="updateField('pollIntervalMs', Number($event ?? 0))"
                    />
                  </label>

                  <label class="field-block">
                    <span class="field-label">{{ t("connectionPanel.fields.heartbeatInterval") }}</span>
                    <InputText
                      :model-value="String(modelValue.heartbeatIntervalMs)"
                      :placeholder="t('connectionPanel.placeholders.heartbeatInterval')"
                      @update:model-value="updateField('heartbeatIntervalMs', Number($event ?? 0))"
                    />
                  </label>

                  <div class="reporter-enabled-card field-span-2">
                    <div class="reporter-enabled-copy">
                      <span class="field-label">{{ t("connectionPanel.fields.reporterEnabled") }}</span>
                      <strong>{{ modelValue.reporterEnabled ? t("connectionPanel.toggles.enabled") : t("connectionPanel.toggles.reporterDisabled") }}</strong>
                      <span>
                        {{ t("connectionPanel.help.reporterEnabledOnboarding") }}
                      </span>
                    </div>
                    <ToggleSwitch
                      :model-value="modelValue.reporterEnabled"
                      input-id="onboarding-reporter-enabled"
                      @update:model-value="updateField('reporterEnabled', Boolean($event))"
                    />
                  </div>

                  <div class="settings-section-head settings-disclosure-subhead field-span-2">
                    <strong>{{ t("connectionPanel.sections.reportContent") }}</strong>
                    <span>{{ t("connectionPanel.help.reportContent") }}</span>
                  </div>

                  <div class="compact-toggle-grid field-span-2">
                    <div
                      v-for="option in reporterContentOptions"
                      :key="`onboarding-${option.key}`"
                      class="compact-toggle-card"
                    >
                      <div class="compact-toggle-copy">
                        <strong>{{ option.label }}</strong>
                        <span>{{ option.description }}</span>
                      </div>
                      <ToggleSwitch
                        :model-value="modelValue[option.key]"
                        :input-id="`onboarding-${option.inputId}`"
                        @update:model-value="updateField(option.key, Boolean($event))"
                      />
                    </div>
                  </div>
                </div>
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
          <div class="panel-grid">
            <label class="field-block field-span-2">
              <span class="field-label">{{ t("connectionPanel.fields.deviceName") }}</span>
              <InputText
                :model-value="modelValue.device"
                :placeholder="t('connectionPanel.placeholders.deviceName')"
                @update:model-value="updateField('device', $event ?? '')"
              />
            </label>
          </div>
        </div>
      </template>

      <template v-else>
        <div class="settings-section">
          <div class="settings-section-head">
            <strong>{{ t("connectionPanel.sections.connection") }}</strong>
            <span>{{ t("connectionPanel.help.connection") }}</span>
          </div>
          <div class="panel-grid">
            <label class="field-block field-span-2">
              <span class="field-label">{{ t("connectionPanel.fields.baseUrl") }}</span>
              <InputText
                :model-value="modelValue.baseUrl"
                :placeholder="t('connectionPanel.placeholders.baseUrl')"
                @update:model-value="updateField('baseUrl', $event ?? '')"
              />
            </label>

            <label class="field-block field-span-2">
              <span class="field-label">{{ t("connectionPanel.fields.apiToken") }}</span>
              <Password
                :model-value="modelValue.apiToken"
                :placeholder="t('connectionPanel.placeholders.apiToken')"
                fluid
                toggle-mask
                :feedback="false"
                @update:model-value="updateField('apiToken', $event)"
              />
            </label>

            <label class="field-block">
              <span class="field-label">{{ t("connectionPanel.fields.deviceName") }}</span>
              <InputText
                :model-value="modelValue.device"
                :placeholder="t('connectionPanel.placeholders.deviceName')"
                @update:model-value="updateField('device', $event ?? '')"
              />
            </label>

            <label class="field-block field-span-2">
              <span class="field-label">{{ t("connectionPanel.fields.generatedHashKey") }}</span>
              <InputText
                :model-value="currentGeneratedHashKey"
                readonly
                :placeholder="t('connectionPanel.placeholders.generatedHashKey')"
              />
              <small class="field-help">{{ t("connectionPanel.help.generatedHashKey") }}</small>
            </label>

            <div v-if="reporterSupported" class="reporter-enabled-card field-span-2">
              <div class="reporter-enabled-copy">
                <span class="field-label">{{ t("connectionPanel.fields.useSystemProxy") }}</span>
                <strong>{{ modelValue.useSystemProxy ? t("connectionPanel.toggles.enabled") : t("connectionPanel.toggles.disabled") }}</strong>
                <span>
                  {{ t("connectionPanel.help.useSystemProxy") }}
                </span>
              </div>
              <ToggleSwitch
                :model-value="modelValue.useSystemProxy"
                input-id="settings-use-system-proxy"
                @update:model-value="updateField('useSystemProxy', Boolean($event))"
              />
            </div>
          </div>
        </div>

        <div v-if="reporterSupported" class="settings-section">
          <div class="settings-section-head">
            <strong>{{ t("connectionPanel.sections.reporter") }}</strong>
            <span>{{ t("connectionPanel.help.reporter") }}</span>
          </div>
          <div class="panel-grid">
            <label class="field-block">
              <span class="field-label">{{ t("connectionPanel.fields.pollInterval") }}</span>
              <InputText
                :model-value="String(modelValue.pollIntervalMs)"
                :placeholder="t('connectionPanel.placeholders.pollInterval')"
                @update:model-value="updateField('pollIntervalMs', Number($event ?? 0))"
              />
            </label>

            <label class="field-block">
              <span class="field-label">{{ t("connectionPanel.fields.heartbeatInterval") }}</span>
              <InputText
                :model-value="String(modelValue.heartbeatIntervalMs)"
                :placeholder="t('connectionPanel.placeholders.heartbeatInterval')"
                @update:model-value="updateField('heartbeatIntervalMs', Number($event ?? 0))"
              />
            </label>

            <div class="reporter-enabled-card field-span-2">
              <div class="reporter-enabled-copy">
                <span class="field-label">{{ t("connectionPanel.fields.reporterEnabled") }}</span>
                <strong>{{ modelValue.reporterEnabled ? t("connectionPanel.toggles.enabled") : t("connectionPanel.toggles.reporterDisabled") }}</strong>
                <span>
                  {{ t("connectionPanel.help.reporterEnabledSettings") }}
                </span>
              </div>
              <ToggleSwitch
                :model-value="modelValue.reporterEnabled"
                input-id="settings-reporter-enabled"
                @update:model-value="updateField('reporterEnabled', Boolean($event))"
              />
            </div>

            <div class="settings-section-head field-span-2">
              <strong>{{ t("connectionPanel.sections.reportContent") }}</strong>
              <span>{{ t("connectionPanel.help.reportContent") }}</span>
            </div>

            <div class="compact-toggle-grid field-span-2">
              <div
                v-for="option in reporterContentOptions"
                :key="option.key"
                class="compact-toggle-card"
              >
                <div class="compact-toggle-copy">
                  <strong>{{ option.label }}</strong>
                  <span>{{ option.description }}</span>
                </div>
                <ToggleSwitch
                  :model-value="modelValue[option.key]"
                  :input-id="option.inputId"
                  @update:model-value="updateField(option.key, Boolean($event))"
                />
              </div>
            </div>
          </div>
        </div>
        <div v-else class="settings-section">
          <div class="settings-section-head">
            <strong>{{ t("connectionPanel.sections.mobile") }}</strong>
            <span>{{ t("connectionPanel.help.mobileMode") }}</span>
          </div>
        </div>

        <div class="settings-section">
          <div class="settings-section-head">
            <strong>{{ t("connectionPanel.sections.quickImport") }}</strong>
            <span>{{ t("connectionPanel.help.import") }}</span>
          </div>
          <div class="panel-grid">
            <label class="field-block field-span-2">
              <span class="field-label">{{ t("connectionPanel.fields.importConfig") }}</span>
              <Textarea
                v-model="importPayload.text"
                rows="4"
                auto-resize
                :placeholder="t('connectionPanel.placeholders.importConfig')"
              />
            </label>
          </div>
        </div>

        <div class="actions-row">
          <Button :label="t('connectionPanel.buttons.import')" icon="pi pi-upload" @click="importConfig" />
        </div>
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

<script setup lang="ts">
import InputText from "primevue/inputtext";
import ToggleSwitch from "primevue/toggleswitch";
import { useI18n } from "vue-i18n";

import type { ClientConfig } from "@/types";

interface ReporterContentOption {
  key: "reportForegroundApp" | "reportWindowTitle" | "reportMedia" | "reportPlaySource";
  label: string;
  description: string;
  inputId: string;
}

const props = defineProps<{
  modelValue: ClientConfig;
  variant: "default" | "onboarding";
  titleKey: string;
  helpKey: string;
  reporterContentOptions: ReporterContentOption[];
  showHeading?: boolean;
  headClass?: string;
}>();

const emit = defineEmits<{
  updateField: [key: keyof ClientConfig, value: ClientConfig[keyof ClientConfig]];
}>();

const { t } = useI18n();
</script>

<template>
  <div v-if="props.showHeading !== false" :class="props.headClass || 'settings-section-head'">
    <strong>{{ t(props.titleKey) }}</strong>
    <span>{{ t(props.helpKey) }}</span>
  </div>
  <div class="panel-grid">
    <label class="field-block">
      <span class="field-label">{{ t("connectionPanel.fields.pollInterval") }}</span>
      <InputText
        :model-value="String(props.modelValue.pollIntervalMs)"
        :placeholder="t('connectionPanel.placeholders.pollInterval')"
        @update:model-value="emit('updateField', 'pollIntervalMs', Number($event ?? 0))"
      />
    </label>

    <label class="field-block">
      <span class="field-label">{{ t("connectionPanel.fields.heartbeatInterval") }}</span>
      <InputText
        :model-value="String(props.modelValue.heartbeatIntervalMs)"
        :placeholder="t('connectionPanel.placeholders.heartbeatInterval')"
        @update:model-value="emit('updateField', 'heartbeatIntervalMs', Number($event ?? 0))"
      />
    </label>

    <div class="reporter-enabled-card field-span-2">
      <div class="reporter-enabled-copy">
        <span class="field-label">{{ t("connectionPanel.fields.reporterEnabled") }}</span>
        <strong>{{ props.modelValue.reporterEnabled ? t("connectionPanel.toggles.enabled") : t("connectionPanel.toggles.reporterDisabled") }}</strong>
        <span>
          {{
            props.variant === "onboarding"
              ? t("connectionPanel.help.reporterEnabledOnboarding")
              : t("connectionPanel.help.reporterEnabledSettings")
          }}
        </span>
      </div>
      <ToggleSwitch
        :model-value="props.modelValue.reporterEnabled"
        :input-id="props.variant === 'onboarding' ? 'onboarding-reporter-enabled' : 'settings-reporter-enabled'"
        @update:model-value="emit('updateField', 'reporterEnabled', Boolean($event))"
      />
    </div>

    <div :class="`${props.headClass || 'settings-section-head'} field-span-2`">
      <strong>{{ t("connectionPanel.sections.reportContent") }}</strong>
      <span>{{ t("connectionPanel.help.reportContent") }}</span>
    </div>

    <div class="compact-toggle-grid field-span-2">
      <div
        v-for="option in props.reporterContentOptions"
        :key="props.variant === 'onboarding' ? `onboarding-${option.key}` : option.key"
        class="compact-toggle-card"
      >
        <div class="compact-toggle-copy">
          <strong>{{ option.label }}</strong>
          <span>{{ option.description }}</span>
        </div>
        <ToggleSwitch
          :model-value="props.modelValue[option.key]"
          :input-id="props.variant === 'onboarding' ? `onboarding-${option.inputId}` : option.inputId"
          @update:model-value="emit('updateField', option.key, Boolean($event))"
        />
      </div>
    </div>
  </div>
</template>

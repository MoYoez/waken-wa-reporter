<script setup lang="ts">
import InputText from "primevue/inputtext";
import Password from "primevue/password";
import ToggleSwitch from "primevue/toggleswitch";
import { useI18n } from "vue-i18n";

import type { ClientConfig } from "@/types";

const props = withDefaults(defineProps<{
  modelValue: ClientConfig;
  currentGeneratedHashKey: string;
  reporterSupported: boolean;
  showConnectionFields?: boolean;
  showDeviceName?: boolean;
  showGeneratedHashKey?: boolean;
  showUseSystemProxy?: boolean;
  deviceFieldClass?: string;
  proxyInputId?: string;
}>(), {
  showConnectionFields: true,
  showDeviceName: true,
  showGeneratedHashKey: true,
  showUseSystemProxy: true,
  deviceFieldClass: "field-block",
  proxyInputId: "settings-use-system-proxy",
});

const emit = defineEmits<{
  updateField: [key: keyof ClientConfig, value: ClientConfig[keyof ClientConfig]];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="panel-grid">
    <label v-if="props.showConnectionFields" class="field-block field-span-2">
      <span class="field-label">{{ t("connectionPanel.fields.baseUrl") }}</span>
      <InputText
        :model-value="props.modelValue.baseUrl"
        :placeholder="t('connectionPanel.placeholders.baseUrl')"
        @update:model-value="emit('updateField', 'baseUrl', $event ?? '')"
      />
    </label>

    <label v-if="props.showConnectionFields" class="field-block field-span-2">
      <span class="field-label">{{ t("connectionPanel.fields.apiToken") }}</span>
      <Password
        :model-value="props.modelValue.apiToken"
        :placeholder="t('connectionPanel.placeholders.apiToken')"
        fluid
        toggle-mask
        :feedback="false"
        @update:model-value="emit('updateField', 'apiToken', $event)"
      />
    </label>

    <label v-if="props.showDeviceName" :class="props.deviceFieldClass">
      <span class="field-label">{{ t("connectionPanel.fields.deviceName") }}</span>
      <InputText
        :model-value="props.modelValue.device"
        :placeholder="t('connectionPanel.placeholders.deviceName')"
        @update:model-value="emit('updateField', 'device', $event ?? '')"
      />
    </label>

    <label v-if="props.showGeneratedHashKey" class="field-block field-span-2">
      <span class="field-label">{{ t("connectionPanel.fields.generatedHashKey") }}</span>
      <InputText
        :model-value="props.currentGeneratedHashKey"
        readonly
        :placeholder="t('connectionPanel.placeholders.generatedHashKey')"
      />
      <small class="field-help">{{ t("connectionPanel.help.generatedHashKey") }}</small>
    </label>

    <div
      v-if="props.reporterSupported && props.showUseSystemProxy"
      class="reporter-enabled-card field-span-2"
    >
      <div class="reporter-enabled-copy">
        <span class="field-label">{{ t("connectionPanel.fields.useSystemProxy") }}</span>
        <strong>{{ props.modelValue.useSystemProxy ? t("connectionPanel.toggles.enabled") : t("connectionPanel.toggles.disabled") }}</strong>
        <span>
          {{ t("connectionPanel.help.useSystemProxy") }}
        </span>
      </div>
      <ToggleSwitch
        :model-value="props.modelValue.useSystemProxy"
        :input-id="props.proxyInputId"
        @update:model-value="emit('updateField', 'useSystemProxy', Boolean($event))"
      />
    </div>
  </div>
</template>
